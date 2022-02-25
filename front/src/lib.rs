#![deny(missing_docs)]
//! Compiler for the yex language
mod error;
mod lexer;
mod parser;
mod tokens;

pub use error::ParseError;

use lexer::Lexer;
use parser::{ast::{Stmt, Expr}, Parser};

/// Parses a given string into an AST
pub fn parse<T: Into<String>>(str: T) -> Result<Vec<Stmt>, error::ParseError> {
    let lexer = Lexer::new(str);
    let parser = Parser::new(lexer)?;
    let ast = parser.parse()?;

    Ok(ast)
}

/// Parses the given string in a single expression
pub fn parse_expr<T: Into<String>>(str: T) -> Result<Expr, error::ParseError> {
    let lexer = Lexer::new(str);

    let parser = Parser::new(lexer)?;
    let ast = parser.parse_expr()?;

    Ok(ast)
}
