use std::collections::HashMap;

use vm::{
    gc::GcRef, stackvec, Bytecode, EnvTable, Fn, FnKind, List, OpCode, OpCodeMetadata, Symbol,
    Value, YexModule,
};

use crate::parser::ast::{
    ArmType, BinOp, Bind, Def, Expr, ExprKind, Literal, Location, Stmt, StmtKind, VarDecl,
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
        if let Some(idx) = self.constants.iter().position(|c| c == &const_) {
            self.emit_op(OpCode::Push(idx), node);
            idx
        } else {
            let pos = self.constants.len();
            self.constants.push(const_.clone());
            self.emit_op(OpCode::Push(pos), node);
            pos
        }
    }

    fn emit_save(&mut self, bind: VarDecl, node: &Location) {
        let len = self.scope().locals.len();

        self.scope_mut().locals.insert(bind.name, len);
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

    fn when_arm(&mut self, arm: &ArmType, loc: &Location) -> usize {
        match arm {
            ArmType::Arm(when) => {
                self.emit_op(OpCode::Dup, loc);

                // emits the current arm's condition
                self.expr(&when.cond);

                // check if the cond is equal to the current arm's condition
                self.emit_op(OpCode::Eq, loc);

                // keeps track of the jump offset
                let label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), loc);

                // if the cond is equal to the current arm's condition, check if there's any
                // guard, and, if so, evaluate it, checking if it's true or false
                let guard_label = if let Some(guard) = &when.guard {
                    self.expr(guard);
                    let label = self.scope().opcodes.len();
                    self.emit_op(OpCode::Jmf(0), loc);
                    Some(label)
                } else {
                    None
                };

                // emits an extra pop, since we Dup'd the condition
                self.emit_op(OpCode::Pop, loc);

                self.expr(&when.body);

                // emit a new jump, since we need to jump to the end of the when if the condition was
                // met
                let jmp_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), loc);

                // fix the jump offset
                self.scope_mut().opcodes[label].opcode = OpCode::Jmf(self.scope().opcodes.len());
                guard_label.map(|label| {
                    self.scope_mut().opcodes[label].opcode =
                        OpCode::Jmf(self.scope().opcodes.len());
                });

                jmp_label
            }
            ArmType::Else(arm) => {
                // duplicate the value on the stack
                self.emit_op(OpCode::Dup, loc);

                // save the matched value to a local
                self.emit_save(arm.bind, loc);

                // check if there's any guard, and, if so, evaluate it, checking if it's true or
                // not
                let guard_label = if let Some(guard) = &arm.guard {
                    self.expr(guard);

                    let label = self.scope().opcodes.len();
                    self.emit_op(OpCode::Jmf(0), loc);
                    Some(label)
                } else {
                    None
                };

                // emit an extra pop, since we Dup'd the condition
                self.emit_op(OpCode::Pop, loc);

                // compiles the body of the arm
                self.expr(&arm.body);

                // emit a new jump and return the jump index to be fixed later
                let jmp_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), &arm.location);

                // if there was a guard, fix the jump offset
                guard_label.map(|label| {
                    self.scope_mut().opcodes[label].opcode =
                        OpCode::Jmf(self.scope().opcodes.len());
                });

                jmp_label
            }
        }
    }

    fn when_expr(&mut self, cond: &Expr, arms: &[ArmType], loc: &Location) {
        // compiles the condition
        // TODO: this is a bit hacky, but it works for now
        self.expr(cond);

        // keep track of all the jump offsets
        let mut jmps = vec![];

        for arm in arms {
            let jmp = self.when_arm(arm, loc);
            jmps.push(jmp);
        }

        // fix all the jump offsets
        let ip = self.scope().opcodes.len();
        for jmp in jmps {
            self.scope_mut().opcodes[jmp].opcode = OpCode::Jmp(ip);
        }
    }

    fn lambda_expr(&mut self, args: &[VarDecl], body: &Expr, loc: &Location) -> GcRef<Fn> {
        // creates the lambda scope
        let mut scope = Scope {
            opcodes: Vec::new(),
            locals: HashMap::new(),
        };

        for (idx, arg) in args.iter().enumerate() {
            // insert the argument into the scope
            scope.locals.insert(arg.name, idx);

            // pushes the opcode to save the argument
            let op = OpCodeMetadata::new(loc.line, loc.column, OpCode::Save(idx));
            scope.opcodes.push(op);
        }

        self.scope_stack.push(scope);

        // compiles the body
        self.expr(body);

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
                for arg in args.iter().rev() {
                    self.expr(arg);
                }

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

            ExprKind::When { expr, arms } => self.when_expr(expr, arms, loc),

            ExprKind::Let {
                bind: Bind { bind, value, .. },
                body,
            }
            | ExprKind::Def {
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
                self.expr(tail);
                self.expr(head);

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
                self.emit_op(OpCode::Ref(method.name), loc);
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

            ExprKind::Struct { ty, fields } => {
                self.emit_op(OpCode::Struct(*ty), loc);

                for (name, value) in fields {
                    self.expr(value);
                    self.emit_op(OpCode::Set(*name), loc);
                }
            }

            ExprKind::Get { field, obj } => {
                self.expr(obj);
                self.emit_op(OpCode::Get(*field), loc);
            }
        }
    }

    fn stmt(&mut self, node: &Stmt) {
        match &node.kind {
            // compiles a `def` statement into a `Savg` instruction
            StmtKind::Def(Def { bind, value, .. }) => {
                self.expr(value);
                self.emit_op(OpCode::Savg(bind.name), &node.location);
            }

            // compiles a `let` statement into a `Savg` instruction
            StmtKind::Let(Bind { bind, value, .. }) => {
                self.expr(value);
                self.emit_op(OpCode::Savg(bind.name), &node.location);
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

            table.insert(m.bind.name, func);
        }

        let ty = YexModule::new(decl.name, table);
        let ty = Value::Module(GcRef::new(ty));

        let index = self.emit_const(ty, loc);
        self.emit_op(OpCode::Savg(decl.name), loc);

        for (name, args) in variants {
            let scope = Scope::new();
            self.scope_stack.push(scope);

            self.emit_op(OpCode::Tup(args.len()), loc);
            self.emit_op(OpCode::Push(index), loc);
            self.emit_op(OpCode::Tag(name.name), loc);

            let Scope { opcodes, .. } = self.scope_stack.pop().unwrap();

            let constructor = Fn {
                body: GcRef::new(FnKind::Bytecode(opcodes)),
                arity: args.len(),
                args: stackvec![],
            };

            self.emit_const(constructor.into(), loc);
            self.emit_save(*name, loc);
        }
    }

    pub fn compile_stmts(mut self, stmts: &[Stmt]) -> (Vec<OpCodeMetadata>, Vec<Value>) {
        self.scope_stack.push(Scope::new());
        for stmt in stmts {
            self.stmt(stmt);
        }
        (self.scope_stack.pop().unwrap().opcodes, self.constants)
    }
}
