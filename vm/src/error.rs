use std::fmt;

pub struct InterpretError {
    pub err: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}] {}", self.line, self.column, self.err)
    }
}
pub type InterpretResult<T> = Result<T, InterpretError>;

