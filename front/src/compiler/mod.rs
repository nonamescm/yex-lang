use std::collections::HashMap;

use vm::{gc::GcRef, Bytecode, Either, Fun, List, OpCode, OpCodeMetadata, Symbol, Value};

use crate::parser::ast::{Expr, ExprKind, Literal, VarDecl, Stmt, StmtKind, Location};

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

    fn expr(&mut self, node: &Expr) {
        match &node.kind {
            ExprKind::Lit(lit) => self.emit_lit(lit, &node.location),

            ExprKind::Lambda { args, body } => {
                let mut scope = Scope {
                    opcodes: Vec::new(),
                    locals: HashMap::new(),
                };

                for (idx, arg) in args.iter().enumerate() {
                    scope.locals.insert(arg.name, idx);
                    scope.opcodes.push(OpCodeMetadata {
                        line: node.location.line,
                        column: node.location.column,
                        opcode: OpCode::Save(idx),
                    });
                }

                self.scope_stack.push(scope);
                self.expr(body);
                let Scope { opcodes, .. } = self.scope_stack.pop().unwrap();

                let func = Fun {
                    body: GcRef::new(Either::Left(opcodes)),
                    arity: args.len(),
                };

                self.emit_const(Value::Fun(GcRef::new(func)), &node.location)
            }

            ExprKind::App { callee, args } => {
                for arg in args {
                    self.expr(arg);
                }
                self.expr(callee);
                self.emit_op(OpCode::Call(args.len()), &node.location);
            }

            ExprKind::Var(name) => {
                let pred = self.scope().locals.get(name).copied();

                if let Some(idx) = pred {
                    self.emit_op(OpCode::Load(idx), &node.location);
                } else {
                    self.emit_op(OpCode::Loag(*name), &node.location);
                }
            }

            ExprKind::If { cond, then, else_ } => {
                self.expr(cond);
                let then_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), &node.location);

                self.expr(then);

                let else_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), &node.location);

                self.scope_mut().opcodes[then_label].opcode =
                    OpCode::Jmf(self.scope().opcodes.len());

                self.expr(else_);

                self.scope_mut().opcodes[else_label].opcode =
                    OpCode::Jmp(self.scope().opcodes.len());
            }

            ExprKind::Bind { bind, value, body } => {
                self.expr(value);
                self.emit_save(*bind, &node.location);
                self.expr(body);
            }

            ExprKind::Binary { left, op, right } => {
                self.expr(left);
                self.expr(right);
                self.emit_ops((*op).into(), &node.location);
            }

            ExprKind::List(xs) => {
                self.emit_const(Value::List(List::new()), &node.location);
                for x in xs.iter().rev() {
                    self.expr(x);
                    self.emit_op(OpCode::Prep, &node.location);
                }
            }

            ExprKind::Cons { head, tail } => {
                self.expr(tail);
                self.expr(head);
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
        }
    }

    fn stmt(&mut self, node: &Stmt) {
        match &node.kind {
            StmtKind::Def { bind, value } => {
                self.expr(value);
                self.emit_op(OpCode::Savg(bind.name), &node.location);
            }
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
