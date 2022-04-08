use std::{fmt, io};

use crate::{Symbol, COLUMN, LINE};

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
    fn from(err: io::Error) -> Self {
        InterpretError {
            err: err.kind().to_string().into(),
            line: unsafe { LINE },
            column: unsafe { COLUMN },
        }
    }
}

pub type InterpretResult<T> = Result<T, InterpretError>;
