use std::{iter::Peekable, mem::take};

use crate::{
    error::{ParseError, ParseResult},
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};

use self::ast::{Expr, ExprKind, Literal, Stmt, StmtKind, Type, VarDecl};

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
            stmts.push(self.def_bind()?);
        }

        Ok(stmts)
    }

    fn def_bind(&mut self) -> ParseResult<Stmt> {
        self.expect(Tkt::Def)?;
        let line = self.current.line;
        let column = self.current.column;

        let bind = self.var_decl()?;

        self.expect(Tkt::Assign)?;
        let value = self.expr()?;

        Ok(Stmt::new(StmtKind::Def { bind, value }, line, column))
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
                "Expected {}, found {}",
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

    fn peek(&mut self) -> ParseResult<&Token> {
        match self.lexer.peek().unwrap() {
            Ok(t) => Ok(t),
            Err(e) => Err(*e),
        }
    }

    fn type_(&mut self) -> ParseResult<Type> {
        let ty = match take(&mut self.current.token) {
            Tkt::Name(id) => id,
            other => self.throw(format!("Expected type name, found {}", other))?,
        };

        let ty = if self.peek()?.token == Tkt::Lbrack {
            self.next()?;
            self.next()?;

            let args = Some(self.generics()?);
            self.expect(Tkt::Rbrack)?;

            Type { ty, args }
        } else {
            self.next()?;
            Type { ty, args: None }
        };

        Ok(ty)
    }

    fn generics(&mut self) -> ParseResult<Vec<Type>> {
        let mut args = Vec::new();
        loop {
            args.push(self.type_()?);
            if self.current.token == Tkt::Rbrack {
                break;
            }
            self.expect(Tkt::Comma)?;
        }
        Ok(args)
    }

    fn expr(&mut self) -> ParseResult<Expr> {
        let mut expr;

        loop {
            expr = match self.current.token {
                Tkt::Let => self.bind()?,
                Tkt::If => self.condition()?,
                Tkt::Fn => self.function()?,
                _ => self.logic_or()?,
            };

            if self.current.token != Tkt::Seq {
                break;
            }
            self.next()?;
        }

        Ok(expr)
    }

    fn condition(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::If)?;
        let line = self.current.line;
        let column = self.current.column;

        let cond = self.expr()?;

        self.expect(Tkt::Then)?;
        let then = self.expr()?;

        self.expect(Tkt::Else)?;
        let else_ = self.expr()?;

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
        let mut args = vec![];

        while self.current.token == Tkt::Lparen {
            self.next()?;
            args.push(self.var_decl()?);
            self.expect(Tkt::Rparen)?;
        }

        Ok(args)
    }

    fn function(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Fn)?;
        let line = self.current.line;
        let column = self.current.column;

        let args = self.args()?;

        self.expect(Tkt::Colon)?;
        let ty = self.type_()?;

        self.expect(Tkt::Assign)?;

        let body = self.expr()?;

        Ok(Expr::new(
            ExprKind::Lambda {
                args,
                ty,
                body: Box::new(body),
            },
            line,
            column,
        ))
    }

    fn var_decl(&mut self) -> ParseResult<VarDecl> {
        let name = match take(&mut self.current.token) {
            Tkt::Name(id) => id,
            other => self.throw(format!("Expected name, found {}", other))?,
        };

        self.next()?;
        self.expect(Tkt::Colon)?;
        let ty = self.type_()?;

        Ok(VarDecl { name, ty })
    }

    fn bind(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Let)?;
        let line = self.current.line;
        let column = self.current.column;

        let bind = self.var_decl()?;

        self.expect(Tkt::Assign)?;
        let value = self.expr()?;

        self.expect(Tkt::In)?;
        let body = self.expr()?;

        Ok(Expr::new(
            ExprKind::Bind {
                bind,
                value: Box::new(value),
                body: Box::new(body),
            },
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

            let line = left.line;
            let column = left.column;

            left = Expr::new(
                ExprKind::Logic {
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

            let line = left.line;
            let column = left.column;

            left = Expr::new(
                ExprKind::Logic {
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
                ExprKind::Eq {
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
                ExprKind::Cmp {
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
            let right = self.bitwise()?;

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
                ExprKind::Bitwise {
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
                ExprKind::Math {
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
                ExprKind::Math {
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
        if let Tkt::Sub | Tkt::Not | Tkt::Len = &self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.prefix()?;
            Ok(Expr::new(
                ExprKind::UnOp(op.token, Box::new(right)),
                op.line,
                op.column,
            ))
        } else {
            let expr = self.call()?;
            self.next()?;
            Ok(expr)
        }
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let callee = self.primary()?;

        let mut args = Vec::new();
        while self.peek()?.token.is_expression() {
            self.next()?;
            args.push(self.primary()?);
        }

        if args.is_empty() {
            Ok(callee)
        } else {
            let line = callee.line;
            let column = callee.column;

            Ok(Expr::new(
                ExprKind::App {
                    callee: Box::new(callee),
                    args,
                },
                line,
                column,
            ))
        }
    }

    fn list(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

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

    fn primary(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let expr = match self.current.token.clone() {
            Tkt::Num(n) => Expr::new(ExprKind::Lit(Literal::Num(n)), line, column),
            Tkt::Str(s) => Expr::new(ExprKind::Lit(Literal::Str(s)), line, column),
            Tkt::True => Expr::new(ExprKind::Lit(Literal::Bool(true)), line, column),
            Tkt::False => Expr::new(ExprKind::Lit(Literal::Bool(false)), line, column),
            Tkt::Name(s) => Expr::new(ExprKind::Var(s), line, column),
            Tkt::Sym(s) => Expr::new(ExprKind::Lit(Literal::Sym(s)), line, column),
            Tkt::Lbrack => {
                self.next()?;
                self.list()?
            }
            Tkt::Lparen => {
                self.next()?;
                let expr = self.expr()?;
                self.assert(Tkt::Rparen)?;
                expr
            }
            Tkt::Nil => Expr::new(ExprKind::Lit(Literal::Unit), line, column),
            other => self.throw(format!("unexpected token {}", other))?,
        };
        Ok(expr)
    }
}
