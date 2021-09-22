#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    line: usize,
    column: usize,
    message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] {}", self.line, self.column, self.message)
    }
}

impl ParseError {
    pub fn throw<T>(line: usize, column: usize, message: String) -> Result<T, Self> {
        Err(Self {
            line,
            column,
            message,
        })
    }
}
