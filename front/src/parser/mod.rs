use std::{iter::Peekable, mem::take};

use crate::{
    error::{ParseError, ParseResult},
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};

use self::ast::{
    ArmType, Bind, Def, Expr, ExprKind, Literal, Stmt, StmtKind, VarDecl, WhenArm, WhenElse,
};

pub mod ast;

pub struct Parser {
    lexer: Peekable<Lexer>,
    current: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> ParseResult<Self> {
        let mut this = Parser {
            lexer: lexer.peekable(),
            current: Token::default(),
        };
        this.next()?;
        Ok(this)
    }

    pub fn parse(mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while self.current.token != Tkt::Eof {
            match self.current.token {
                Tkt::Class => {
                    stmts.push(self.type_bind()?);
                }

                Tkt::Def => stmts.push(self.def_global()?),
                Tkt::Let => stmts.push(self.let_global()?),

                _ => stmts.push(self.expr()?.into()),
            }
        }

        Ok(stmts)
    }

    pub fn let_global(&mut self) -> ParseResult<Stmt> {
        let line = self.current.line;
        let column = self.current.column;

        self.expect(Tkt::Let)?;

        let bind = self.var_decl()?;

        self.expect(Tkt::Assign)?;

        let value = Box::new(self.expr()?);

        Ok(Stmt::new(
            StmtKind::Let(Bind::new(bind, value, line, column)),
            line,
            column,
        ))
    }

    pub fn parse_expr(mut self) -> ParseResult<Expr> {
        self.expr()
    }

    fn type_bind(&mut self) -> ParseResult<Stmt> {
        self.expect(Tkt::Class)?;
        let line = self.current.line;
        let column = self.current.column;

        let name = self.var_decl()?;
        let mut params = Vec::new();

        self.expect(Tkt::Lparen)?;
        while self.current.token != Tkt::Rparen {
            params.push(self.var_decl()?);

            if self.current.token == Tkt::Rparen {
                break;
            }

            self.expect_and_skip(Tkt::Comma)?;
        }

        self.next()?;

        let mut methods = Vec::new();
        let mut init = None;

        while self.current.token != Tkt::End {
            let def = match self.def_global()?.kind {
                StmtKind::Def(def) => def,
                _ => unreachable!(),
            };

            if def.bind.name.as_str() == "init" {
                init = Some(def);
                continue;
            }

            match def.value.kind {
                ExprKind::Lambda { ref args, .. }
                    if !args.is_empty() && args[0].name.as_str() == "this" =>
                {
                    methods.push(def)
                }
                _ => self.throw("Methods should receive `this` as a parameter")?,
            }
        }
        self.next()?;

        Ok(Stmt::new(
            StmtKind::Class {
                name,
                params,
                methods,
                init,
            },
            line,
            column,
        ))
    }

    fn def_global(&mut self) -> ParseResult<Stmt> {
        let line = self.current.line;
        let column = self.current.column;

        self.expect(Tkt::Def)?;

        let bind = self.var_decl()?;
        let value = self.function()?;

        Ok(Stmt::new(StmtKind::Def(Def { bind, value }), line, column))
    }

    fn next(&mut self) -> ParseResult<()> {
        self.current = self.lexer.next().unwrap()?;
        Ok(())
    }

    fn throw<T>(&self, err: impl Into<String>) -> ParseResult<T> {
        ParseError::throw(self.current.line, self.current.column, err.into())
    }

    fn expect(&mut self, expected: Tkt) -> ParseResult<()> {
        self.assert(expected)?;
        self.next()
    }

    fn assert(&mut self, expected: Tkt) -> ParseResult<()> {
        if self.current.token == expected {
            Ok(())
        } else {
            self.throw(format!(
                "Expected {}, found `{}`",
                expected, self.current.token
            ))
        }
    }

    fn skip(&mut self, tokens: Tkt) -> ParseResult<()> {
        while self.current.token == tokens {
            self.next()?;
        }
        Ok(())
    }

    fn expect_and_skip(&mut self, tokens: Tkt) -> ParseResult<()> {
        self.expect(tokens.clone())?;
        self.skip(tokens)
    }

    #[allow(dead_code)]
    fn peek(&mut self) -> ParseResult<&Token> {
        match self.lexer.peek().unwrap() {
            Ok(t) => Ok(t),
            Err(e) => Err(*e),
        }
    }

    fn expr(&mut self) -> ParseResult<Expr> {
        match self.current.token {
            Tkt::Let => self.let_(),
            Tkt::Def => self.def_(),
            Tkt::If => self.condition(),
            Tkt::Fn => self.fn_(),
            Tkt::Become => self.become_(),
            Tkt::When => self.when_(),
            Tkt::Do => {
                self.expect(Tkt::Do)?;
                self.block(Tkt::End)
            }
            Tkt::Try => self.try_(),
            _ => self.logic_or(),
        }
    }

    fn condition(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::If)?;
        let line = self.current.line;
        let column = self.current.column;

        let cond = self.expr()?;

        self.expect(Tkt::Then)?;

        let then = self.block(Tkt::Else)?;
        let else_ = self.block(Tkt::End)?;

        Ok(Expr::new(
            ExprKind::If {
                cond: Box::new(cond),
                then: Box::new(then),
                else_: Box::new(else_),
            },
            line,
            column,
        ))
    }

    fn args(&mut self) -> ParseResult<Vec<VarDecl>> {
        self.assert(Tkt::Lparen)?;

        let mut args = vec![];

        self.next()?;
        while self.current.token != Tkt::Rparen {
            let var = self.var_decl()?;
            args.push(var);

            match &self.current.token {
                Tkt::Comma => self.skip(Tkt::Comma)?,
                Tkt::Rparen => break,
                _ => self.throw(format!(
                    "Expected `,`, `)` or other token, found `{}`",
                    &self.current.token
                ))?,
            }
        }
        self.next()?;

        Ok(args)
    }

    fn become_(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Become)?;

        let line = self.current.line;
        let column = self.current.column;

        let callee = self.expr()?;

        match callee.kind {
            ExprKind::App { callee, args, .. } => Ok(Expr::new(
                ExprKind::App {
                    callee,
                    args,
                    tail: true,
                },
                line,
                column,
            )),
            _ => self.throw("Become can only be used on function calls"),
        }
    }

    fn when_(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::When)?;

        let line = self.current.line;
        let column = self.current.column;

        let expr = Box::new(self.expr()?);

        self.expect(Tkt::Do)?;

        let mut arms = vec![];

        while self.current.token != Tkt::End {
            if self.current.token == Tkt::Else {
                arms.push(ArmType::Else(self.when_else()?));
                continue;
            }

            let arm = self.when_arm()?;

            arms.push(ArmType::Arm(arm));
        }

        self.expect(Tkt::End)?;

        Ok(Expr::new(ExprKind::When { expr, arms }, line, column))
    }

    fn when_else(&mut self) -> ParseResult<WhenElse> {
        self.expect(Tkt::Else)?;

        let line = self.current.line;
        let column = self.current.column;

        let ident = self.var_decl()?;

        let guard = if self.current.token == Tkt::If {
            self.next()?;
            Some(self.expr()?)
        } else {
            None
        };

        self.expect(Tkt::FatArrow)?;

        let body = self.expr()?;

        Ok(WhenElse::new(ident, body, guard, line, column))
    }

    fn when_arm(&mut self) -> ParseResult<WhenArm> {
        let line = self.current.line;
        let column = self.current.column;

        let cond = self.expr()?;

        let guard = if self.current.token == Tkt::If {
            self.next()?;
            Some(self.expr()?)
        } else {
            None
        };

        self.expect(Tkt::FatArrow)?;

        let body = self.expr()?;

        Ok(WhenArm::new(cond, body, guard, line, column))
    }

    fn try_(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Try)?;

        let line = self.current.line;
        let column = self.current.column;

        let body = Box::new(self.block(Tkt::Rescue)?);

        let bind = self.var_decl()?;

        let rescue = Box::new(self.block(Tkt::End)?);

        Ok(Expr::new(
            ExprKind::Try { body, bind, rescue },
            line,
            column,
        ))
    }

    fn fn_(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Fn)?;
        self.function()
    }

    fn function(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let args = self.args()?;

        let body = self.fn_body()?;

        Ok(Expr::new(
            ExprKind::Lambda {
                args,
                body: Box::new(body),
            },
            line,
            column,
        ))
    }

    fn fn_body(&mut self) -> ParseResult<Expr> {
        if self.current.token == Tkt::Colon {
            self.next()?;
            Ok(self.expr()?)
        } else {
            self.expect(Tkt::Do)?;
            self.block(Tkt::End)
        }
    }

    fn block(&mut self, end: Tkt) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let mut body = vec![];

        while self.current.token != end {
            let expr = self.expr()?;
            body.push(expr);
        }

        self.next()?;

        Ok(Expr::new(ExprKind::Do(body), line, column))
    }

    fn var_decl(&mut self) -> ParseResult<VarDecl> {
        let name = match take(&mut self.current.token) {
            Tkt::Name(id) => id,
            other => self.throw(format!("Expected name, found `{}`", other))?,
        };

        self.next()?;

        Ok(VarDecl::new(name))
    }

    fn let_(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        self.expect(Tkt::Let)?;

        let bind = self.var_decl()?;

        self.expect(Tkt::Assign)?;

        let value = self.expr()?;

        Ok(Expr::new(
            ExprKind::Let(Bind::new(bind, Box::new(value), line, column)),
            line,
            column,
        ))
    }

    fn def_(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Def)?;

        let line = self.current.line;
        let column = self.current.column;

        let name = self.var_decl()?;
        let value = self.function()?;

        Ok(Expr::new(
            ExprKind::Def(Bind::new(name, Box::new(value), line, column)),
            line,
            column,
        ))
    }

    fn logic_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.logic_and()?;

        while let Tkt::Or = self.current.token {
            let op = take(&mut self.current.token).try_into().unwrap();

            self.next()?;
            let right = self.logic_and()?;

            let line = left.line();
            let column = left.column();

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn logic_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.eq()?;

        while let Tkt::And = self.current.token {
            let op = take(&mut self.current.token).try_into().unwrap();

            self.next()?;
            let right = self.eq()?;

            let line = left.line();
            let column = left.column();

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                line,
                column,
            );
        }

        Ok(left)
    }

    fn eq(&mut self) -> ParseResult<Expr> {
        let mut left = self.cmp()?;

        while let Tkt::Eq | Tkt::Ne = self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.cmp()?;

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op: op.token.try_into().unwrap(),
                    right: Box::new(right),
                },
                op.line,
                op.column,
            );
        }

        Ok(left)
    }

    fn cmp(&mut self) -> ParseResult<Expr> {
        let mut left = self.cons()?;

        while let Tkt::Less | Tkt::LessEq | Tkt::Greater | Tkt::GreaterEq = self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.cons()?;

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op: op.token.try_into().unwrap(),
                    right: Box::new(right),
                },
                op.line,
                op.column,
            );
        }

        Ok(left)
    }

    fn cons(&mut self) -> ParseResult<Expr> {
        let mut left = self.bitwise()?;

        while let Tkt::Cons = self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.cons()?;

            left = Expr::new(
                ExprKind::Cons {
                    head: Box::new(left),
                    tail: Box::new(right),
                },
                op.line,
                op.column,
            );
        }

        Ok(left)
    }

    fn bitwise(&mut self) -> ParseResult<Expr> {
        let mut left = self.term()?;

        while let Tkt::BitOr | Tkt::BitAnd | Tkt::BitXor | Tkt::Shr | Tkt::Shl = self.current.token
        {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.term()?;

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op: op.token.try_into().unwrap(),
                    right: Box::new(right),
                },
                op.line,
                op.column,
            );
        }

        Ok(left)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut left = self.fact()?;

        while let Tkt::Add | Tkt::Sub = self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.fact()?;

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op: op.token.try_into().unwrap(),
                    right: Box::new(right),
                },
                op.line,
                op.column,
            );
        }

        Ok(left)
    }

    fn fact(&mut self) -> ParseResult<Expr> {
        let mut left = self.prefix()?;

        while let Tkt::Mul | Tkt::Div = self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.prefix()?;

            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op: op.token.try_into().unwrap(),
                    right: Box::new(right),
                },
                op.line,
                op.column,
            );
        }

        Ok(left)
    }

    fn prefix(&mut self) -> ParseResult<Expr> {
        if let Tkt::Sub | Tkt::Not = &self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.prefix()?;
            Ok(Expr::new(
                ExprKind::UnOp(op.token.try_into().unwrap(), Box::new(right)),
                op.line,
                op.column,
            ))
        } else {
            self.instance()
        }
    }

    fn instance(&mut self) -> ParseResult<Expr> {
        if let Tkt::New = &self.current.token {
            let op = take(&mut self.current);
            self.next()?;

            let ty = Box::new(self.primary()?);
            self.next()?;

            self.assert(Tkt::Lparen)?;
            let args = self.call_args()?;

            Ok(Expr::new(ExprKind::New { ty, args }, op.line, op.column))
        } else {
            self.dot()
        }
    }

    fn dot(&mut self) -> ParseResult<Expr> {
        let mut obj = self.call()?;

        while self.current.token == Tkt::Dot {
            obj = self.dot_access(obj)?;
        }

        Ok(obj)
    }

    fn dot_access(&mut self, obj: Expr) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        self.next()?;

        let obj = Box::new(obj);

        let field = self.var_decl()?;
        if self.current.token == Tkt::Lparen {
            let args = self.call_args()?;

            Ok(Expr::new(
                ExprKind::Invoke { obj, field, args },
                line,
                column,
            ))
        } else {
            Ok(Expr::new(ExprKind::Field { obj, field }, line, column))
        }
    }

    fn call_args(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = vec![];

        self.next()?;
        while self.current.token != Tkt::Rparen {
            args.push(self.expr()?);

            match &self.current.token {
                Tkt::Comma => self.skip(Tkt::Comma)?,
                Tkt::Rparen => break,
                _ => self.throw(format!(
                    "Expected `,`, `)` or other token, found `{}`",
                    &self.current.token
                ))?,
            }
        }
        self.next()?;

        Ok(args)
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut callee = self.method_ref()?;

        while self.current.token == Tkt::Lparen {
            let args = self.call_args()?;

            let line = callee.line();
            let column = callee.column();

            callee = Expr::new(
                ExprKind::App {
                    callee: Box::new(callee),
                    tail: false,
                    args,
                },
                line,
                column,
            )
        }

        Ok(callee)
    }

    fn method_ref(&mut self) -> ParseResult<Expr> {
        let mut ty = self.primary()?;
        self.next()?;

        while self.current.token == Tkt::Len {
            self.next()?;
            let method = self.var_decl()?;

            ty = Expr::new(
                ExprKind::MethodRef {
                    ty: Box::new(ty),
                    method,
                },
                self.current.line,
                self.current.column,
            );
        }

        Ok(ty)
    }

    fn list(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        self.expect(Tkt::Lbrack)?;

        let mut exprs = Vec::new();
        while self.current.token != Tkt::Rbrack {
            exprs.push(self.expr()?); // compiles the argument

            match &self.current.token {
                Tkt::Comma => self.skip(Tkt::Comma)?,
                Tkt::Rbrack => break,
                _ => self.throw(format!(
                    "Expected `,`, `]` or other token, found `{}`",
                    &self.current.token
                ))?,
            }
        }

        Ok(Expr::new(ExprKind::List(exprs), line, column))
    }

    fn tuple(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let mut exprs = Vec::new();

        self.expect(Tkt::Lparen)?;

        while self.current.token != Tkt::Rparen {
            exprs.push(self.expr()?); // compiles the argument

            match &self.current.token {
                Tkt::Comma => self.skip(Tkt::Comma)?,
                Tkt::Rparen => break,
                _ => self.throw(format!(
                    "Expected `,`, `)` or other token, found `{}`",
                    &self.current.token
                ))?,
            }
        }

        if exprs.len() == 1 {
            Ok(exprs.pop().unwrap())
        } else {
            Ok(Expr::new(ExprKind::Tuple(exprs), line, column))
        }
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let obj = match self.current.token.clone() {
            Tkt::Num(n) => Expr::new(ExprKind::Lit(Literal::Num(n)), line, column),
            Tkt::Str(s) => Expr::new(ExprKind::Lit(Literal::Str(s)), line, column),
            Tkt::True => Expr::new(ExprKind::Lit(Literal::Bool(true)), line, column),
            Tkt::False => Expr::new(ExprKind::Lit(Literal::Bool(false)), line, column),
            Tkt::Name(s) => Expr::new(ExprKind::Var(s), line, column),
            Tkt::Sym(s) => Expr::new(ExprKind::Lit(Literal::Sym(s)), line, column),
            Tkt::Lbrack => self.list()?,
            Tkt::Lparen => self.tuple()?,
            Tkt::Nil => Expr::new(ExprKind::Lit(Literal::Unit), line, column),
            other => self.throw(format!("unexpected token `{}`", other))?,
        };

        Ok(obj)
    }
}
