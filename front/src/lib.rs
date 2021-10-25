mod compiler;
mod error;
mod lexer;
mod tests;
mod tokens;

use compiler::Compiler;
use lexer::Lexer;

pub fn compile<T: Into<String>>(str: T) -> Result<vm::Bytecode, error::ParseError> {
    Compiler::compile(Lexer::new(str))
}
