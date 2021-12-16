use std::{
    mem,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub},
};
pub mod symbol;
use crate::{
    error::InterpretResult, gc::GcRef, list::List, table::Table, Bytecode, Either, VirtualMachine,
};
use symbol::Symbol;
pub type NativeFun = fn(*mut VirtualMachine, Vec<Constant>) -> Constant;
pub type FunBody = GcRef<Either<Bytecode, NativeFun>>;

pub fn nil() -> Constant {
    Constant::Nil
}

pub fn ok() -> Constant {
    Constant::Sym(crate::Symbol::new("ok"))
}

pub fn err() -> Constant {
    Constant::Sym(crate::Symbol::new("err"))
}

#[derive(Debug, PartialEq)]
/// Yex function struct
pub struct Fun {
    /// The number of argument the function receives
    pub arity: usize,
    /// The function body
    pub body: FunBody,
    /// The arguments that where already passed to the function
    pub args: Vec<Constant>,
}

/// Immediate values that can be consumed
#[derive(Debug, PartialEq)]
pub enum Constant {
    /// float-precision numbers
    Num(f64),
    /// Strings
    Str(GcRef<String>),
    /// erlang-like atoms
    Sym(Symbol),
    /// Booleans
    Bool(bool),
    /// Functions
    Fun(GcRef<Fun>),
    /// Yex lists
    List(GcRef<List>),
    /// Yex Tables
    Table(GcRef<Table>),
    /// null
    Nil,
}

impl Clone for Constant {
    fn clone(&self) -> Self {
        use Constant::*;

        match self {
            Num(n) => Num(*n),
            Sym(s) => Sym(*s),
            Bool(b) => Bool(*b),
            Nil => Nil,
            Str(ref_s) => Str(GcRef::clone(ref_s)),
            List(xs) => List(GcRef::clone(xs)),
            Table(ts) => Table(GcRef::clone(ts)),
            Fun(f) => Fun(GcRef::clone(f)),
        }
    }
}

impl Constant {
    /// checks if the constant is `nil`
    pub fn is_nil(&self) -> bool {
        self == &Self::Nil
    }

    /// Returns the size of `self`
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Constant::List(xs) => xs.len(),
            Constant::Num(_) => std::mem::size_of::<f64>(),
            Constant::Sym(_) => std::mem::size_of::<Symbol>(),
            Constant::Str(s) => s.len(),
            Constant::Table(ts) => ts.len(),
            Constant::Fun(f) => {
                mem::size_of_val(&f.arity) + mem::size_of_val(&f.body) + mem::size_of_val(&f.args)
            }
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

type ConstantErr = InterpretResult<Constant>;

macro_rules! panic {
    ($($tt:tt)+) => {
        unsafe {
            let msg = format!($($tt)+);
            Err($crate::error::InterpretError {
                line: $crate::LINE,
                column: $crate::COLUMN,
                err: msg
            })
        }
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
            Fun(f) => {
                format!("<fun({})>", f.arity)
            }
            Nil => "nil".to_string(),
            List(xs) => format!("{}", xs.get()),
            Table(ts) => format!("{}", ts.get()),
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
            (Self::Str(x), Self::Str(y)) => Ok(Self::Str(GcRef::new(x.get().to_string() + &y))),
            (s, r) => panic!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Add for &Constant {
    type Output = ConstantErr;

    fn add(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x + y)),
            (Str(x), Str(y)) => Ok(Str(GcRef::new(x.get().to_string() + y))),
            (s, r) => panic!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Sub for Constant {
    type Output = ConstantErr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x - y)),
            (s, r) => panic!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Sub for &Constant {
    type Output = ConstantErr;

    fn sub(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x - y)),
            (s, r) => panic!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Mul for Constant {
    type Output = ConstantErr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x * y)),
            (s, r) => panic!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Mul for &Constant {
    type Output = ConstantErr;

    fn mul(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x * y)),
            (s, r) => panic!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Div for Constant {
    type Output = ConstantErr;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x / y)),
            (s, r) => panic!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Div for &Constant {
    type Output = ConstantErr;

    fn div(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x / y)),
            (s, r) => panic!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Neg for Constant {
    type Output = ConstantErr;

    fn neg(self) -> Self::Output {
        match self {
            Self::Num(n) => Ok(Self::Num(-n)),
            s => panic!("Can't apply unary `-` operator on {}", s),
        }
    }
}

impl Neg for &Constant {
    type Output = ConstantErr;

    fn neg(self) -> Self::Output {
        use Constant::*;

        match self {
            Num(n) => Ok(Num(-n)),
            s => panic!("Can't apply unary `-` operator on {}", s),
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
    type Output = Constant;

    fn not(self) -> Self::Output {
        use Constant::*;

        match self {
            Bool(true) => Constant::Bool(false),
            Bool(false) => Constant::Bool(true),
            Sym(_) => Constant::Bool(false),
            Str(s) if s.is_empty() => Constant::Bool(true),
            Str(_) => Constant::Bool(false),
            Num(n) if *n == 0.0 => Constant::Bool(true),
            Num(_) => Constant::Bool(false),
            Nil => Constant::Bool(true),
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
            (x, y) => panic!("Can't apply bitwise `^` between {} and {}", x, y),
        }
    }
}

impl BitXor for &Constant {
    type Output = ConstantErr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) ^ (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `^` between {} and {}", x, y),
        }
    }
}

impl BitAnd for Constant {
    type Output = ConstantErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `&` between {} and {}", x, y),
        }
    }
}

impl BitAnd for &Constant {
    type Output = ConstantErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `&` between {} and {}", x, y),
        }
    }
}

impl BitOr for Constant {
    type Output = ConstantErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `|` between {} and {}", x, y),
        }
    }
}

impl BitOr for &Constant {
    type Output = ConstantErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `|` between {} and {}", x, y),
        }
    }
}

impl Shr for Constant {
    type Output = ConstantErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `>>` between {} and {}", x, y),
        }
    }
}

impl Shr for &Constant {
    type Output = ConstantErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `>>` between {} and {}", x, y),
        }
    }
}

impl Shl for Constant {
    type Output = ConstantErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `<<` between {} and {}", x, y),
        }
    }
}

impl Shl for &Constant {
    type Output = ConstantErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Constant::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `<<` between {} and {}", x, y),
        }
    }
}
