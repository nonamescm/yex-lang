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
    proxies: Vec<Vec<OpCodeMetadata>>,
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
            proxies: vec![vec![]],
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

        Ok((this.proxies.pop().unwrap(), this.constants))
    }

    fn compiled_opcodes(&self) -> usize {
        self.proxies.last().unwrap().len() - 1
    }

    fn emit(&mut self, intr: OpCode) {
        let op = OpCodeMetadata {
            line: self.current.line,
            column: self.current.column,
            opcode: intr,
        };
        self.emit_metadata(op)
    }

    fn emit_metadata(&mut self, op: OpCodeMetadata) {
        let proxy = self.proxies.last_mut().unwrap();
        proxy.push(op);
    }

    fn emit_patch(&mut self, intr: OpCode, idx: usize) {
        let proxy = self.proxies.last_mut().unwrap();
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
            Tkt::Let => self.let_(),
            Tkt::Fn => self.fn_(),
            Tkt::Become => self.become_(),
            _ => self.equality(),
        }
    }

    fn function(&mut self) -> ParseResult {
        let mut arity = 0;
        self.proxies.push(vec![]);

        while matches!(self.current.token, Tkt::Name(_)) {
            let id = match take(&mut self.current.token) {
                Tkt::Name(id) => id,
                _ => unreachable!(),
            };

            let idx = self.emit_const(Constant::Val(Symbol::new(id)));
            self.emit(OpCode::Save(idx));
            arity += 1;
            self.next()?;
        }

        self.consume(
            &[Tkt::Assign],
            format!(
                "Expected `=` after argument, found `{}`",
                self.current.token
            ),
        )?;

        self.expression()?;
        let body= self.proxies.pop().unwrap();

        self.emit_const_push(Constant::Fun { body, arity });

        Ok(())
    }

    fn become_(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Become); // security check
        self.next()?;
        self.call()?;
        let proxy = &mut self.proxies.last_mut().unwrap();
        match proxy.pop() {
            Some(OpCodeMetadata {
                opcode: OpCode::Call(arity),
                line,
                column,
            }) => self.emit_metadata(OpCodeMetadata {
                line,
                column,
                opcode: OpCode::TCall(arity),
            }),
            _ => unreachable!(),
        }
        self.next()?; // skips the leading `)` token
        Ok(())
    }

    fn fn_(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Fn); // security check
        self.next()?;
        self.function()
    }

    fn if_elif(&mut self) -> Result<usize, ParseError> {
        self.next()?; // skips the if token

        self.expression()?; // compiles the condition
        self.consume(
            &[Tkt::Do],
            format!(
                "expected `do` after condition, found {}",
                &self.current.token
            ),
        )?; // checks for do

        let then_jump_ip = self.compiled_opcodes();
        self.emit(OpCode::Jmf(0));

        self.expression()?; // compiles the if branch

        Ok(then_jump_ip)
    }

    fn condition(&mut self) -> ParseResult {
        assert!(matches!(self.current.token, Tkt::If | Tkt::Elif)); // security check

        self.emit(OpCode::Nsc); // creates a new scope

        let mut patch_stack = vec![];

        while matches!(self.current.token, Tkt::If | Tkt::Elif) {
            let then_jump_ip = self.if_elif()?;
            self.emit_patch(OpCode::Jmf(self.compiled_opcodes() + 1), then_jump_ip);

            patch_stack.push(self.compiled_opcodes());
            self.emit(OpCode::Jmp(0));
        }

        self.consume(
            &[Tkt::Else],
            format!("Expected `else` after if, found `{}`", self.current.token),
        )?;
        self.expression()?; // compiles the else branch
        self.consume(
            &[Tkt::End],
            format!("Expected `else` after if, found `{}`", self.current.token),
        )?;

        let compiled_opcodes = self.compiled_opcodes();
        let jmp = OpCode::Jmp(compiled_opcodes);

        patch_stack
            .into_iter()
            .for_each(|it| self.emit_patch(jmp, it));

        self.emit(OpCode::Esc); // End the new scope

        Ok(())
    }

    fn let_(&mut self) -> ParseResult {
        assert_eq!(self.current.token, Tkt::Let); // security check
        self.next()?; // skips the let token

        let name = match take(&mut self.current.token) {
            Tkt::Name(v) => v,
            o => return self.throw(format!("Expected variable name after `let`, found {}", o)),
        };
        self.next()?;

        if matches!(self.current.token, Tkt::Name(_)) {
            self.function()?;
        } else {
            self.consume(
                &[Tkt::Assign],
                format!("Expected `=` after name, found {}", self.current.token),
            )?;
            self.expression()?;
        }

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

    fn call_args(&mut self, arity: &mut usize) -> ParseResult {
        self.next()?;

        loop {
            if matches!(self.current.token, Tkt::Rparen) {
                break;
            }
            self.next()?;

            self.expression()?; // compiles the argument
            *arity += 1;
            if !matches!(&self.current.token, Tkt::Colon | Tkt::Rparen) {
                self.throw(format!(
                    "Expected `,`, `)` or other token, found {}",
                    &self.current.token
                ))?
            }
        }
        Ok(())
    }

    fn call(&mut self) -> ParseResult {
        let comp = self.compiled_opcodes();
        self.primary()?; // compiles the called expresion
        let callee = {
            let mut old_comp = self.compiled_opcodes() - comp;
            let mut proxy = self.proxies.pop().unwrap();
            let mut new = vec![];
            
            while old_comp > 0 {
                new.push(proxy.pop().unwrap());
                old_comp -= 1;
            }

            self.proxies.push(proxy);
            new
        };

        let mut arity = 0;

        if matches!(
            self.lexer.peek().unwrap().as_ref().map(|c| &c.token),
            Ok(Tkt::Lparen)
        ) {
            self.call_args(&mut arity)?;
        } else {
            callee.iter().for_each(|it| self.emit(it.opcode));
            return Ok(());
        }
        callee.iter().for_each(|it| self.emit(it.opcode));

        self.emit(OpCode::Call(arity));

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
            Tkt::Name(v) => {
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
