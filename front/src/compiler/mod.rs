use crate::{
    error::ParseError,
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};
use std::iter::Peekable;
use vm::{Instruction, Literal, symbol::Symbol};

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
        while {
            this.next()?;
            this.current.token != Tkt::Eof
        } {
            this.expression()?;
            this.consume(
                vec![Tkt::Semicolon, Tkt::Eof],
                format!(
                    "expected `;` after expression, found `{}`",
                    this.current.token
                ),
            )?;
        }

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

    fn consume(&self, token: Vec<Tkt>, err: impl Into<String>) -> ParseResult {
        if !token.contains(&self.current.token) {
            self.throw(err)
        } else {
            Ok(())
        }
    }

    fn expression(&mut self) -> ParseResult {
        self.equality()
    }

    fn throw(&self, err: impl Into<String>) -> ParseResult {
        ParseError::throw(self.current.line, self.current.column, err.into())
    }

    fn equality(&mut self) -> ParseResult {
        self.bitwise()?;

        while let Tkt::Eq = self.current.token {
            let operator = match self.current.token {
                Tkt::Eq => Instruction::Eq,
                _ => unreachable!(),
            };
            self.next()?;
            self.bitwise()?;
            self.emit(operator);
        }

        Ok(())
    }

    fn bitwise(&mut self) -> ParseResult {
        self.term()?; // expands to a unary rule

        while let Tkt::BitAnd | Tkt::BitOr | Tkt::Shr | Tkt::Shl | Tkt::BitXor = self.current.token
        {
            let operator = match self.current.token {
                Tkt::BitAnd => Instruction::BitAnd,
                Tkt::BitOr => Instruction::BitOr,
                Tkt::BitXor => Instruction::Xor,
                Tkt::Shr => Instruction::Shr,
                Tkt::Shl => Instruction::Shl,
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
        let tk = std::mem::take(&mut self.current.token);
        match tk {
            Tkt::Num(n) => self.emit(Push(Num(n))),
            Tkt::Str(s) => self.emit(Push(Str(s))),
            Tkt::Sym(s) => self.emit(Push(Sym(Symbol::new(s)))),
            Tkt::Lparen => {
                self.next()?;
                self.expression()?;
                self.consume(
                    vec![Tkt::Rparen],
                    format!(
                        "expected `)` to close the block, found `{}`",
                        self.current.token
                    ),
                )?;
            }
            _ => self.throw(format!("expected expression, found `{}`", tk))?,
        }

        Ok(())
    }
}
