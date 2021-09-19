use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Literal {
    Num(i64),
    Nil,
}

impl Add for Literal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x + y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
        }
    }
}

impl Sub for Literal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x - y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
        }
    }
}

impl Mul for Literal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x * y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
        }
    }
}

impl Div for Literal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Self::Num(x / y),
            (Self::Nil, _) | (_, Self::Nil) => todo!(),
        }
    }
}

