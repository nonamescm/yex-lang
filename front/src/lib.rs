#![deny(missing_docs)]
//! Compiler for the yex language
mod compiler;
mod error;
mod lexer;
mod tests;
mod tokens;

use compiler::Compiler;
use lexer::Lexer;

/// Compiles a given string into yex bytecode
pub fn compile<T: Into<String>>(str: T) -> Result<vm::Bytecode, error::ParseError> {
    Compiler::compile(Lexer::new(str))
}
