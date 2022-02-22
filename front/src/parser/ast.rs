use std::fmt::Display;

use vm::Symbol;

use crate::tokens::TokenType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseType {
    pub ty: Symbol,
    pub args: Option<Vec<Type>>,
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)?;

        if let Some(ref args) = self.args {
            write!(f, "[")?;

            for (idx, arg) in args.iter().enumerate() {
                if idx == args.len() - 1 {
                    write!(f, "{}", arg)?;
                } else {
                    write!(f, "{}, ", arg)?;
                }
            }

            write!(f, "]")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnType(pub Vec<Type>);

impl Display for FnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, ty) in self.0.iter().enumerate() {
            if idx == self.0.len() - 1 {
                write!(f, "{}", ty)?;
            } else {
                write!(f, "{} -> ", ty)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Base(BaseType),
    Fn(FnType),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base(ty) => write!(f, "{}", ty),
            Self::Fn(ty) => write!(f, "{}", ty),
        }
    }
}

impl Type {
    pub fn num() -> Self {
        Self::Base(BaseType {
            ty: Symbol::new("Num"),
            args: None,
        })
    }

    pub fn string() -> Self {
        Self::Base(BaseType {
            ty: Symbol::new("String"),
            args: None,
        })
    }

    pub fn bool() -> Self {
        Self::Base(BaseType {
            ty: Symbol::new("Bool"),
            args: None,
        })
    }

    pub fn sym() -> Self {
        Self::Base(BaseType {
            ty: Symbol::new("Sym"),
            args: None,
        })
    }

    pub fn unit() -> Self {
        Self::Base(BaseType {
            ty: Symbol::new("Unit"),
            args: None,
        })
    }

    pub fn list(ty: Self) -> Self {
        Self::Base(BaseType {
            ty: Symbol::new("List"),
            args: Some(vec![ty]),
        })
    }
}

#[derive(Debug)]
pub struct VarDecl {
    pub name: Symbol,
    pub ty: Type,
}

impl VarDecl {
    pub fn new(name: Symbol, ty: Type) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
pub enum CmpOp {
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

impl TryFrom<TokenType> for CmpOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::Less => Ok(CmpOp::Less),
            TokenType::LessEq => Ok(CmpOp::LessEq),
            TokenType::Greater => Ok(CmpOp::Greater),
            TokenType::GreaterEq => Ok(CmpOp::GreaterEq),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum EqOp {
    Eq,
    Ne,
}

impl TryFrom<TokenType> for EqOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::Eq => Ok(EqOp::Eq),
            TokenType::Ne => Ok(EqOp::Ne),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl TryFrom<TokenType> for MathOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::Add => Ok(MathOp::Add),
            TokenType::Sub => Ok(MathOp::Sub),
            TokenType::Mul => Ok(MathOp::Mul),
            TokenType::Div => Ok(MathOp::Div),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum BitOp {
    And,
    Or,
    Xor,
    Lsh,
    Rsh,
}

impl TryFrom<TokenType> for BitOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::BitAnd => Ok(BitOp::And),
            TokenType::BitOr => Ok(BitOp::Or),
            TokenType::BitXor => Ok(BitOp::Xor),
            TokenType::Shr => Ok(BitOp::Lsh),
            TokenType::Shl => Ok(BitOp::Rsh),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum LogicalOp {
    And,
    Or,
}

impl TryFrom<TokenType> for LogicalOp {
    type Error = ();

    fn try_from(t: TokenType) -> Result<Self, Self::Error> {
        match t {
            TokenType::And => Ok(LogicalOp::And),
            TokenType::Or => Ok(LogicalOp::Or),
            _ => Err(()),
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
        ty: FnType,         // the type of the entire function, like Int -> Int -> Int
        ret: Type,          // the return type of the function
        body: Box<Expr>,    // the function body
    },
    App {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    Var(Symbol),
    Lit(Literal),
    List(Vec<Expr>),

    Bitwise {
        left: Box<Expr>,
        op: BitOp,
        right: Box<Expr>,
    },
    Math {
        left: Box<Expr>,
        op: MathOp,
        right: Box<Expr>,
    },
    Cmp {
        left: Box<Expr>,
        op: CmpOp,
        right: Box<Expr>,
    },
    Eq {
        left: Box<Expr>,
        op: EqOp,
        right: Box<Expr>,
    },
    Logic {
        left: Box<Expr>,
        op: LogicalOp,
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

    UnOp(TokenType, Box<Expr>),
}

#[derive(Debug)]
pub enum Literal {
    Num(f64),
    Str(String),
    Bool(bool),
    Sym(Symbol),
    Unit,
}

impl Literal {
    pub fn type_of(&self) -> Type {
        match self {
            Self::Num(..) => Type::num(),
            Self::Str(..) => Type::string(),
            Self::Bool(..) => Type::bool(),
            Self::Sym(..) => Type::sym(),
            Self::Unit => Type::unit(),
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
