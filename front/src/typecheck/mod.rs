use crate::{
    error::ParseResult,
    parser::ast::{Expr, ExprKind, Stmt, StmtKind, Type},
    tokens::TokenType as Tkt,
    ParseError,
};
use std::collections::HashMap;
use vm::Symbol;

#[derive(Clone)]
pub struct Context {
    pub vars: HashMap<Symbol, Type>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub fn throw<T>(node: &Expr, message: impl Into<String>) -> ParseResult<T> {
    ParseError::throw(node.line, node.column, message.into())
}

pub fn assert_type(ctx: &Context, node: &Expr, ty: &Type) -> ParseResult<()> {
    let typ = typecheck(ctx, node)?;

    if &typ == ty {
        Ok(())
    } else {
        throw(
            &node,
            format!(
                "This expression was expected to have type, {}, but here it has type {}",
                ty, typ
            ),
        )
    }
}

pub fn typecheck(ctx: &Context, node: &Expr) -> ParseResult<Type> {
    match &node.kind {
        ExprKind::Var(name) => match ctx.vars.get(&name) {
            Some(ty) => Ok(ty.clone()),
            None => throw(node, format!("Unknown variable {name}")),
        },

        ExprKind::Lit(lit) => Ok(lit.type_of()),

        ExprKind::Eq { left, right, .. } => {
            let ty = typecheck(ctx, &left)?;
            assert_type(ctx, &right, &ty)?;
            Ok(Type::bool())
        }

        ExprKind::If { cond, then, else_ } => {
            // asserts that the condition is a boolean
            assert_type(ctx, &cond, &Type::bool())?;

            let ty = typecheck(ctx, &then)?;
            assert_type(ctx, &else_, &ty)?;

            Ok(ty)
        }

        ExprKind::UnOp(op, expr) => {
            let ty = typecheck(ctx, &expr)?;
            match op {
                Tkt::Not => assert_type(ctx, &expr, &Type::bool())?,
                Tkt::Sub => assert_type(ctx, &expr, &Type::num())?,
                _ => unreachable!(),
            }
            Ok(ty)
        }

        ExprKind::Cons { head, tail } => {
            let ty = typecheck(ctx, &head)?;
            assert_type(ctx, &tail, &Type::list(ty.clone()))?;
            Ok(Type::list(ty))
        }

        ExprKind::Math { left, right, .. } | ExprKind::Bitwise { left, right, .. } => {
            let ty = Type::num();
            assert_type(ctx, left, &ty)?;
            assert_type(ctx, right, &ty)?;

            Ok(ty)
        }

        ExprKind::Cmp { left, right, .. } => {
            let ty = Type::num();
            assert_type(ctx, left, &ty)?;
            assert_type(ctx, right, &ty)?;

            Ok(Type::bool())
        }

        ExprKind::Logic { left, right, .. } => {
            let ty = Type::bool();
            assert_type(ctx, left, &ty)?;
            assert_type(ctx, right, &ty)?;

            Ok(ty)
        }

        ExprKind::List(xs) => {
            let ty = typecheck(ctx, &xs[0])?; // TODO: add support for empty lists
            for item in xs.iter().skip(1) {
                assert_type(ctx, item, &ty)?;
            }
            Ok(Type::list(ty))
        }

        ExprKind::Bind { bind, value, body } => {
            let ty = &bind.ty;
            let mut ctx = ctx.clone();
            ctx.vars.insert(bind.name, bind.ty.clone());

            assert_type(&ctx, value, ty)?;
            typecheck(&ctx, body)
        }

        ExprKind::Lambda {
            args,
            ret,
            ty,
            body,
        } => {
            let mut ctx = ctx.clone();
            for arg in args.iter() {
                ctx.vars.insert(arg.name, arg.ty.clone());
            }

            assert_type(&ctx, body, ret)?;

            Ok(Type::Fn(ty.clone()))
        }

        ExprKind::Seq { left, .. } => typecheck(ctx, left),

        ExprKind::App { callee, args } => typecheck_app(ctx, callee, args),
    }
}

fn typecheck_app(ctx: &Context, callee: &Expr, args: &[Expr]) -> ParseResult<Type> {
    let ty = typecheck(ctx, callee)?;

    match ty {
        Type::Fn(ret) if args.len() != ret.0.len() - 1 => throw(
            callee,
            format!("Expected {} arguments, but got {}", ret.0.len(), args.len()),
        ),
        Type::Fn(ret) => {
            for (arg, ty) in args.iter().zip(ret.0.iter()) {
                assert_type(ctx, arg, ty)?;
            }
            Ok(Type::Fn(ret))
        }
        _ => throw(callee, format!("Expected a function type, found {}", ty)),
    }
}

pub fn typecheck_stmt(ctx: &Context, def: &Stmt) -> ParseResult<()> {
    match &def.kind {
        StmtKind::Def { bind, value } => {
            let ty = &bind.ty;
            let mut ctx = ctx.clone();
            ctx.vars.insert(bind.name, bind.ty.clone());

            assert_type(&ctx, &value, ty)?;
        }

        _ => todo!(),
    }
    Ok(())
}
