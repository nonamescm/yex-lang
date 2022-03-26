use std::{
    cmp::Ordering,
    mem,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub},
};

pub mod fun;
pub mod instance;
pub mod list;
pub mod symbol;
pub mod yextype;

use crate::{error::InterpretResult, gc::GcRef};

use fun::Fun;
use instance::Instance;
use list::List;
use symbol::Symbol;
use yextype::YexType;

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
    /// Yex user-defined types
    Type(GcRef<YexType>),
    /// Yex instances
    Instance(GcRef<Instance>),
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
            Type(t) => Type(t.clone()),
            Instance(i) => Instance(i.clone()),
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
            Value::Num(_) => mem::size_of::<f64>(),
            Value::Sym(_) => mem::size_of::<Symbol>(),
            Value::Str(s) => s.len(),
            Value::Fun(f) => mem::size_of_val(&f),
            Value::Bool(_) => mem::size_of::<bool>(),
            Value::Type(t) => mem::size_of_val(&t),
            Value::Instance(i) => mem::size_of_val(&i),
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
            Bool(b) => *b,
            Sym(_) => true,
            Str(s) if s.is_empty() => false,
            Str(_) => true,
            Num(n) if *n == 0.0 => false,
            Num(_) => true,
            Nil => false,
            List(xs) => !xs.is_empty(),
            Fun(_) => true,
            Value::Type(_) => true,
            Value::Instance(_) => true,
        }
    }

    /// returns the type of the value
    pub fn type_of(&self) -> GcRef<YexType> {
        use Value::*;

        match self {
            Type(t) => return t.clone(),
            Instance(i) => return i.ty.clone(),
            _ => {}
        };

        let ty = match self {
            List(_) => YexType::list(),
            Fun(_) => YexType::fun(),
            Num(_) => YexType::num(),
            Str(_) => YexType::str(),
            Bool(_) => YexType::bool(),
            Nil => YexType::nil(),
            Sym(_) => YexType::sym(),
            Type(_) | Instance(_) => unreachable!(),
        };

        GcRef::new(ty)
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
            Fun(f) => format!("<fun({})>", f.arity),
            Nil => "nil".to_string(),
            List(xs) => format!("{}", *xs),
            Str(s) => "\"".to_owned() + s + "\"",
            Sym(s) => format!("{}", s),
            Num(n) => n.to_string(),
            Type(t) => format!("<type({})>", t.name),
            Instance(i) => format!("<instance({})>", i.ty.name),
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
