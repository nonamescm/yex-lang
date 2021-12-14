use std::{
    mem,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub},
};
pub mod symbol;
use crate::{list::List, Bytecode};
use symbol::Symbol;
use either::Either;
pub type NativeFun = fn(Vec<Constant>) -> Constant;

/// Immediate values that can be consumed
#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    /// float-precision numbers
    Num(f64),
    /// Strings
    Str(String),
    /// erlang-like atoms
    Sym(Symbol),
    /// Booleans
    Bool(bool),
    /// Functions
    Fun {
        /// The number of argument the function receives
        arity: usize,
        /// The function body
        body: Bytecode,
    },
    /// Partially applied function
    PartialFun {
        /// The number of argument the function receives
        arity: usize,
        /// The function body
        body: Either<Bytecode, NativeFun>,
        /// The arguments that where already passed to the function
        args: Vec<Constant>,
    },
    /// A native rust function
    NativeFun {
        /// The function's arity
        arity: usize,
        /// The function pointer
        fp: NativeFun,
    },

    /// Yex lists
    List(List),
    /// null
    Nil,
}

impl Constant {
    /// checks if the constant is `nil`
    pub fn is_nil(&self) -> bool {
        self == &Self::Nil
    }

    /// Returns the size of `self`
    pub fn len(&self) -> usize {
        match self {
            Constant::List(xs) => xs.len(),
            Constant::Num(_) => std::mem::size_of::<f64>(),
            Constant::Sym(_) => std::mem::size_of::<Symbol>(),
            Constant::Str(s) => s.len(),
            Constant::Fun { arity, body } => mem::size_of_val(&body) + mem::size_of_val(&arity),
            Constant::PartialFun { arity, body, args } => {
                mem::size_of_val(&body) + mem::size_of_val(&arity) + mem::size_of_val(&args)
            }
            Constant::NativeFun { arity, fp } => mem::size_of_val(&arity) + mem::size_of_val(&fp),
            Constant::Bool(_) => std::mem::size_of::<bool>(),
            Constant::Nil => 4,
        }
    }
}

impl Default for Constant {
    fn default() -> Self {
        Self::Nil
    }
}

type ConstantErr = Result<Constant, String>;

macro_rules! err {
    ($($tt: tt),+) => {
        Err(format!($($tt),+))
    }
}

impl From<Constant> for bool {
    fn from(o: Constant) -> Self {
        match !o {
            Constant::Bool(true) => false,
            Constant::Bool(false) => true,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Constant::*;
        let tk = match self {
            Fun { arity, .. } | PartialFun { arity, .. } | NativeFun { arity, .. } => {
                format!("<fun({})>", arity)
            }
            Nil => "nil".to_string(),
            List(xs) => format!("{}", xs),
            Str(s) => "\"".to_owned() + s + "\"",
            Sym(s) => format!("{}", s),
            Num(n) => n.to_string(),
            Bool(b) => b.to_string(),
        };
        write!(f, "{}", tk)
    }
}

impl Add for Constant {
    type Output = ConstantErr;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x + y)),
            (Self::Str(x), Self::Str(y)) => Ok(Self::Str(x + &y)),
            (s, r) => err!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Add for &Constant {
    type Output = ConstantErr;

    fn add(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x + y)),
            (Str(x), Str(y)) => Ok(Str(x.to_string() + y)),
            (s, r) => err!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Sub for Constant {
    type Output = ConstantErr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x - y)),
            (s, r) => err!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Sub for &Constant {
    type Output = ConstantErr;

    fn sub(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x - y)),
            (s, r) => err!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Mul for Constant {
    type Output = ConstantErr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x * y)),
            (s, r) => err!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Mul for &Constant {
    type Output = ConstantErr;

    fn mul(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x * y)),
            (s, r) => err!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Div for Constant {
    type Output = ConstantErr;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x / y)),
            (s, r) => err!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Div for &Constant {
    type Output = ConstantErr;

    fn div(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x / y)),
            (s, r) => err!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Neg for Constant {
    type Output = ConstantErr;

    fn neg(self) -> Self::Output {
        match self {
            Self::Num(n) => Ok(Self::Num(-n)),
            s => err!("Can't apply unary `-` operator on {}", s),
        }
    }
}

impl Neg for &Constant {
    type Output = ConstantErr;

    fn neg(self) -> Self::Output {
        use Constant::*;

        match self {
            Num(n) => Ok(Num(-n)),
            s => err!("Can't apply unary `-` operator on {}", s),
        }
    }
}

impl Not for Constant {
    type Output = Constant;

    fn not(self) -> Self::Output {
        use Constant::*;

        match self {
            Bool(true) => Constant::Bool(false),
            Bool(false) => Constant::Bool(true),
            Sym(_) => Constant::Bool(false),
            Str(s) if s.is_empty() => Constant::Bool(true),
            Str(_) => Constant::Bool(false),
            Num(n) if n == 0.0 => Constant::Bool(true),
            Num(_) => Constant::Bool(false),
            Nil => Constant::Bool(true),
            _ => unreachable!(),
        }
    }
}

impl Not for &Constant {
    type Output = ConstantErr;

    fn not(self) -> Self::Output {
        use Constant::*;

        match self {
            Bool(true) => Ok(Constant::Bool(false)),
            Bool(false) => Ok(Constant::Bool(true)),
            Sym(_) => Ok(Constant::Bool(false)),
            Str(s) if s.is_empty() => Ok(Constant::Bool(true)),
            Str(_) => Ok(Constant::Bool(false)),
            Num(n) if *n == 0.0 => Ok(Constant::Bool(true)),
            Num(_) => Ok(Constant::Bool(false)),
            Nil => Ok(Constant::Bool(true)),
            _ => unreachable!(),
        }
    }
}
impl BitXor for Constant {
    type Output = ConstantErr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) ^ (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `^` between {} and {}", x, y),
        }
    }
}

impl BitXor for &Constant {
    type Output = ConstantErr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) ^ (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `^` between {} and {}", x, y),
        }
    }
}

impl BitAnd for Constant {
    type Output = ConstantErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `&` between {} and {}", x, y),
        }
    }
}

impl BitAnd for &Constant {
    type Output = ConstantErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `&` between {} and {}", x, y),
        }
    }
}

impl BitOr for Constant {
    type Output = ConstantErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `|` between {} and {}", x, y),
        }
    }
}

impl BitOr for &Constant {
    type Output = ConstantErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `|` between {} and {}", x, y),
        }
    }
}

impl Shr for Constant {
    type Output = ConstantErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `>>` between {} and {}", x, y),
        }
    }
}

impl Shr for &Constant {
    type Output = ConstantErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `>>` between {} and {}", x, y),
        }
    }
}

impl Shl for Constant {
    type Output = ConstantErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `<<` between {} and {}", x, y),
        }
    }
}

impl Shl for &Constant {
    type Output = ConstantErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64)),
            (x, y) => err!("Can't apply bitwise `<<` between {} and {}", x, y),
        }
    }
}
