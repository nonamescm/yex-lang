use vm::{Symbol, Constant, gc::GcRef, OpCode};

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

#[derive(Debug, Clone, Copy)]
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

impl<'a> Into<&'a [OpCode]> for BinOp {
    fn into(self) -> &'a [OpCode] {
        match self {
            Self::Less => &[OpCode::Less],
            Self::LessEq => &[OpCode::LessEq],
            Self::Greater => &[OpCode::Not, OpCode::Less],
            Self::GreaterEq => &[OpCode::Not, OpCode::LessEq],
            Self::Add => &[OpCode::Add],
            Self::Sub => &[OpCode::Sub],
            Self::Mul => &[OpCode::Mul],
            Self::Div => &[OpCode::Div],
            Self::BitAnd => &[OpCode::BitAnd],
            Self::BitOr => &[OpCode::BitOr],
            Self::BitXor => &[OpCode::Xor],
            Self::Shr => &[OpCode::Shr],
            Self::Shl => &[OpCode::Shl],
            Self::Eq => &[OpCode::Eq],
            Self::Ne => &[OpCode::Not, OpCode::Eq],
            Self::And => unreachable!(),
            Self::Or => unreachable!(),
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

impl<'a> Into<&'a [OpCode]> for UnOp {
    fn into(self) -> &'a [OpCode] {
        match self {
            Self::Not => &[OpCode::Not],
            Self::Neg => &[OpCode::Neg],
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
        callee: Box<Expr>,
        args: Vec<Expr>,
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

impl PartialEq<Constant> for Literal {
    fn eq(&self, other: &Constant) -> bool {
        match (self, other) {
            (Literal::Num(a), Constant::Num(b)) => a == b,
            (Literal::Str(a), Constant::Str(b)) => a == &**b,
            (Literal::Bool(a), Constant::Bool(b)) => a == b,
            (Literal::Sym(a), Constant::Sym(b)) => a == b,
            (Literal::Unit, Constant::Nil) => true,
            _ => false,
        }
    }
}

impl Into<Constant> for Literal {
    fn into(self) -> Constant {
        match self {
            Literal::Num(n) => Constant::Num(n),
            Literal::Str(s) => Constant::Str(GcRef::new(s)),
            Literal::Bool(b) => Constant::Bool(b),
            Literal::Sym(s) => Constant::Sym(s),
            Literal::Unit => Constant::Nil,
        }
    }
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
    pub column: usize,
}

impl Expr {
    pub fn new(kind: ExprKind, line: usize, column: usize) -> Self {
        Expr { kind, line, column }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr {
            kind: ExprKind::Lit(Literal::Unit),
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: usize,
    pub column: usize,
}

impl Stmt {
    pub fn new(kind: StmtKind, line: usize, column: usize) -> Self {
        Stmt { kind, line, column }
    }
}

#[derive(Debug)]
pub enum StmtKind {
    Def { bind: VarDecl, value: Expr },
    Open(String),
}
