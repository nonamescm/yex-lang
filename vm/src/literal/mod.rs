use std::{
    cmp::Ordering,
    ffi::c_void,
    mem,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub},
};
pub mod symbol;
use crate::{
    error::InterpretResult, gc::GcRef, list::List, stack::StackVec, table::Table, Bytecode, Either,
    VirtualMachine,
};
use symbol::Symbol;

pub type NativeFun = fn(*mut VirtualMachine, Vec<Constant>) -> InterpretResult<Constant>;
pub type FunBody = GcRef<Either<Bytecode, NativeFun>>;
pub type FunArgs = StackVec<Constant, 8>;

pub type FFINoArgFunction = unsafe extern "C" fn() -> *mut c_void;
pub type FFIFunction = unsafe extern "C" fn(usize, *mut u8) -> *mut c_void;

pub fn nil() -> Constant {
    Constant::Nil
}

#[derive(Debug, PartialEq)]
/// Yex function struct
pub struct Fun {
    /// The number of argument the function receives
    pub arity: usize,
    /// The function body
    pub body: FunBody,
    /// The arguments that where already passed to the function
    pub args: FunArgs,
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
    /// External Function
    ExternalFunctionNoArg(FFINoArgFunction),
    /// External Function With Arguments
    ExternalFunction(FFIFunction),
    /// null
    Nil,
}

impl Clone for Constant {
    fn clone(&self) -> Self {
        use Constant::*;

        match self {
            List(xs) => List(GcRef::clone(xs)),
            Table(ts) => Table(GcRef::clone(ts)),
            Str(str) => Str(GcRef::clone(str)),
            Fun(f) => Fun(GcRef::clone(f)),
            Bool(b) => Bool(*b),
            Num(n) => Num(*n),
            Sym(s) => Sym(*s),
            ExternalFunction(f) => ExternalFunction(*f),
            ExternalFunctionNoArg(f) => ExternalFunctionNoArg(*f),
            Nil => Nil,
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
            Constant::ExternalFunction(f) => mem::size_of_val(f),
            Constant::ExternalFunctionNoArg(f) => mem::size_of_val(f),
            Constant::Bool(_) => std::mem::size_of::<bool>(),
            Constant::Nil => 4,
        }
    }

    /// Compares the left and the right value
    pub fn ord_cmp(&self, rhs: &Self) -> InterpretResult<Ordering> {
        let (left, right) = match (self, rhs) {
            (Self::Num(left), Self::Num(right)) => (left, right),
            (left, right) => return crate::panic!("Can't compare `{}` and `{}`", left, right),
        };

        match left.partial_cmp(right) {
            Some(ord) => Ok(ord),
            None => panic!("Error applying cmp"),
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
            ExternalFunction(_) => "<extern fun<?>>".to_string(),
            ExternalFunctionNoArg(_) => "<extern fun<0>".to_string(),
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

impl Sub for Constant {
    type Output = ConstantErr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x - y)),
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

impl Div for Constant {
    type Output = ConstantErr;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x / y)),
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
