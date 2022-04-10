use std::{fmt, io};

use crate::{Symbol, raise_err};

#[derive(Debug)]
pub struct InterpretError {
    pub err: Symbol,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}] {}", self.line, self.column, self.err)
    }
}

impl From<io::Error> for InterpretError {
    fn from(_: io::Error) -> Self {
        raise_err!(IOError)
    }
}

pub type InterpretResult<T> = Result<T, InterpretError>;
