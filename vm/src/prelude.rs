use crate::{
    env::Table,
    literal::{symbol::Symbol, Constant},
    opcode::{OpCode, OpCodeMetadata},
};
use std::io::{self, Write};

pub fn prelude() -> Table {
    let mut prelude = Table::new();

    macro_rules! vecop {
            ($($elems: expr),*) => {{
                let mut vec = Vec::new();
                $({
                    vec.push($elems)
                })*;
                vec.into_iter().map(|opcode| OpCodeMetadata {
                    line: 0,
                    column: 0,
                    opcode,
                }
                ).collect::<Vec<_>>()
            }}
        }

    macro_rules! nativefn {
        ($closure: expr) => {
            Constant::Fun {
                arity: 1,
                body: vecop![OpCode::Cnll($closure)],
            }
        };
    }

    prelude.insert(
        Symbol::new("puts"),
        nativefn!(|c| {
            match c {
                Constant::Str(s) => println!("{}", s),
                other => println!("{}", other),
            };
            Constant::Nil
        }),
    );

    prelude.insert(
        Symbol::new("print"),
        nativefn!(|c| {
            match c {
                Constant::Str(s) => print!("{}", s),
                other => print!("{}", other),
            };
            Constant::Nil
        }),
    );

    prelude.insert(
        Symbol::new("input"),
        nativefn!(|c| {
            match c {
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
        }),
    );

    prelude.insert(
        Symbol::new("head"),
        nativefn!(|xs| {
            match xs {
                Constant::List(xs) => match xs.head() {
                    Some(x) => x.clone(),
                    None => Constant::Nil,
                },
                other => panic!("head() expected a list, found {}", other),
            }
        }),
    );

    prelude.insert(
        Symbol::new("tail"),
        nativefn!(|xs| {
            match xs {
                Constant::List(xs) => Constant::List(xs.tail()),
                other => panic!("tail() expected a list, found {}", other),
            }
        }),
    );

    prelude.insert(
        Symbol::new("str"),
        nativefn!(|xs| Constant::Str(format!("{}", xs))),
    );

    prelude.insert(
        Symbol::new("type"),
        nativefn!(|it| Constant::Str(
            match it {
                Constant::List(_) => "list",
                Constant::Str(_) => "str",
                Constant::Num(_) => "num",
                Constant::Bool(_) => "bool",
                Constant::Sym(_) => "symbol",
                Constant::Nil => "nil",
                Constant::Fun { .. } => "fn",
                Constant::PartialFun { .. } => "fn",
            }
            .into()
        )),
    );

    prelude.insert(
        Symbol::new("inspect"),
        nativefn!(|it| { Constant::Str(format!("{:#?}", it)) }),
    );
    prelude
}
