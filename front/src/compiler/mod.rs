use crate::{
    error::ParseError,
    lexer::Lexer,
    tokens::{Token, TokenType},
};
use std::iter::Peekable;
use vm::{Instruction, Literal};

type ParseResult = std::result::Result<(), ParseError>;

pub struct Compiler {
    lexer: Peekable<Lexer>,
    instructions: Vec<Instruction>,
    current_token: Token,
}

impl Compiler {
    pub fn compile(lexer: Lexer) -> Result<Vec<Instruction>, ParseError> {
        let mut this = Self {
            lexer: lexer.peekable(),
            instructions: vec![],
            current_token: Token::default(),
        };
        this.next_token();

        this.unary()?;

        Ok(this.instructions)
    }

    fn emit(&mut self, intr: Instruction) {
        self.instructions.push(intr)
    }

    fn next_token(&mut self) -> &Token {
        let tk = self.lexer.next().unwrap_or_default();
        self.current_token = tk;
        &self.current_token
    }

    fn unary(&mut self) -> ParseResult {
        use Instruction::*;

        match self.current_token.token {
            TokenType::Sub => {
                self.next_token();
                self.nary()?;
                self.emit(Neg);
            },
            _ => self.nary()?,
        };

        Ok(())
    }

    fn nary(&mut self) -> ParseResult {
        use {Instruction::*, Literal::*};
        match self.current_token.token {
            TokenType::Num(n) => self.emit(Push(Num(n))),
            _ => ParseError::throw(
                self.current_token.line,
                self.current_token.column,
                format!("Expected expression, found `{}`", self.current_token.token),
            )?,
        };
        Ok(())
    }
}
