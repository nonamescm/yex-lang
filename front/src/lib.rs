#![deny(missing_docs)]
//! Compiler for the yex language
mod compiler;
mod error;
mod lexer;
mod parser;
mod tokens;

use compiler::Compiler;
pub use error::ParseError;

use lexer::Lexer;
use parser::{
    ast::{Expr, Stmt},
    Parser,
};
use vm::VirtualMachine;

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

    let compiler = Compiler::new();
    let (bt, ct) = compiler.compile_expr(&ast);

    println!("{:?}", bt);

    let mut vm = VirtualMachine::default();
    vm.set_consts(ct);
    let _ = vm.run(&bt).unwrap();
    println!("{}", vm.pop_last());

    Ok(ast)
}
