use crate::{env::Table, Constant};
use std::io::{self, Write};

fn puts(args: &[Constant]) -> Constant {
    match args[0] {
        Constant::Str(ref s) => println!("{}", s),
        ref other => println!("{}", other),
    };
    Constant::Nil
}

fn print(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::Str(s) => print!("{}", s),
        other => print!("{}", other),
    };
    Constant::Nil
}

fn input(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::Str(s) => print!("{}", s),
        other => print!("{}", other),
    };

    if io::stdout().flush().is_err() {
        panic!("Error flushing stdout")
    }
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        panic!("Error reading line")
    }
    input.pop();
    Constant::Str(input)
}

fn head(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::List(xs) => match xs.head() {
            Some(x) => x.clone(),
            None => Constant::Nil,
        },
        other => panic!("head() expected a list, found {}", other),
    }
}

fn tail(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::List(xs) => Constant::List(xs.tail()),
        other => panic!("tail() expected a list, found {}", other),
    }
}

fn str(args: &[Constant]) -> Constant {
    Constant::Str(format!("{}", &args[0]))
}

fn r#type(args: &[Constant]) -> Constant {
    Constant::Str(
        match &args[0] {
            Constant::List(_) => "list",
            Constant::Str(_) => "str",
            Constant::Num(_) => "num",
            Constant::Bool(_) => "bool",
            Constant::Sym(_) => "symbol",
            Constant::Nil => "nil",
            Constant::Fun { .. } => "fn",
            Constant::PartialFun { .. } => "fn",
            Constant::NativeFun { .. } => "fn",
        }
        .into(),
    )
}

fn inspect(args: &[Constant]) -> Constant {
    Constant::Str(format!("{:#?}", &args[0]))
}

fn int(args: &[Constant]) -> Constant {
    let str = match &args[0] {
        Constant::Sym(symbol) => symbol.to_str(),
        Constant::Str(ref str) => str,
        other => crate::panic!("Expected a string or a symbol, found {}", other)
    };

    match str.parse::<f64>() {
        Ok(n) => Constant::Num(n),
        Err(_) => Constant::Nil,
    }
}

pub fn prelude() -> Table {
    let mut prelude = Table::new();
    macro_rules! insert_fn {
        ($name: expr, $fn: expr) => {
            insert_fn!($name, $fn, arity: 1)
        };
        ($name: expr, $fn: expr, arity: $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Constant::NativeFun {
                    arity: 1,
                    fp: |it| $fn(&*it),
                },
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

    prelude
}
