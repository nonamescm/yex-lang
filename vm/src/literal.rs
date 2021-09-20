use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Num(f64),
    Str(String),
    Nil,
}

impl Add for Literal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x + y),
            (Self::Str(x), Self::Str(y)) => Self::Str(x + y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Add for &Literal {
    type Output = Literal;

    fn add(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Num(x + y),
            (Str(x), Str(y)) => Str(x.to_string() + y),
            (Nil, _) | (_, Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Sub for Literal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x - y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Sub for &Literal {
    type Output = Literal;

    fn sub(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Num(x - y),
            (Nil, _) | (_, Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Mul for Literal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x * y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Mul for &Literal {
    type Output = Literal;

    fn mul(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Num(x * y),
            (Nil, _) | (_, Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Div for Literal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x / y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
            _ => todo!()
        }
    }
}

impl Div for &Literal {
    type Output = Literal;

    fn div(self, rhs: Self) -> Self::Output {
        use Literal::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Num(x / y),
            (Nil, _) | (_, Nil) => todo!(),
            _ => todo!()
        }
    }
}
