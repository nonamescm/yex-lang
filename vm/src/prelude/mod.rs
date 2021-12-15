use crate::{
    env::Table,
    gc::GcRef,
    literal::{ok, ConstantRef},
    Constant,
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
        xs = xs.prepend(GcRef::new(Constant::Str(msg)));
        xs = xs.prepend($crate::literal::err());
        return GcRef::new(Constant::List(xs))
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! ok_tuple {
    ($reason: expr) => {{
        let mut xs = $crate::List::new();
        xs = xs.prepend($reason);
        xs = xs.prepend($crate::literal::ok());
        GcRef::new(Constant::List(xs))
    }};
}

fn puts(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::Str(s) => println!("{}", s),
        other => println!("{}", other),
    };
    ok()
}

fn print(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::Str(s) => print!("{}", s),
        other => print!("{}", other),
    };
    ok()
}

fn input(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::Str(s) => print!("{}", s),
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
    ok_tuple!(GcRef::new(Constant::Str(input)))
}

fn str(args: &[ConstantRef]) -> ConstantRef {
    GcRef::new(Constant::Str(format!("{}", args[0].get())))
}

fn r#type(args: &[ConstantRef]) -> ConstantRef {
    let type_name = Constant::Str(
        match args[0].get() {
            Constant::List(_) => "list",
            Constant::Str(_) => "str",
            Constant::Num(_) => "num",
            Constant::Bool(_) => "bool",
            Constant::Sym(_) => "symbol",
            Constant::Nil => "nil",
            Constant::Fun { .. } => "fn",
        }
        .into(),
    );
    GcRef::new(type_name)
}

fn inspect(args: &[ConstantRef]) -> ConstantRef {
    GcRef::new(Constant::Str(format!("{:#?}", &args[0])))
}

fn int(args: &[ConstantRef]) -> ConstantRef {
    let str = match args[0].get() {
        Constant::Sym(symbol) => symbol.to_str(),
        Constant::Str(str) => str,
        other => err_tuple!("Expected a string or a symbol, found {}", other),
    };

    match str.parse::<f64>() {
        Ok(n) => GcRef::new(Constant::Num(n)),
        Err(e) => err_tuple!("{:?}", e),
    }
}

pub fn prelude() -> Table {
    use {io::*, list::*, self::str::*};

    let mut prelude = Table::new();
    macro_rules! insert_fn {
        ($name: expr, $fn: expr) => {
            insert_fn!($name, $fn, 1)
        };
        ($name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                $crate::GcRef::new(Constant::Fun {
                    arity: $arity,
                    args: vec![],
                    body: GcRef::new($crate::Either::Right(|_, it| $fn(&*it))),
                }),
            )
        };

        (@vm $name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                $crate::GcRef::new(Constant::Fun {
                    arity: $arity,
                    args: vec![],
                    body: GcRef::new($crate::Either::Right(|vm, it| {
                        $fn(unsafe { vm.as_mut().unwrap() }, &*it)
                    })),
                }),
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
    insert_fn!("starts_with", starts_with, 2);
    insert_fn!("ends_with", ends_with, 2);
    insert_fn!("replace", replace, 3);
    prelude
}
