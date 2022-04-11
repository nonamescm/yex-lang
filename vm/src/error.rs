use std::{fmt, io};

use crate::{Symbol, raise_err};

#[derive(Debug)]
pub struct InterpretError {
    pub msg: String,
    pub err: Symbol,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}] @{}\n  {}", self.line, self.column, self.err, self.msg)
    }
}

impl From<io::Error> for InterpretError {
    fn from(_: io::Error) -> Self {
        raise_err!(IOError, "Internal IO error")
    }
}

pub type InterpretResult<T> = Result<T, InterpretError>;
