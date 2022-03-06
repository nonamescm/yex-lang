use std::{
    cmp::Ordering,
    ffi::c_void,
    mem,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub},
};
pub mod symbol;
use crate::{
    error::InterpretResult, gc::GcRef, list::List, stack::StackVec, Bytecode, Either,
    VirtualMachine,
};
use symbol::Symbol;

pub type NativeFun = fn(*mut VirtualMachine, Vec<Value>) -> InterpretResult<Value>;
pub type FunBody = GcRef<Either<Bytecode, NativeFun>>;
pub type FunArgs = StackVec<Value, 8>;

pub type FFINoArgFunction = unsafe extern "C" fn() -> *mut c_void;
pub type FFIFunction = unsafe extern "C" fn(usize, *mut u8) -> *mut c_void;

#[derive(PartialEq, Clone)]
/// Yex function struct
pub struct Fun {
    /// The number of argument the function receives
    pub arity: usize,
    /// The function body
    pub body: FunBody,
    /// The function Arguments
    pub args: FunArgs,
}

impl Fun {
    /// Apply the function to the given arguments
    pub fn apply(&self, args: FunArgs) -> Self {
        Fun {
            arity: self.arity + self.args.len() - args.len(),
            body: self.body.clone(),
            args,
        }
    }
}

impl std::fmt::Debug for Fun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fun {{ arity: {}, body: {:?} }}", self.arity, self.body)
    }
}

impl std::fmt::Display for Fun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun({})>", self.arity)
    }
}

pub fn nil() -> Value {
    Value::Nil
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

/// Immediate values that can be consumed
#[derive(Debug, PartialEq)]
pub enum Value {
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
    List(List),
    /// External Function
    ExternalFunctionNoArg(FFINoArgFunction),
    /// External Function With Arguments
    ExternalFunction(FFIFunction),
    /// null
    Nil,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        use Value::*;

        match self {
            List(xs) => List(xs.clone()),
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

impl Value {
    /// checks if the constant is `nil`
    pub fn is_nil(&self) -> bool {
        self == &Self::Nil
    }

    /// Returns the size of `self`
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Value::List(xs) => xs.len(),
            Value::Num(_) => std::mem::size_of::<f64>(),
            Value::Sym(_) => std::mem::size_of::<Symbol>(),
            Value::Str(s) => s.len(),
            Value::Fun(f) => mem::size_of_val(&f),
            Value::ExternalFunction(f) => mem::size_of_val(f),
            Value::ExternalFunctionNoArg(f) => mem::size_of_val(f),
            Value::Bool(_) => std::mem::size_of::<bool>(),
            Value::Nil => 4,
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

    /// Convert the constant to a boolean
    pub fn to_bool(&self) -> bool {
        use Value::*;

        match self {
            Bool(true) => true,
            Bool(false) => false,
            Sym(_) => true,
            Str(s) if s.is_empty() => false,
            Str(_) => true,
            Num(n) if *n == 0.0 => false,
            Num(_) => true,
            Nil => false,
            List(xs) => !xs.is_empty(),
            Fun(_) => true,
            ExternalFunction(_) => true,
            ExternalFunctionNoArg(_) => true,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

type ConstantErr = InterpretResult<Value>;

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

impl From<Value> for bool {
    fn from(o: Value) -> Self {
        o.to_bool()
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;
        let tk = match self {
            Fun(f) => {
                format!("<fun({})>", f.arity)
            }
            ExternalFunction(_) => "<extern fun<?>>".to_string(),
            ExternalFunctionNoArg(_) => "<extern fun<0>".to_string(),
            Nil => "nil".to_string(),
            List(xs) => format!("{}", *xs),
            Str(s) => "\"".to_owned() + s + "\"",
            Sym(s) => format!("{}", s),
            Num(n) => n.to_string(),
            Bool(b) => b.to_string(),
        };
        write!(f, "{}", tk)
    }
}

impl Add for Value {
    type Output = ConstantErr;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x + y)),
            (Self::Str(x), Self::Str(y)) => Ok(Self::Str(GcRef::new(x.to_string() + &y))),
            (s, r) => panic!("Can't apply `+` operator between {} and {}", s, r),
        }
    }
}

impl Sub for Value {
    type Output = ConstantErr;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x - y)),
            (s, r) => panic!("Can't apply `-` operator between {} and {}", s, r),
        }
    }
}

impl Mul for Value {
    type Output = ConstantErr;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x * y)),
            (s, r) => panic!("Can't apply `*` operator between {} and {}", s, r),
        }
    }
}

impl Div for Value {
    type Output = ConstantErr;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, &rhs) {
            (Self::Num(x), Self::Num(y)) => Ok(Self::Num(x / y)),
            (s, r) => panic!("Can't apply `/` operator between {} and {}", s, r),
        }
    }
}

impl Neg for Value {
    type Output = ConstantErr;

    fn neg(self) -> Self::Output {
        match self {
            Self::Num(n) => Ok(Self::Num(-n)),
            s => panic!("Can't apply unary `-` operator on {}", s),
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        Self::Bool(!self.to_bool())
    }
}

impl BitXor for Value {
    type Output = ConstantErr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use Value::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) ^ (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `^` between {} and {}", x, y),
        }
    }
}

impl BitAnd for Value {
    type Output = ConstantErr;

    fn bitand(self, rhs: Self) -> Self::Output {
        use Value::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) & (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `&` between {} and {}", x, y),
        }
    }
}

impl BitOr for Value {
    type Output = ConstantErr;

    fn bitor(self, rhs: Self) -> Self::Output {
        use Value::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) | (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `|` between {} and {}", x, y),
        }
    }
}

impl Shr for Value {
    type Output = ConstantErr;

    fn shr(self, rhs: Self) -> Self::Output {
        use Value::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) >> (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `>>` between {} and {}", x, y),
        }
    }
}

impl Shl for Value {
    type Output = ConstantErr;

    fn shl(self, rhs: Self) -> Self::Output {
        use Value::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(((x.round() as i64) << (y.round() as i64)) as f64)),
            (x, y) => panic!("Can't apply bitwise `<<` between {} and {}", x, y),
        }
    }
}

impl Rem for Value {
    type Output = ConstantErr;

    fn rem(self, rhs: Self) -> Self::Output {
        use Value::*;

        match (self, rhs) {
            (Num(x), Num(y)) => Ok(Num(x % y)),
            (x, y) => panic!("Can't apply `%` between {} and {}", x, y),
        }
    }
}
