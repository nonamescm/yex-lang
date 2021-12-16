use crate::{
    env::Table,
    gc::GcRef,
    literal::{ok, Constant},
};
use std::io::Write;
mod io;
mod list;
mod str;

#[macro_export]
#[doc(hidden)]
macro_rules! err_tuple {
    ($($tt:tt)+) => {{
        let msg = format!($($tt)+);
        let mut xs = $crate::List::new();
        xs = xs.prepend(Constant::Str(GcRef::new(msg)));
        xs = xs.prepend($crate::literal::err());
        return Constant::List(GcRef::new(xs));
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! ok_tuple {
    ($reason: expr) => {{
        let mut xs = $crate::List::new();
        xs = xs.prepend($reason);
        xs = xs.prepend($crate::literal::ok());
        Constant::List(GcRef::new(xs))
    }};
}

fn puts(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::Str(s) => println!("{}", s.get()),
        other => println!("{}", other),
    };
    ok()
}

fn print(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::Str(s) => print!("{}", s.get()),
        other => print!("{}", other),
    };
    ok()
}

fn input(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::Str(s) => print!("{}", s.get()),
        other => print!("{}", other),
    };

    if std::io::stdout().flush().is_err() {
        err_tuple!("Error flushing stdout");
    }

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        err_tuple!("Error reading line");
    }

    input.pop();
    ok_tuple!(Constant::Str(GcRef::new(input)))
}

fn str(args: &[Constant]) -> Constant {
    Constant::Str(GcRef::new(format!("{}", &args[0])))
}

fn r#type(args: &[Constant]) -> Constant {
    let type_name = GcRef::new(
        match &args[0] {
            Constant::List(_) => "list",
            Constant::Table(_) => "table",
            Constant::Str(_) => "str",
            Constant::Num(_) => "num",
            Constant::Bool(_) => "bool",
            Constant::Sym(_) => "symbol",
            Constant::Nil => "nil",
            Constant::Fun { .. } => "fn",
        }
        .into(),
    );
    Constant::Str(type_name)
}

fn inspect(args: &[Constant]) -> Constant {
    Constant::Str(GcRef::new(format!("{:#?}", &args[0])))
}

fn int(args: &[Constant]) -> Constant {
    let str = match &args[0] {
        Constant::Sym(symbol) => symbol.to_str(),
        Constant::Str(str) => str.get(),
        other => err_tuple!("Expected a string or a symbol, found {}", other),
    };

    match str.parse::<f64>() {
        Ok(n) => Constant::Num(n),
        Err(e) => err_tuple!("{:?}", e),
    }
}

pub fn prelude() -> Table {
    use {self::str::*, io::*, list::*};

    let mut prelude = Table::new();
    macro_rules! insert_fn {
        ($name: expr, $fn: expr) => {
            insert_fn!($name, $fn, 1)
        };
        ($name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Constant::Fun(GcRef::new(crate::literal::Fun {
                    arity: $arity,
                    args: vec![],
                    body: GcRef::new($crate::Either::Right(|_, it| {
                        $fn(&*it)
                    })),
                })),
            )
        };

        (@vm $name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Constant::Fun(GcRef::new(crate::literal::Fun {
                    arity: $arity,
                    args: vec![],
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

    insert_fn!("fread", read_file);
    insert_fn!("fwrite", write_file, 2);
    insert_fn!("remove", remove_file);
    insert_fn!("creat", create_file);
    insert_fn!("exists", exists_file);
    insert_fn!("system", system, 2);
    insert_fn!("getargs", get_args, 0);
    insert_fn!("getenv", getenv);
    insert_fn!("setenv", setenv, 2);

    insert_fn!(@vm "map", map, 2);
    insert_fn!(@vm "fold", fold, 3);
    insert_fn!("rev", rev, 1);
    insert_fn!("insert", insert, 3);

    insert_fn!("starts_with", starts_with, 2);
    insert_fn!("ends_with", ends_with, 2);
    insert_fn!("replace", replace, 3);
    prelude
}
