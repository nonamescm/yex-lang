use std::collections::HashMap;

use vm::{
    gc::GcRef, stackvec, Bytecode, EnvTable, Fn, FnKind, List, OpCode, OpCodeMetadata, Symbol,
    Value, YexModule,
};

use crate::parser::ast::{
    BinOp, Bind, Def, Expr, ExprKind, Literal, Location, MatchArm, Pattern, Stmt, StmtKind, VarDecl,
};

#[derive(Default)]
struct Scope {
    opcodes: Vec<OpCodeMetadata>,
    locals: HashMap<Symbol, usize>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct Compiler {
    scope_stack: Vec<Scope>,
    constants: Vec<Value>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler::default()
    }

    pub fn compile_expr(mut self, expr: &Expr) -> (Bytecode, Vec<Value>) {
        self.scope_stack.push(Scope::new());
        self.expr(expr);
        (self.scope_stack.pop().unwrap().opcodes, self.constants)
    }

    fn scope_mut(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().unwrap()
    }

    fn scope(&self) -> &Scope {
        self.scope_stack.last().unwrap()
    }

    fn emit_op(&mut self, op: OpCode, loc: &Location) {
        self.scope_mut().opcodes.push(OpCodeMetadata {
            opcode: op,
            line: loc.line,
            column: loc.column,
        })
    }

    fn emit_ops(&mut self, ops: &[OpCode], node: &Location) {
        for op in ops {
            self.emit_op(*op, node);
        }
    }

    fn emit_lit(&mut self, lit: &Literal, node: &Location) {
        if let Some(idx) = self.constants.iter().position(|c| lit == c) {
            self.emit_op(OpCode::Push(idx), node);
        } else {
            self.constants.push(lit.clone().into());
            self.emit_op(OpCode::Push(self.constants.len() - 1), node);
        }
    }

    fn emit_const(&mut self, const_: Value, node: &Location) -> usize {
        if !matches!(const_, Value::Module(_)) {
            if let Some(idx) = self.constants.iter().position(|c| c == &const_) {
                self.emit_op(OpCode::Push(idx), node);
                return idx;
            }
        }

        let pos = self.constants.len();
        self.constants.push(const_);
        self.emit_op(OpCode::Push(pos), node);
        pos
    }

    fn emit_save(&mut self, bind: VarDecl, node: &Location) {
        let len = self.scope().locals.len();

        self.scope_mut().locals.insert(bind, len);
        self.emit_op(OpCode::Save(len), node);
    }

    fn if_expr(&mut self, cond: &Expr, then: &Expr, else_: &Expr, loc: &Location) {
        // compiles the codition
        self.expr(cond);

        // keeps track of the jump offset
        let then_label = self.scope().opcodes.len();
        self.emit_op(OpCode::Jmf(0), loc);

        // compiles the then branch
        self.expr(then);

        // keeps track of the else jump offset
        let else_label = self.scope().opcodes.len();
        self.emit_op(OpCode::Jmp(0), loc);

        // fix the then jump offset
        self.scope_mut().opcodes[then_label].opcode = OpCode::Jmf(self.scope().opcodes.len());

        self.expr(else_);

        // fix the else jump offset
        self.scope_mut().opcodes[else_label].opcode = OpCode::Jmp(self.scope().opcodes.len());
    }

    fn match_arm(&mut self, arm: &MatchArm, loc: &Location) -> usize {
        // creates a stack of jmp indexes to be fixed later
        let (should_pop, fix_stack) = self.match_pattern(&arm.cond, true, false, loc);

        // the arm may request more than one pop after matching to clean up the stack
        if should_pop {
            self.emit_op(OpCode::Pop, loc);
        }

        // emits the guard check if it exists
        let guard_label = if let Some(guard) = &arm.guard {
            self.expr(guard);
            let label = self.scope().opcodes.len();
            self.emit_op(OpCode::Jmf(0), loc);
            Some(label)
        } else {
            None
        };

        // emits an extra pop, since we Dup'd the condition
        self.emit_op(OpCode::Pop, loc);

        self.expr(&arm.body);

        // emit a new jump, since we need to jump to the end of the when if the condition was
        // met
        let jmp_label = self.scope().opcodes.len();
        self.emit_op(OpCode::Jmp(0), loc);

        // fix the jump offset
        for label in fix_stack {
            self.scope_mut().opcodes[label].opcode = OpCode::Jmf(self.scope().opcodes.len());
        }

        guard_label.map(|label| {
            self.scope_mut().opcodes[label].opcode = OpCode::Jmf(self.scope().opcodes.len());
        });

        // sometimes the arm needs to pop if it returned false, so, we check for this case here
        if should_pop {
            self.emit_op(OpCode::Pop, loc);
        }

        jmp_label
    }

    // the return type of this function is a tuple, where the first element mark how many extra
    // pops should be emitted after the function ends, and the second one is the index of the jumps
    // that were emitted and need to be patched.
    fn match_pattern(
        &mut self,
        pattern: &Pattern,
        duplicate: bool,
        global: bool,
        loc: &Location,
    ) -> (bool, Vec<usize>) {
        // all top-level checks needs to duplicate the value on the stack in case it doesn't match
        // so the next arm can check against it again
        if duplicate {
            self.emit_op(OpCode::Dup, loc);
        }

        match pattern {
            Pattern::Lit(ref lt) => {
                // compares the value against the literal
                self.emit_lit(lt, loc);
                self.emit_op(OpCode::Eq, loc);

                // in case it returns false, emit the necessary jump instruction
                let label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), loc);

                (false, vec![label])
            }
            Pattern::Id(id) => {
                if global {
                    self.emit_op(OpCode::Savg(*id), loc);
                } else {
                    self.emit_save(*id, loc);
                }
                (false, vec![])
            }
            Pattern::Variant(path, args) => {
                // gets the tag of the value
                let name = path
                    .iter()
                    .map(Symbol::as_str)
                    .map(str::to_string)
                    .collect::<Vec<String>>()
                    .join(".");

                // Dups the condition again
                self.emit_op(OpCode::Dup, loc);

                // compares it with the tag of the value on the top of the stack
                self.emit_op(OpCode::TagOf, loc);
                self.emit_const(Symbol::from(name).into(), loc);
                self.emit_op(OpCode::Eq, loc);

                // emits the jump instruction that will jump to the next clause if it doesn't match
                let mut labels = vec![self.scope().opcodes.len()];
                self.emit_op(OpCode::Jmf(0), loc);

                // gets the inner tuple from the tagged value
                self.emit_op(OpCode::TagTup, loc);

                // checks if the two "tuples" have the same length
                self.emit_op(OpCode::Dup, loc);
                self.emit_op(OpCode::Len, loc);
                self.emit_lit(&Literal::Num(args.len() as f64), loc);
                self.emit_op(OpCode::Eq, loc);

                // emit the jump place-holder
                labels.push(self.scope().opcodes.len());

                self.emit_op(OpCode::Jmf(0), loc);

                for (idx, arg) in args.iter().enumerate() {
                    self.emit_op(OpCode::Dup, loc);
                    self.emit_op(OpCode::TupGet(idx), loc);

                    let (should_pop, offsets) = self.match_pattern(arg, false, false, loc);
                    if should_pop {
                        self.emit_op(OpCode::Pop, loc);
                    }
                    offsets.iter().for_each(|it| labels.push(*it));
                }

                (true, labels)
            }
        }
    }

    fn match_expr(&mut self, cond: &Expr, arms: &[MatchArm], _loc: &Location) {
        // compiles the condition
        // TODO: this is a bit hacky, but it works for now
        self.expr(cond);

        // keep track of all the jump offsets
        let mut jmps = vec![];

        for arm in arms {
            let jmp = self.match_arm(arm, &arm.location);
            jmps.push(jmp);
        }

        // fix all the jump offsets
        let ip = self.scope().opcodes.len();
        for jmp in jmps {
            self.scope_mut().opcodes[jmp].opcode = OpCode::Jmp(ip);
        }
    }

    fn lambda_expr(&mut self, args: &[Pattern], body: &Expr, loc: &Location) -> GcRef<Fn> {
        // creates the lambda scope
        self.scope_stack.push(Scope::new());

        let mut fix_stack = vec![];

        // emit all the patterns, most of them are probably just variable assignments, but some of
        // them may be complex patterns, so we still need to check for the should_pop value
        for arg in args.iter() {
            let (should_pop, fixes) = self.match_pattern(arg, true, false, loc);

            // pop any extra values that were left on the stack
            if should_pop {
                self.emit_op(OpCode::Pop, loc);
            }
            self.emit_op(OpCode::Pop, loc);

            fix_stack.extend(fixes);
        }

        // compiles the body
        self.expr(body);

        // emit a jump to ignore the
        let jmp_label = self.scope().opcodes.len();
        self.emit_op(OpCode::Jmp(0), loc);

        for offset in fix_stack {
            self.scope_mut().opcodes[offset].opcode = OpCode::Jmf(self.scope().opcodes.len());
        }

        // emit the call to raise
        self.emit_const("No match of rhs value".to_string().into(), loc);
        self.emit_const(Symbol::from("MatchError").into(), loc);
        self.emit_op(OpCode::Loag("raise".into()), loc);
        self.emit_op(OpCode::Call(2), loc);

        // patch the jump to the end
        self.scope_mut().opcodes[jmp_label].opcode = OpCode::Jmp(self.scope().opcodes.len());

        // pops the lambda scope
        let Scope { opcodes, .. } = self.scope_stack.pop().unwrap();

        // convert it to a `Fn` struct
        let func = Fn {
            body: GcRef::new(FnKind::Bytecode(opcodes)),
            arity: args.len(),
            args: stackvec![],
        };

        // push the function onto the stack
        GcRef::new(func)
    }

    fn expr(&mut self, node: &Expr) {
        let loc = &node.location;

        match &node.kind {
            // pushes a literal value onto the stack
            ExprKind::Lit(lit) => self.emit_lit(lit, loc),

            // compiles a lambda expression
            ExprKind::Lambda { args, body } => {
                let func = self.lambda_expr(args, body, loc);
                self.emit_const(Value::Fn(func), loc);
            }

            ExprKind::App { callee, args, tail } => {
                // iterate over the arguments
                // pushing them onto the stack
                for arg in args.iter() {
                    self.expr(arg);
                }

                self.emit_op(OpCode::RevN(args.len()), loc);

                // compiles the caller
                self.expr(callee);

                // emits the `Call` opcode
                if *tail {
                    self.emit_op(OpCode::TCall(args.len()), loc);
                } else {
                    self.emit_op(OpCode::Call(args.len()), loc);
                }
            }

            ExprKind::Var(name) => {
                // get the local index
                let pred = self.scope().locals.get(name).copied();

                if let Some(idx) = pred {
                    // if the variable is in the current scope
                    // emit the `Load` opcode, which loads a local
                    self.emit_op(OpCode::Load(idx), loc);
                } else {
                    // otherwise emit the `Loag` opcode, which loads a global
                    self.emit_op(OpCode::Loag(*name), loc);
                }
            }

            ExprKind::If { cond, then, else_ } => self.if_expr(cond, then, else_, loc),

            ExprKind::Match { expr, arms } => self.match_expr(expr, arms, loc),

            ExprKind::Let { bind, value, body } => {
                // compiles the value and pushes it on the stack
                self.expr(value);

                // try to match against the value
                let (should_pop, fix_stack) = self.match_pattern(bind, true, false, loc);

                if should_pop {
                    self.emit_op(OpCode::Pop, loc);
                }

                // pops the extra value that is left on the stack
                self.emit_op(OpCode::Pop, loc);

                self.expr(body);

                // emit a jump to ignore the
                let jmp_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), loc);

                for offset in fix_stack {
                    self.scope_mut().opcodes[offset].opcode =
                        OpCode::Jmf(self.scope().opcodes.len());
                }

                // emit a pop if necessary
                if should_pop {
                    self.emit_op(OpCode::Pop, loc);
                }

                // emit the call to raise
                self.emit_const("No match of rhs value".to_string().into(), loc);
                self.emit_const(Symbol::from("MatchError").into(), loc);
                self.emit_op(OpCode::Loag("raise".into()), loc);
                self.emit_op(OpCode::Call(2), loc);

                // patch the jump to the end
                self.scope_mut().opcodes[jmp_label].opcode =
                    OpCode::Jmp(self.scope().opcodes.len());
            }
            ExprKind::Def {
                bind: Bind { bind, value, .. },
                body,
            } => {
                // compiles the value
                self.expr(value);

                // emits the `Save` instruction
                self.emit_save(*bind, loc);

                // emits a `nil` value, since everything should return something
                self.expr(body);
            }

            ExprKind::Binary { left, op, right } if op == &BinOp::And => {
                // compiles the left side of the and expression
                self.expr(left);

                // duplicate the value on the stack
                self.emit_op(OpCode::Dup, loc);

                // keeps track of the jump location
                let then_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), loc);

                // pop's the duplicated left value
                self.emit_op(OpCode::Pop, loc);
                self.expr(right);

                // fix the jump offset
                self.scope_mut().opcodes[then_label].opcode =
                    OpCode::Jmf(self.scope().opcodes.len());
            }

            ExprKind::Binary { left, op, right } if op == &BinOp::Or => {
                // compiles the left side of the and expression
                self.expr(left);

                // duplicate the value on the stack
                self.emit_op(OpCode::Dup, loc);

                // negates the value on the stack, since this is an or
                self.emit_op(OpCode::Not, loc);

                // keeps track of the jump location
                let then_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), loc);

                // pop's the duplicated left value
                self.emit_op(OpCode::Pop, loc);
                self.expr(right);

                // fix the jump offset
                self.scope_mut().opcodes[then_label].opcode =
                    OpCode::Jmf(self.scope().opcodes.len());
            }

            ExprKind::Binary { left, op, right } => {
                self.expr(left);
                self.expr(right);
                self.emit_ops((*op).into(), loc);
            }

            ExprKind::List(xs) => {
                // prepend each element to the list, in the reverse order
                // since it's a linked list
                for x in xs.iter() {
                    self.expr(x);
                }

                // emits the empty list
                self.emit_const(Value::List(List::new()), loc);

                for _ in 0..xs.len() {
                    self.emit_op(OpCode::Prep, loc);
                }
            }

            ExprKind::Cons { head, tail } => {
                self.expr(head);
                self.expr(tail);

                // prepend the head to the tail
                self.emit_op(OpCode::Prep, loc);
            }

            ExprKind::UnOp(op, right) => {
                self.expr(right);
                self.emit_ops((*op).into(), loc);
            }

            // compiles a method reference access
            ExprKind::MethodRef { ty, method } => {
                self.expr(ty);
                self.emit_op(OpCode::Ref(*method), loc);
            }

            ExprKind::Try { body, bind, rescue } => {
                // keeps track of the try location
                let try_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Try(0), loc);

                // compiles the body
                self.expr(body);

                // ends the try block
                self.emit_op(OpCode::EndTry, loc);

                // keep track of the new jump location
                let end_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), loc);

                // fix the try jump offset
                self.scope_mut().opcodes[try_label].opcode =
                    OpCode::Try(self.scope().opcodes.len());

                // pop the return from the try block
                self.emit_op(OpCode::Pop, loc);

                // saves the exception to the bind
                self.emit_save(*bind, loc);

                // compiles the rescue block
                self.expr(rescue);

                // fix the end of the rescue block
                self.scope_mut().opcodes[end_label].opcode =
                    OpCode::Jmp(self.scope().opcodes.len());
            }

            ExprKind::Tuple(xs) => {
                for x in xs.iter().rev() {
                    self.expr(x);
                }

                self.emit_op(OpCode::Tup(xs.len()), loc);
            }
        }
    }

    fn stmt(&mut self, node: &Stmt) {
        let loc = &node.location;

        match &node.kind {
            // compiles a `def` statement into a `Savg` instruction
            StmtKind::Def(Def { bind, value, .. }) => {
                self.expr(value);
                self.emit_op(OpCode::Savg(*bind), &node.location);
            }

            // compiles a `let` statement into a `Savg` instruction
            StmtKind::Let { bind, value } => {
                // compiles the value and pushes it on the stack
                self.expr(value);

                // try to match against the value
                let (should_pop, fix_stack) = self.match_pattern(bind, true, true, loc);

                if should_pop {
                    self.emit_op(OpCode::Pop, loc);
                }

                // pops the extra value that is left on the stack
                self.emit_op(OpCode::Pop, loc);

                // emit a jump to ignore the
                let jmp_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), loc);

                for offset in fix_stack {
                    self.scope_mut().opcodes[offset].opcode =
                        OpCode::Jmf(self.scope().opcodes.len());
                }

                // emit a pop if necessary
                if should_pop {
                    self.emit_op(OpCode::Pop, loc);
                }

                // emit the call to raise
                self.emit_const("No match of rhs value".to_string().into(), loc);
                self.emit_const(Symbol::from("MatchError").into(), loc);
                self.emit_op(OpCode::Loag("raise".into()), loc);
                self.emit_op(OpCode::Call(2), loc);

                // patch the jump to the end
                self.scope_mut().opcodes[jmp_label].opcode =
                    OpCode::Jmp(self.scope().opcodes.len());
            }

            // compiles a `module` declaration into an YexModule and save the module to a global name
            StmtKind::Type {
                name,
                variants,
                members,
            } => {
                self.type_(name, variants, members, &node.location);
            }
        }
    }

    fn type_(
        &mut self,
        decl: &VarDecl,
        variants: &[(VarDecl, Vec<VarDecl>)],
        members: &[Def],
        loc: &Location,
    ) {
        let mut table = EnvTable::new();
        for m in members {
            let func = match &m.value.kind {
                ExprKind::Lambda { args, body } => Value::Fn(self.lambda_expr(args, body, loc)),
                _ => unreachable!(),
            };

            table.insert(m.bind, func);
        }

        let index = self.constants.len();
        self.constants.push(YexModule::default().into()); // place-holder, since I'm still building the type I can't emit it yet.

        let mut patch_list = vec![];

        for (name, args) in variants {
            if args.is_empty() {
                patch_list.push(name.as_str().split('.').last().unwrap().into());
                continue;
            }

            let scope = Scope::new();
            self.scope_stack.push(scope);

            self.emit_op(OpCode::Tup(args.len()), loc);
            self.emit_op(OpCode::Push(index), loc);
            self.emit_op(OpCode::Tag(*name), loc);

            let Scope { opcodes, .. } = self.scope_stack.pop().unwrap();

            let constructor = Fn {
                body: GcRef::new(FnKind::Bytecode(opcodes)),
                arity: args.len(),
                args: stackvec![],
            };

            table.insert(
                name.as_str().split('.').last().unwrap().into(),
                constructor.into(),
            );
        }

        let mut type_ = GcRef::new(YexModule::new(*decl, table));
        for entry in patch_list {
            unsafe {
                let clone = type_.clone();
                type_
                    .mut_ref()
                    .fields
                    .insert(entry, Value::Tagged(clone, entry, vec![].into()));
            }
        }

        self.constants[index] = Value::Module(type_);
        self.emit_op(OpCode::Push(index), loc);
        self.emit_op(OpCode::Savg(*decl), loc);
    }

    pub fn compile_stmts(mut self, stmts: &[Stmt]) -> (Vec<OpCodeMetadata>, Vec<Value>) {
        self.scope_stack.push(Scope::new());
        for stmt in stmts {
            self.stmt(stmt);
        }
        (self.scope_stack.pop().unwrap().opcodes, self.constants)
    }
}
