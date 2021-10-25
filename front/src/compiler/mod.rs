use crate::{
    error::ParseError,
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};
use std::{iter::Peekable, mem::take};
use vm::{Symbol, OpCode, Constant, Bytecode};

type ParseResult = Result<(), ParseError>;

pub struct Compiler {
    lexer: Peekable<Lexer>,
    instructions: Vec<OpCode>,
    constants: Vec<Constant>,
    current: Token,
}

impl Compiler {
    pub fn compile(lexer: Lexer) -> Result<Bytecode, ParseError> {
        let mut this = Self {
            lexer: lexer.peekable(),
            constants: vec![],
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
                &[Tkt::Semicolon, Tkt::Eof],
                format!(
                    "expected `;` after expression, found `{}`",
                    this.current.token
                ),
            )?;
        }

        Ok(Bytecode {
            instructions: this.instructions,
            constants: this.constants,
        })
    }

    fn emit(&mut self, intr: OpCode) {
        self.instructions.push(intr)
    }

    fn emit_const(&mut self, constant: Constant) {
        self.constants.push(constant)
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

    fn consume(&self, token: &[Tkt], err: impl Into<String>) -> ParseResult {
        if !token.contains(&self.current.token) {
            self.throw(err)
        } else {
            Ok(())
        }
    }

    fn expression(&mut self) -> ParseResult {
        self.assign()
    }

    fn throw(&self, err: impl Into<String>) -> ParseResult {
        ParseError::throw(self.current.line, self.current.column, err.into())
    }

    fn assign(&mut self) -> ParseResult {
        if let Tkt::Var(val) = self.current.token.clone() {
            if let Some(Ok(Token {
                token: Tkt::Assign, ..
            })) = self.lexer.peek()
            {
                self.next()?;
                self.next()?;
                self.equality()?;
                self.emit(OpCode::Save);
                self.emit_const(Constant::Sym(Symbol::new(val)));
            } else {
                self.equality()?;
            }
        } else {
            self.equality()?;
        }

        Ok(())
    }

    fn equality(&mut self) -> ParseResult {
        self.bitwise()?;

        while let Tkt::Eq = self.current.token {
            let operator = match self.current.token {
                Tkt::Eq => OpCode::Eq,
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
                Tkt::BitAnd => OpCode::BitAnd,
                Tkt::BitOr => OpCode::BitOr,
                Tkt::BitXor => OpCode::Xor,
                Tkt::Shr => OpCode::Shr,
                Tkt::Shl => OpCode::Shl,
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
                Tkt::Add => OpCode::Add,
                Tkt::Sub => OpCode::Sub,
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
                Tkt::Mul => OpCode::Mul,
                Tkt::Div => OpCode::Div,
                _ => unreachable!(),
            };
            self.next()?;
            self.unary()?;
            self.emit(operator);
        }

        Ok(())
    }

    fn unary(&mut self) -> ParseResult {
        use OpCode::*;

        if matches!(self.current.token, Tkt::Sub | Tkt::Not) {
            let operator = match self.current.token {
                Tkt::Sub => Neg,
                Tkt::Not => Not,
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
        macro_rules! push {
            ($($tt:tt)+) => {{
                self.emit(Push);
                self.emit_const($($tt)+)
            }}
        }

        use {OpCode::*, Constant::*};
        let tk = take(&mut self.current.token);
        match tk {
            Tkt::Num(n) => push!(Num(n)),
            Tkt::Str(s) => push!(Str(s)),
            Tkt::Sym(s) => push!(Sym(Symbol::new(s))),
            Tkt::True => push!(Bool(true)),
            Tkt::False => push!(Bool(false)),
            Tkt::Var(v) => {
                self.emit(Load);
                self.emit_const(Sym(Symbol::new(v)))
            },
            Tkt::Nil => push!(Nil),
            Tkt::Lparen => {
                self.next()?;
                self.expression()?;
                self.consume(
                    &[Tkt::Rparen],
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
