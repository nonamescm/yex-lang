use crate::{
    env::EnvTable,
    gc::GcRef,
    literal::{nil, Constant},
    stackvec, InterpretResult,
};
use std::io::Write;
mod ffi;
mod list;
mod str;

fn puts(args: &[Constant]) -> InterpretResult<Constant> {
    match &args[0] {
        Constant::Str(s) => println!("{}", s.get()),
        other => println!("{}", other),
    };
    Ok(nil())
}

fn print(args: &[Constant]) -> InterpretResult<Constant> {
    match &args[0] {
        Constant::Str(s) => print!("{}", s.get()),
        other => print!("{}", other),
    };
    Ok(nil())
}

fn input(args: &[Constant]) -> InterpretResult<Constant> {
    match &args[0] {
        Constant::Str(s) => print!("{}", s.get()),
        other => print!("{}", other),
    };

    if std::io::stdout().flush().is_err() {
        panic!("Error flushing stdout");
    }

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        panic!("Error reading line");
    }

    input.pop();
    Ok(Constant::Str(GcRef::new(input)))
}

fn str(args: &[Constant]) -> InterpretResult<Constant> {
    Ok(Constant::Str(GcRef::new(format!("{}", &args[0]))))
}

fn r#type(args: &[Constant]) -> InterpretResult<Constant> {
    let type_name = GcRef::new(
        match &args[0] {
            Constant::List(_) => "list",
            Constant::Table(_) => "table",
            Constant::Str(_) => "str",
            Constant::Num(_) => "num",
            Constant::Bool(_) => "bool",
            Constant::Sym(_) => "symbol",
            Constant::Nil => "nil",
            Constant::ExternalFunction(_) | Constant::ExternalFunctionNoArg(_) => "extern fn",
            Constant::Fun { .. } => "fn",
        }
        .into(),
    );
    Ok(Constant::Str(type_name))
}

fn inspect(args: &[Constant]) -> InterpretResult<Constant> {
    Ok(Constant::Str(GcRef::new(format!("{:#?}", &args[0]))))
}

fn get_os(_args: &[Constant]) -> InterpretResult<Constant> {
    Ok(Constant::Str(GcRef::new(std::env::consts::OS.to_string())))
}

fn int(args: &[Constant]) -> InterpretResult<Constant> {
    let str = match &args[0] {
        Constant::Sym(symbol) => symbol.to_str(),
        Constant::Str(str) => str.get(),
        other => panic!("Expected a string or a symbol, found {}", other),
    };

    match str.parse::<f64>() {
        Ok(n) => Ok(Constant::Num(n)),
        Err(e) => panic!("{:?}", e),
    }
}

pub fn prelude() -> EnvTable {
    use {self::str::*, ffi::*, list::*};

    let mut prelude = EnvTable::with_capacity(64);
    macro_rules! insert_fn {
        ($name: expr, $fn: expr) => {
            insert_fn!($name, $fn, 1)
        };
        ($name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Constant::Fun(GcRef::new(crate::literal::Fun {
                    arity: $arity,
                    args: stackvec![],
                    body: GcRef::new($crate::Either::Right(|_, it| $fn(&*it))),
                })),
            )
        };

        (@vm $name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Constant::Fun(GcRef::new(crate::literal::Fun {
                    arity: $arity,
                    args: stackvec![],
                    body: GcRef::new($crate::Either::Right(|vm, it| {
                        $fn(unsafe { vm.as_mut().unwrap() }, &*it)
                    })),
                })),
            )
        };
    }

    insert_fn!("puts", puts);
    insert_fn!("print", print);
    insert_fn!("input", input);
    insert_fn!("head", head);
    insert_fn!("tail", tail);
    insert_fn!("str", str);
    insert_fn!("type", r#type);
    insert_fn!("inspect", inspect);
    insert_fn!("int", int);
    insert_fn!("split", str_split, 2);

    insert_fn!(@vm "map", map, 2);
    insert_fn!(@vm "fold", fold, 3);
    insert_fn!("rev", rev, 1);
    insert_fn!("insert", insert, 3);

    insert_fn!("starts_with", starts_with, 2);
    insert_fn!("ends_with", ends_with, 2);
    insert_fn!("replace", replace, 3);

    insert_fn!("getos", get_os, 0);

    insert_fn!(@vm "dlopen", dlopen, 4);
    insert_fn!(@vm "dlclose", dlclose, 1);

    prelude
}
