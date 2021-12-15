#![deny(missing_docs)]
//! Compiler for the yex language
#[doc(hidden)]
pub static YEX_EXTENSIONS: [&str; 2] = [".yex", ".ml"];
mod compiler;
mod error;
mod lexer;
mod tests;
mod tokens;

pub use error::ParseError;

use compiler::Compiler;
use lexer::Lexer;

/// Compiles a given string into yex bytecode
pub fn compile<T: Into<String>>(
    str: T,
) -> Result<(vm::Bytecode, Vec<vm::Constant>), error::ParseError> {
    Compiler::compile(Lexer::new(str))
}

/// Compiles a given expression into yex bytecode
pub fn compile_expr<T: Into<String>>(
    str: T,
) -> Result<(vm::Bytecode, Vec<vm::Constant>), error::ParseError> {
    Compiler::compile_expr(Lexer::new(str))
}
