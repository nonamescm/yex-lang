use std::collections::HashMap;

use vm::{
    gc::GcRef, stackvec, Bytecode, Constant, Either, Fun, List, OpCode, OpCodeMetadata, Symbol,
};

use crate::parser::ast::{Expr, ExprKind, Literal, VarDecl};

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
    constants: Vec<Constant>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler::default()
    }

    pub fn compile_expr(mut self, expr: &Expr) -> (Bytecode, Vec<Constant>) {
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

    fn emit_op(&mut self, op: OpCode, node: &Expr) {
        self.scope_mut().opcodes.push(OpCodeMetadata {
            opcode: op,
            line: node.line,
            column: node.column,
        })
    }

    fn emit_ops(&mut self, ops: &[OpCode], node: &Expr) {
        for op in ops {
            self.emit_op(*op, node);
        }
    }

    fn emit_lit(&mut self, lit: &Literal, node: &Expr) {
        if let Some(idx) = self.constants.iter().position(|c| lit == c) {
            self.emit_op(OpCode::Push(idx), node);
        } else {
            self.constants.push(lit.clone().into());
            self.emit_op(OpCode::Push(self.constants.len() - 1), node);
        }
    }

    fn emit_const(&mut self, const_: Constant, node: &Expr) {
        if let Some(idx) = self.constants.iter().position(|c| c == &const_) {
            self.emit_op(OpCode::Push(idx), node);
        } else {
            self.constants.push(const_.clone());
            self.emit_op(OpCode::Push(self.constants.len() - 1), node);
        }
    }

    fn emit_save(&mut self, bind: VarDecl, node: &Expr) {
        let len = self.scope().locals.len() + 1;
        self.scope_mut().locals.entry(bind.name).or_insert(len);
        self.emit_op(OpCode::Save(len), node);
    }

    fn expr(&mut self, node: &Expr) {
        match &node.kind {
            ExprKind::Lit(lit) => self.emit_lit(lit, node),

            ExprKind::Lambda { args, body } => {
                let mut scope = Scope {
                    opcodes: Vec::new(),
                    locals: HashMap::new(),
                };

                for (idx, arg) in args.iter().enumerate() {
                    scope.locals.insert(arg.name, idx);
                }

                self.scope_stack.push(scope);
                self.expr(body);
                let Scope { opcodes, .. } = self.scope_stack.pop().unwrap();
                let func = Fun {
                    body: GcRef::new(Either::Left(opcodes)),
                    args: stackvec![],
                    arity: args.len(),
                };

                self.emit_const(Constant::Fun(GcRef::new(func)), node)
            }

            ExprKind::App { callee, args } => {
                for arg in args {
                    self.expr(arg);
                }
                self.expr(callee);
                self.emit_op(OpCode::Call(args.len()), node);
            }

            ExprKind::Var(name) => {
                let pred = self.scope().locals.get(name).copied();

                if let Some(idx) = pred {
                    self.emit_op(OpCode::Load(idx), node);
                } else {
                    self.emit_op(OpCode::Loag(*name), node);
                }
            }

            ExprKind::If { cond, then, else_ } => {
                self.expr(cond);
                let then_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmf(0), node);

                self.expr(then);

                let else_label = self.scope().opcodes.len();
                self.emit_op(OpCode::Jmp(0), node);

                self.scope_mut().opcodes[then_label].opcode =
                    OpCode::Jmf(self.scope().opcodes.len());

                self.expr(else_);

                self.scope_mut().opcodes[else_label].opcode =
                    OpCode::Jmp(self.scope().opcodes.len());
            }

            ExprKind::Bind { bind, value, body } => {
                self.expr(value);
                self.emit_save(*bind, node);
                self.expr(body);
            }

            ExprKind::Binary { left, op, right } => {
                self.expr(left);
                self.expr(right);
                self.emit_ops((*op).into(), node);
            }

            ExprKind::List(xs) if xs.len() >= 1 => {
                self.emit_const(Constant::List(GcRef::new(List::new())), node);
                for x in xs.iter().rev() {
                    self.expr(x);
                    self.emit_op(OpCode::Prep, node);
                }
            }

            ExprKind::List(..) => {
                self.emit_const(Constant::List(GcRef::new(List::new())), node);
            }

            ExprKind::Cons { head, tail } => {
                self.expr(tail);
                self.expr(head);
                self.emit_op(OpCode::Prep, node);
            }

            ExprKind::UnOp(op, right) => {
                self.expr(right);
                self.emit_ops((*op).into(), node);
            }

            ExprKind::Seq { left, right } => {
                self.expr(left);
                self.expr(right);
            }
        }
    }
}
