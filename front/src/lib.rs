mod compiler;
mod error;
mod lexer;
mod tests;
mod tokens;

use compiler::Compiler;
use lexer::Lexer;

pub fn compile<T: Into<String>>(str: T) -> Vec<vm::Instruction> {
    Compiler::compile(Lexer::new(str)).unwrap()
}
