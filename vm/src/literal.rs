use std::ops::{Add, Div, Mul, Neg, Sub, BitAnd, BitOr, BitXor, Shr, Shl};

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Num(f64),
    Str(String),
    Nil,
}

type LiteralErr = Result<Literal, String>;

macro_rules! err {
    ($($tt: tt),+) => {
        Err(format!($($tt),+))
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Literal::*;
        let tk = match self {
            Nil => "nil".to_string(),
            Str(s) => "\"".to_owned() + s + "\"",
            Num(n) => n.to_string(),
        };
        write!(f, "{}", tk)
    }
}

impl Add for Literal {
    type Output = Result<Self, String>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x + y)),
            (Self::Str(x), Self::Str(y)) => Ok(Self::Str(x + &y)),
            (s, r) => err!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Add for &Literal {
    type Output = LiteralErr;

    fn add(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x + y)),
            (Str(x), Str(y)) => Ok(Str(x.to_string() + y)),
            (s, r) => err!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Sub for Literal {
    type Output = LiteralErr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x - y)),
            (s, r) => err!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Sub for &Literal {
    type Output = LiteralErr;

    fn sub(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x - y)),
            (s, r) => err!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Mul for Literal {
    type Output = LiteralErr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x * y)),
            (s, r) => err!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Mul for &Literal {
    type Output = LiteralErr;

    fn mul(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x * y)),
            (s, r) => err!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Div for Literal {
    type Output = Result<Self, String>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x / y)),
            (s, r) => err!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Div for &Literal {
    type Output = LiteralErr;

    fn div(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x / y)),
            (s, r) => err!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Neg for Literal {
    type Output = LiteralErr;

    fn neg(self) -> Self::Output {
        match self {
            Self::Num(n) => Ok(Self::Num(-n)),
            s => err!("Can't apply unary `-` operator on {}", s),
        }
    }
}

impl Neg for &Literal {
    type Output = LiteralErr;

    fn neg(self) -> Self::Output {
        use Literal::*;

        match self {
            Num(n) => Ok(Num(-n)),
            s => err!("Can't apply unary `-` operator on {}", s),
        }
    }
}

impl BitXor for Literal {
    type Output = LiteralErr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) ^ (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise xor between `{}` and `{}`", x, y)
        }
    }
}

impl BitXor for &Literal {
    type Output = LiteralErr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) ^ (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise xor between `{}` and `{}`", x, y)
        }
    }
}


impl BitAnd for Literal {
    type Output = LiteralErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise and between `{}` and `{}`", x, y)
        }
    }
}

impl BitAnd for &Literal {
    type Output = LiteralErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise and between `{}` and `{}`", x, y)
        }
    }
}


impl BitOr for Literal {
    type Output = LiteralErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise or between `{}` and `{}`", x, y)
        }
    }
}

impl BitOr for &Literal {
    type Output = LiteralErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise or between `{}` and `{}`", x, y)
        }
    }
}

impl Shr for Literal {
    type Output = LiteralErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise or between `{}` and `{}`", x, y)
        }
    }
}

impl Shr for &Literal {
    type Output = LiteralErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise or between `{}` and `{}`", x, y)
        }
    }
}

impl Shl for Literal {
    type Output = LiteralErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise or between `{}` and `{}`", x, y)
        }
    }
}

impl Shl for &Literal {
    type Output = LiteralErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => {
                Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64))
            }
            (x, y) => err!("Can't apply bitwise or between `{}` and `{}`", x, y)
        }
    }
}
