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
    constants: Vec<Constant>,
    current: Token,
    proxies: Vec<(Vec<OpCodeMetadata>, usize)>,
}

impl Compiler {
    pub fn compile(lexer: Lexer) -> Result<(Bytecode, Vec<Constant>), ParseError> {
        let mut this = Self {
            lexer: lexer.peekable(),
            constants: vec![],
            current: Token {
                line: 0,
                column: 0,
                token: Tkt::Eof,
            },
            proxies: vec![(vec![], 0)],
        };
        while {
            this.next()?;
            this.current.token != Tkt::Eof
        } {
            this.expression()?;
            this.consume(
                &[Tkt::Semicolon, Tkt::Eof],
                format!(
                    "expected `;` or <eof> after expression, found `{}`",
                    this.current.token
                ),
            )?;
        }

        Ok((this.proxies.pop().unwrap().0, this.constants))
    }

    fn compiled_opcodes(&self) -> usize {
        self.proxies.last().unwrap().1
    }

    fn emit(&mut self, intr: OpCode) {
        let op = OpCodeMetadata {
            line: self.current.line,
            column: self.current.column,
            opcode: intr,
        };

        let (proxy, compiled) = self.proxies.last_mut().unwrap();
        proxy.push(op);
        *compiled += 1;
    }

    fn emit_patch(&mut self, intr: OpCode, idx: usize) {
        let (proxy, _) = self.proxies.last_mut().unwrap();
        proxy[idx].opcode = intr;
    }

    fn emit_const_push(&mut self, constant: Constant) {
        if let Some(idx) = self.constants.iter().position(|c| c == &constant) {
            self.emit(OpCode::Push(idx))
        } else {
            self.emit(OpCode::Push(self.constants.len()));
            self.constants.push(constant)
        }
    }

    fn emit_const(&mut self, constant: Constant) -> usize {
        if let Some(idx) = self.constants.iter().position(|c| c == &constant) {
            idx
        } else {
            self.constants.push(constant);
            self.constants.len() - 1
        }
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

    fn assert(&mut self, token: &[Tkt], err: impl Into<String>) -> ParseResult {
        if !token.contains(&self.current.token) {
            self.throw(err)
        } else {
            Ok(())
        }
    }

    fn consume(&mut self, token: &[Tkt], err: impl Into<String>) -> ParseResult {
        self.assert(token, err)?;
        self.next()?;

        Ok(())
    }

    fn throw(&self, err: impl Into<String>) -> ParseResult {
        ParseError::throw(self.current.line, self.current.column, err.into())
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

        (!matches!(self.current.token, Tkt::Idnt(_))).then(|| self.throw("Expected argument name"));

        self.proxies.push((vec![], 0));

        if let Tkt::Idnt(id) = take(&mut self.current.token) {
            let idx = self.emit_const(Constant::Val(Symbol::new(id)));
            self.emit(OpCode::Save(idx));
        }

        self.next()?;

        self.consume(&[Tkt::Arrow], "Expected `->` after argument")?;

        self.expression()?;
        let (body, _) = self.proxies.pop().unwrap();

        self.emit_const_push(Constant::Fun(body));

        Ok(())
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

        self.emit(OpCode::Nsc); // creates a new scope

        let then_jump_ip = self.compiled_opcodes();
        self.emit(OpCode::Jmf(0));

        self.expression()?; // compiles the if branch

        let else_jump_ip = self.compiled_opcodes();
        self.emit(OpCode::Jmp(0));

        self.emit_patch(OpCode::Jmf(self.compiled_opcodes()), then_jump_ip);

        self.consume(&[Tkt::Else], "Expected `else` after if")?;
        self.expression()?; // compiles the else branch
        self.consume(&[Tkt::End], "expected `end` to close the else block")?;
        self.emit_patch(OpCode::Jmp(self.compiled_opcodes()), else_jump_ip);

        self.emit(OpCode::Esc); // End the new scope

        Ok(())
    }

    fn assign_val(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Val); // security check
        self.next()?; // skips the val token

        let name = match take(&mut self.current.token) {
            Tkt::Idnt(v) => v,
            o => return self.throw(format!("Expected variable name after `val`, found {}", o)),
        };
        self.next()?;

        self.consume(
            &[Tkt::Assign],
            format!("Expected `=` after name, found {}", self.current.token),
        )?;
        self.expression()?;
        let idx = self.emit_const(Constant::Val(Symbol::new(name)));
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
            self.call()?;
            self.next()?;
        }

        Ok(())
    }

    fn call(&mut self) -> ParseResult {
        self.primary()?; // compiles the called expression

        while matches!(
            self.lexer.peek().unwrap().as_ref().map(|c| &c.token),
            Ok(Tkt::Lparen)
        ) {
            self.next()?;
            self.block()?;
            self.emit(OpCode::Call);
        }

        Ok(())
    }

    fn block(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Lparen);

        self.next()?;
        self.expression()?;
        self.assert(
            &[Tkt::Rparen],
            format!(
                "expected `)` to close the block, found `{}`",
                self.current.token
            ),
        )
    }

    fn primary(&mut self) -> ParseResult {
        macro_rules! push {
            ($type: expr) => {{
                self.emit_const_push($type);
            }};
        }

        use {Constant::*, OpCode::*};
        match take(&mut self.current.token) {
            Tkt::Num(n) => push!(Num(n)),
            Tkt::Str(str) => push!(Str(str)),
            Tkt::Sym(sym) => push!(Sym(sym)), // don't allow for duplicated symbols
            Tkt::True => push!(Bool(true)),
            Tkt::False => push!(Bool(false)),
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
                    self.emit_const(Val(v));
                }
            }
            Tkt::Nil => push!(Nil),

            Tkt::Lparen => {
                self.current.token = Tkt::Lparen; // `(` is needed for self.block() to work correctly
                self.block()?;
            }
            tk => self.throw(format!("expected expression, found `{}`", tk))?,
        }

        Ok(())
    }
}
