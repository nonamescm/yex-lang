use std::fmt;

#[derive(Debug)]
pub struct InterpretError {
    pub file: Option<String>,
    pub err: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "[{}:{}:{}] {}",
            self.file.as_ref().unwrap_or(&String::from("<unknown>")),
            self.line,
            self.column,
            self.err,
        )?;
        if let Some(ref file) = self.file {
            use std::fs;

            if let Ok(content) = fs::read_to_string(file) {
                writeln!(f, "at")?;
                let lines = content.split('\n');
                let lines = lines.collect::<Vec<_>>();
                let current_line = lines.get(self.line - 1).unwrap_or(&"<CAN'T READ FILE>");
                writeln!(f, "  {}: {}", self.line, current_line)?;
                writeln!(f, "{}↑", String::from(" ").repeat(self.column - 1))?;
            }
        }
        Ok(())
    }
}
pub type InterpretResult<T> = Result<T, InterpretError>;
