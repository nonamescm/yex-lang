use vm::{gc::GcRef, OpCode, Symbol, Value};

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Default for Location {
    fn default() -> Self {
        Self { line: 0, column: 0 }
    }
}

use crate::tokens::TokenType;

#[derive(Debug, Clone, Copy)]
pub struct VarDecl {
    pub name: Symbol,
}

impl VarDecl {
    pub fn new(name: Symbol) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    BitOr,
    BitXor,
    Shr,
    Shl,
    Eq,
    Ne,
    And,
    Or,
}

impl<'a> From<BinOp> for &'a [OpCode] {
    fn from(op: BinOp) -> &'a [OpCode] {
        match op {
            BinOp::Less => &[OpCode::Less],
            BinOp::LessEq => &[OpCode::LessEq],
            BinOp::Greater => &[OpCode::Not, OpCode::Less],
            BinOp::GreaterEq => &[OpCode::Not, OpCode::LessEq],
            BinOp::Add => &[OpCode::Add],
            BinOp::Sub => &[OpCode::Sub],
            BinOp::Mul => &[OpCode::Mul],
            BinOp::Div => &[OpCode::Div],
            BinOp::BitAnd => &[OpCode::BitAnd],
            BinOp::BitOr => &[OpCode::BitOr],
            BinOp::BitXor => &[OpCode::Xor],
            BinOp::Shr => &[OpCode::Shr],
            BinOp::Shl => &[OpCode::Shl],
            BinOp::Eq => &[OpCode::Eq],
            BinOp::Ne => &[OpCode::Not, OpCode::Eq],
            BinOp::And => unreachable!(),
            BinOp::Or => unreachable!(),
        }
    }
}

impl TryFrom<TokenType> for BinOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::Less => Ok(BinOp::Less),
            TokenType::LessEq => Ok(BinOp::LessEq),
            TokenType::Greater => Ok(BinOp::Greater),
            TokenType::GreaterEq => Ok(BinOp::GreaterEq),
            TokenType::Add => Ok(BinOp::Add),
            TokenType::Sub => Ok(BinOp::Sub),
            TokenType::Mul => Ok(BinOp::Mul),
            TokenType::Div => Ok(BinOp::Div),
            TokenType::BitAnd => Ok(BinOp::BitAnd),
            TokenType::BitOr => Ok(BinOp::BitOr),
            TokenType::BitXor => Ok(BinOp::BitXor),
            TokenType::Shr => Ok(BinOp::Shr),
            TokenType::Shl => Ok(BinOp::Shl),
            TokenType::Eq => Ok(BinOp::Eq),
            TokenType::Ne => Ok(BinOp::Ne),
            TokenType::And => Ok(BinOp::And),
            TokenType::Or => Ok(BinOp::Or),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnOp {
    Not,
    Neg,
}

impl TryFrom<TokenType> for UnOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::Not => Ok(UnOp::Not),
            TokenType::Sub => Ok(UnOp::Neg),
            _ => Err(()),
        }
    }
}

impl<'a> From<UnOp> for &'a [OpCode] {
    fn from(op: UnOp) -> &'a [OpCode] {
        match op {
            UnOp::Not => &[OpCode::Not],
            UnOp::Neg => &[OpCode::Neg],
        }
    }
}

#[derive(Debug)]
pub enum ExprKind {
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    Bind {
        bind: VarDecl,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Lambda {
        args: Vec<VarDecl>, // specifies the arguments name and types
        body: Box<Expr>,    // the function body
    },
    App {
        callee: Box<Expr>, // the function to call
        args: Vec<Expr>,   // the arguments
        tail: bool,        // true if the call is a tail call
    },

    Var(Symbol),
    Lit(Literal),
    List(Vec<Expr>),

    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Cons {
        head: Box<Expr>,
        tail: Box<Expr>,
    },
    Seq {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    UnOp(UnOp, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Num(f64),
    Str(String),
    Bool(bool),
    Sym(Symbol),
    Unit,
}

impl PartialEq<Value> for Literal {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Literal::Num(a), Value::Num(b)) => a == b,
            (Literal::Str(a), Value::Str(b)) => a == &**b,
            (Literal::Bool(a), Value::Bool(b)) => a == b,
            (Literal::Sym(a), Value::Sym(b)) => a == b,
            (Literal::Unit, Value::Nil) => true,
            _ => false,
        }
    }
}

impl From<Literal> for Value {
    fn from(lit: Literal) -> Value {
        match lit {
            Literal::Num(n) => Value::Num(n),
            Literal::Str(s) => Value::Str(GcRef::new(s)),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Sym(s) => Value::Sym(s),
            Literal::Unit => Value::Nil,
        }
    }
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub location: Location,
}

impl Expr {
    pub fn new(kind: ExprKind, line: usize, column: usize) -> Self {
        Expr {
            kind,
            location: Location { line, column },
        }
    }

    pub fn line(&self) -> usize {
        self.location.line
    }

    pub fn column(&self) -> usize {
        self.location.column
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr {
            kind: ExprKind::Lit(Literal::Unit),
            location: Location { line: 0, column: 0 },
        }
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub location: Location,
}

impl Stmt {
    pub fn new(kind: StmtKind, line: usize, column: usize) -> Self {
        Stmt {
            kind,
            location: Location { line, column },
        }
    }
}

#[derive(Debug)]
pub enum StmtKind {
    Def { bind: VarDecl, value: Expr },
}
