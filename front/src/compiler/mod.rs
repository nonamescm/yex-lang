use crate::{
    error::ParseError,
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};
use std::{iter::Peekable, mem::take};
use vm::{Bytecode, Constant, OpCode, OpCodeMetadata, Symbol};

type ParseResult = Result<(), ParseError>;

pub struct Compiler {
    lexer: Peekable<Lexer>,
    instructions: Vec<OpCodeMetadata>,
    constants: Vec<Constant>,
    current: Token,
    compiled_opcodes: usize,
    emit_proxy_mode: bool,
    proxy: Option<Vec<OpCodeMetadata>>,
}

impl Compiler {
    pub fn compile(lexer: Lexer) -> Result<(Bytecode, Vec<Constant>), ParseError> {
        let mut this = Self {
            lexer: lexer.peekable(),
            constants: vec![],
            instructions: vec![],
            current: Token {
                line: 0,
                column: 0,
                token: Tkt::Eof,
            },
            compiled_opcodes: 0,
            emit_proxy_mode: false,
            proxy: None,
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

        Ok((this.instructions, this.constants))
    }

    fn emit(&mut self, intr: OpCode) {
        let op = OpCodeMetadata {
            line: self.current.line,
            column: self.current.column,
            opcode: intr,
        };

        self.compiled_opcodes += 1;

        if self.emit_proxy_mode {
            match &mut self.proxy {
                None => self.proxy = Some(vec![op]),
                Some(prx) => (*prx).push(op),
            }
        } else {
            self.instructions.push(op)
        }
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

    fn consume(&mut self, token: &[Tkt], err: impl Into<String>) -> ParseResult {
        if !token.contains(&self.current.token) {
            self.throw(err)
        } else {
            self.next()
        }
    }

    fn expression(&mut self) -> ParseResult {
        match self.current.token {
            Tkt::If => self.condition(),
            Tkt::Val => self.assign_val(),
            Tkt::Fun => self.function(),
            _ => self.equality(),
        }
    }

    fn function(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Fun); // security check
        self.next()?;

        let mut arity: usize = 0;

        while let Tkt::Idnt(_) = self.current.token {
            arity += 1;
            self.next()?;
        }
        self.consume(&[Tkt::Arrow], "Expected `->` after argument list")?;

        self.emit_proxy_mode = true;
        self.expression()?;
        let body = std::mem::take(&mut self.proxy).unwrap();
        self.emit_proxy_mode = false;

        self.emit_const(Constant::Fun {
            arity,
            body,
        });

        Ok(())
    }

    fn throw(&self, err: impl Into<String>) -> ParseResult {
        ParseError::throw(self.current.line, self.current.column, err.into())
    }

    fn condition(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::If); // security check
        self.next()?; // skips the if token

        self.expression()?; // compiles the condition
        self.consume(
            &[Tkt::Do],
            format!(
                "expected `do` after condition, found {}",
                &self.current.token
            ),
        )?; // checks for do

        let then_jump_ip = self.compiled_opcodes;
        self.emit(OpCode::Jmf(0));

        self.expression()?; // compiles the if branch

        let else_jump_ip = self.compiled_opcodes;
        self.emit(OpCode::Jmp(0));

        self.instructions[then_jump_ip].opcode = OpCode::Jmf(self.compiled_opcodes);

        self.consume(&[Tkt::Else], "Expected `else` after if")?;
        self.expression()?; // compiles the else branch
        self.consume(&[Tkt::End], "expected `end` to close the else block")?;
        self.instructions[else_jump_ip].opcode = OpCode::Jmp(self.compiled_opcodes);

        Ok(())
    }

    fn assign_val(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Val); // security check
        self.next()?; // skips the val token

        let name = match std::mem::take(&mut self.current.token) {
            Tkt::Idnt(v) => v,
            o => return self.throw(format!("Expected variable name after `val`, found {}", o)),
        };
        self.next()?;

        self.consume(
            &[Tkt::Assign],
            format!("Expected `=` after name, found {}", self.current.token),
        )?;
        self.expression()?;
        self.emit_const(Constant::Val(Symbol::new(name)));
        let idx = self.constants.len() - 1;
        self.emit(OpCode::Save(idx));

        self.consume(
            &[Tkt::In],
            format!(
                "Expected `in` after val expression, found {}",
                self.current.token
            ),
        )?;
        self.expression()?;

        self.emit(OpCode::Drop(idx));

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
            ($type: tt, $item: expr) => {{
                if let Some(idx) = self.constants.iter().position(|c| match c {
                    $type(ref s) => s == &$item,
                    _ => false,
                }) {
                    self.emit(Push(idx))
                } else {
                    self.emit(Push(self.constants.len()));
                    self.emit_const($type($item))
                }
            }};

            ($type: expr) => {{
                self.emit(Push(self.constants.len()));
                self.emit_const($type)
            }};
        }

        use {Constant::*, OpCode::*};
        match take(&mut self.current.token) {
            Tkt::Num(n) => push!(Num, n),
            Tkt::Str(str) => push!(Str(str)),
            Tkt::Sym(sym) => push!(Sym, sym), // don't allow for duplicated symbols
            Tkt::True => push!(Bool, true),
            Tkt::False => push!(Bool, false),
            Tkt::Idnt(v) => {
                let v = Symbol::new(v);

                if let Some(idx) = self
                    .constants
                    .iter()
                    .position(|c| matches!(c, Val(ref s) if s == &v))
                {
                    self.emit(Load(idx))
                } else {
                    self.emit(Load(self.constants.len()));
                    self.emit_const(Val(v))
                }
            }
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
            tk => self.throw(format!("expected expression, found `{}`", tk))?,
        }

        Ok(())
    }
}
