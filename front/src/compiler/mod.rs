use std::collections::HashMap;

use vm::{
    gc::GcRef, stackvec, Bytecode, EnvTable, Fn, List, OpCode, OpCodeMetadata, Symbol,
    Value, YexType, FnKind,
};

use crate::parser::ast::{BinOp, Def, Expr, ExprKind, Literal, Location, Stmt, StmtKind, VarDecl};

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

    fn emit_const(&mut self, const_: Value, node: &Location) {
        if let Some(idx) = self.constants.iter().position(|c| c == &const_) {
            self.emit_op(OpCode::Push(idx), node);
        } else {
            self.constants.push(const_.clone());
            self.emit_op(OpCode::Push(self.constants.len() - 1), node);
        }
    }

    fn emit_save(&mut self, bind: VarDecl, node: &Location) {
        let len = self.scope().locals.len() + 1;
        self.scope_mut().locals.entry(bind.name).or_insert(len);
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

            ExprKind::App { callee, args } => {
                // iterate over the arguments
                // pushing them onto the stack
                for arg in args.iter().rev() {
                    self.expr(arg);
                }

                // compiles the caller
                self.expr(callee);

                // emits the `Call` opcode
                self.emit_op(OpCode::Call(args.len()), loc);
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

            ExprKind::Bind { bind, value, body } => {
                // compiles the value
                self.expr(value);

                // emits the `Save` instruction
                self.emit_save(*bind, loc);

                // compiles the assignment body
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
                // compiles the left side of the or expression
                self.expr(left);

                self.emit_op(OpCode::Dup, loc);

                // apply a unary not to the value on the stack
                self.emit_op(OpCode::Not, loc);

                // keeps track of the jump location
                let then_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), loc);

                // compiles the right side of the or expression
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
                // emits the empty list
                self.emit_const(Value::List(List::new()), &node.location);

                // prepend each element to the list, in the reverse order
                // since it's a linked list
                for x in xs.iter().rev() {
                    self.expr(x);
                    self.emit_op(OpCode::Prep, &node.location);
                }
            }

            ExprKind::Cons { head, tail } => {
                self.expr(tail);
                self.expr(head);

                // prepend the head to the tail
                self.emit_op(OpCode::Prep, &node.location);
            }

            ExprKind::UnOp(op, right) => {
                self.expr(right);
                self.emit_ops((*op).into(), &node.location);
            }

            ExprKind::Seq { left, right } => {
                self.expr(left);
                self.expr(right);
            }

            // compiles field access
            ExprKind::Field { obj, field } => {
                self.expr(obj);
                self.emit_op(OpCode::Get(field.name), &node.location);
            }

            // compiles type instantiation
            ExprKind::New { ty, args } => {
                for arg in args.iter().rev() {
                    self.expr(arg);
                }

                self.expr(ty);
                self.emit_op(OpCode::New(args.len()), &node.location);
            }

            ExprKind::Invoke { obj, field, args } => {
                for arg in args.iter().rev() {
                    self.expr(arg);
                }

                self.expr(obj);
                self.emit_op(OpCode::Invk(field.name, args.len()), &node.location);
            }
        }
    }

    fn stmt(&mut self, node: &Stmt) {
        match &node.kind {
            StmtKind::Def(Def { bind, value, .. }) => {
                self.expr(value);
                self.emit_op(OpCode::Savg(bind.name), &node.location);
            }
            StmtKind::Type {
                name,
                methods,
                params,
                init,
            } => {
                self.typedef(
                    name,
                    methods,
                    init,
                    &params.iter().map(|x| x.name).collect::<Vec<_>>(),
                    &node.location,
                );
            }
        }
    }

    fn typedef(
        &mut self,
        decl: &VarDecl,
        methods: &[Def],
        init: &Option<Def>,
        params: &[Symbol],
        loc: &Location,
    ) {
        let mut table = EnvTable::new();
        for m in methods {
            let func = match &m.value.kind {
                ExprKind::Lambda { args, body } => Value::Fn(self.lambda_expr(args, body, loc)),
                _ => unreachable!(),
            };

            table.insert(m.bind.name, func);
        }

        let mut ty = YexType::new(decl.name, table, params.to_vec());

        if let Some(init) = init {
            let func = match &init.value.kind {
                ExprKind::Lambda { args, body } => self.lambda_expr(args, body, loc),
                _ => unreachable!(),
            };

            ty = ty.with_initializer(func);
        }

        self.emit_const(Value::Type(GcRef::new(ty)), loc);
        self.emit_op(OpCode::Savg(decl.name), loc);
    }

    pub fn compile_stmts(mut self, stmts: &[Stmt]) -> (Vec<OpCodeMetadata>, Vec<Value>) {
        self.scope_stack.push(Scope::new());
        for stmt in stmts {
            self.stmt(stmt);
        }
        (self.scope_stack.pop().unwrap().opcodes, self.constants)
    }
}
