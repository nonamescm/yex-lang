use crate::{
    error::ParseError,
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};
use std::iter::Peekable;
use vm::{Instruction, Literal};

type ParseResult = std::result::Result<(), ParseError>;

pub struct Compiler {
    lexer: Peekable<Lexer>,
    instructions: Vec<Instruction>,
    current: Token,
}

impl Compiler {
    pub fn compile(lexer: Lexer) -> Result<Vec<Instruction>, ParseError> {
        let mut this = Self {
            lexer: lexer.peekable(),
            instructions: vec![],
            current: Token {
                line: 0,
                column: 0,
                token: Tkt::Eof,
            },
        };
        this.next()?;

        this.expression()?;

        Ok(this.instructions)
    }

    fn emit(&mut self, intr: Instruction) {
        self.instructions.push(intr)
    }

    fn next(&mut self) -> ParseResult {
        let tk = self.lexer.next();
        self.current = tk.unwrap_or(Ok(Token {
            line: 0,
            column: 0,
            token: Tkt::Eof,
        }))?;

        Ok(())
    }

    fn consume(&self, token: Tkt, err: impl Into<String>) -> ParseResult {
        if self.current.token != token {
            self.throw(err)
        } else {
            Ok(())
        }
    }

    fn expression(&mut self) -> ParseResult {
        self.bitwise()
    }

    fn throw(&self, err: impl Into<String>) -> ParseResult {
        ParseError::throw(self.current.line, self.current.column, err.into())
    }

    fn bitwise(&mut self) -> ParseResult {
        self.term()?; // expands to a unary rule

        while let Tkt::BitAnd | Tkt::BitOr | Tkt::BitRs | Tkt::BitLs | Tkt::BitXor = self.current.token {
            let operator = match self.current.token {
                Tkt::BitAnd => Instruction::BitAnd,
                Tkt::BitOr => Instruction::BitOr,
                Tkt::BitXor => Instruction::Xor,
                Tkt::BitRs => Instruction::Shr,
                Tkt::BitLs => Instruction::Shl,
                _ => unreachable!(),
            };
            self.next()?;
            self.term()?;
            self.emit(operator);
        }

        Ok(())
    }

    fn term(&mut self) -> ParseResult {
        self.fact()?; // expands to a unary rule

        while let Tkt::Add | Tkt::Sub = self.current.token {
            let operator = match self.current.token {
                Tkt::Add => Instruction::Add,
                Tkt::Sub => Instruction::Sub,
                _ => unreachable!(),
            };
            self.next()?;
            self.fact()?;
            self.emit(operator);
        }

        Ok(())
    }

    fn fact(&mut self) -> ParseResult {
        self.unary()?; // expands to a unary rule

        while let Tkt::Mul | Tkt::Div = self.current.token {
            let operator = match self.current.token {
                Tkt::Mul => Instruction::Mul,
                Tkt::Div => Instruction::Div,
                _ => unreachable!(),
            };
            self.next()?;
            self.unary()?;
            self.emit(operator);
        }

        Ok(())
    }

    fn unary(&mut self) -> ParseResult {
        use Instruction::*;

        if matches!(self.current.token, Tkt::Sub) {
            let operator = match self.current.token {
                Tkt::Sub => Neg,
                _ => unreachable!(),
            };
            self.next()?;
            self.unary()?; // emits the expression to be applied
            self.emit(operator)
        } else {
            self.primary()?;
            self.next()?;
        }

        Ok(())
    }

    fn primary(&mut self) -> ParseResult {
        use {Instruction::*, Literal::*};

        match self.current.token {
            Tkt::Num(n) => self.emit(Push(Num(n))),
            Tkt::Lparen => {
                self.next()?;
                self.expression()?;
                self.consume(
                    Tkt::Rparen,
                    format!(
                        "expected `)` to close the block, found `{}`",
                        self.current.token
                    ),
                )?;
            }
            _ => self.throw(format!(
                "expected expression, found `{}`",
                self.current.token
            ))?,
        }

        Ok(())
    }
}
