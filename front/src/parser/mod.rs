use std::{iter::Peekable, mem::take};

use crate::{
    error::{ParseError, ParseResult},
    lexer::Lexer,
    tokens::{Token, TokenType as Tkt},
};

use self::ast::{Expr, ExprKind, Literal, Stmt, StmtKind, VarDecl};

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

    pub fn parse_expr(mut self) -> ParseResult<Expr> {
        self.expr()
    }

    fn def_bind(&mut self) -> ParseResult<Stmt> {
        self.expect(Tkt::Def)?;

        if self.peek()?.token == Tkt::Lparen {
            return self.def_fn();
        }

        let line = self.current.line;
        let column = self.current.column;

        let bind = self.var_decl()?;

        self.expect(Tkt::Assign)?;
        let value = self.expr()?;

        Ok(Stmt::new(StmtKind::Def { bind, value }, line, column))
    }

    fn def_fn(&mut self) -> ParseResult<Stmt> {
        let line = self.current.line;
        let column = self.current.column;

        let name = match take(&mut self.current.token) {
            Tkt::Name(id) => id,
            other => self.throw(format!("Expected name, found {}", other))?,
        };

        self.next()?;
        let value = self.function()?;
        let bind = VarDecl::new(name);

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

    fn expr(&mut self) -> ParseResult<Expr> {
        let mut expr;

        loop {
            expr = match self.current.token {
                Tkt::Let => self.bind()?,
                Tkt::If => self.condition()?,
                Tkt::Fn => self.fn_()?,
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

    fn fn_(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Fn)?;
        self.function()
    }

    fn function(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let args = self.args()?;

        self.expect(Tkt::FatArrow)?;

        let body = self.expr()?;

        Ok(Expr::new(
            ExprKind::Lambda {
                args,
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

        Ok(VarDecl::new(name))
    }

    fn bind_fn(&mut self) -> ParseResult<Expr> {
        let line = self.current.line;
        let column = self.current.column;

        let name = match take(&mut self.current.token) {
            Tkt::Name(id) => id,
            other => self.throw(format!("Expected name, found {}", other))?,
        };

        self.next()?;
        let value = self.function()?;

        self.expect(Tkt::In)?;
        let body = self.expr()?;

        Ok(Expr::new(
            ExprKind::Bind {
                bind: VarDecl::new(name),
                value: Box::new(value),
                body: Box::new(body),
            },
            line,
            column,
        ))
    }

    fn bind(&mut self) -> ParseResult<Expr> {
        self.expect(Tkt::Let)?;
        if self.peek()?.token == Tkt::Lparen {
            return self.bind_fn();
        }

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

            let line = left.line;
            let column = left.column;

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
        if let Tkt::Sub | Tkt::Not | Tkt::Len = &self.current.token {
            let op = take(&mut self.current);
            self.next()?;
            let right = self.prefix()?;
            Ok(Expr::new(
                ExprKind::UnOp(op.token.try_into().unwrap(), Box::new(right)),
                op.line,
                op.column,
            ))
        } else {
            let expr = self.call()?;
            self.next()?;
            Ok(expr)
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

        Ok(args)
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let callee = self.primary()?;

        let args = if self.peek()?.token == Tkt::Lparen {
            self.next()?;
            Some(self.call_args()?)
        } else {
            None
        };

        if let Some(args) = args {
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
        } else {
            Ok(callee)
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
