use std::{
    hint::unreachable_unchecked,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub},
};
pub mod symbol;
use symbol::Symbol;

/// Immediate values that can be consumed
#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    /// float-precision numbers
    Num(f64),
    /// Strings
    Str(String),
    /// erlang-like atoms
    Sym(Symbol),
    /// Variables
    Val(Symbol),
    /// Booleans
    Bool(bool),
    /// null
    Nil,
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
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Constant::*;
        let tk = match self {
            Nil => "nil".to_string(),
            Str(s) => "\"".to_owned() + s + "\"",
            Sym(s) | Val(s) => format!("{}", s),
            Num(n) => n.to_string(),
            Bool(b) => b.to_string(),
        };
        write!(f, "{}", tk)
    }
}

impl Add for Constant {
    type Output = Result<Self, String>;

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
    type Output = Result<Self, String>;

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
