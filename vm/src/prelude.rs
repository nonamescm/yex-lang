use crate::{env::Table, list, panic, Constant};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

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

fn create_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(ref filename) => {
            let _ = fs::File::create(filename);
        }
        other => panic!("file_create() expected str, found {}", other),
    }
    Nil
}

fn write_file(args: &[Constant]) -> Constant {
    use Constant::*;

    let content = match &args[0] {
        Str(ref content) => content,
        other => panic!("file_write() expected str, found {}", other),
    };
    match &args[1] {
        Str(ref filename) => {
            fs::write(filename, content).ok();
        }
        other => panic!("file_write()[1] expected str, found {}", other),
    }
    Nil
}

fn getenv(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(ref env_var) => {
            if let Ok(evar) = env::var(env_var) {
                return Str(evar);
            }
        }
        other => panic!("getenv() expected str, found {}", other),
    }
    Nil
}

fn setenv(args: &[Constant]) -> Constant {
    use Constant::*;

    let var = match &args[0] {
        Str(ref var) => var,
        other => panic!("getenv() expected str, found {}", other),
    };

    match &args[0] {
        Str(ref value) => {
            env::set_var(var, value);
        }
        other => panic!("getenv()[1] expected str, found {}", other),
    }

    Nil
}

fn system(args: &[Constant]) -> Constant {
    use Constant::*;
    let mut cmd = match &args[0] {
        Str(ref command) => {
            let mut command_pieces = command.split_whitespace();
            let command = match command_pieces.next() {
                Some(v) => v,
                _ => return Nil,
            };
            Command::new(command)
        }
        other => panic!("system() expected str, found {}", other),
    };

    let args = match &args[0] {
        List(list) => list
            .to_vec()
            .into_iter()
            .map(|it| match it {
                Str(s) => s,
                other => format!("{}", other),
            })
            .collect::<Vec<_>>(),
        other => panic!("system()[1] expected a list, found {}", other),
    };

    if let Ok(out) = cmd.args(&args).output() {
        let stdout = String::from_utf8(out.stdout)
            .unwrap_or_default()
            .trim()
            .to_string();

        let stderr = String::from_utf8(out.stderr)
            .unwrap_or_default()
            .trim()
            .to_string();

        let list = list::List::new();
        let list = list.prepend(Str(stderr));
        let list = list.prepend(Str(stdout));

        return List(list);
    }
    Nil
}

fn exists_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(ref filename) => Bool(fs::File::open(filename).is_ok()),
        other => panic!("file_exists() expected str, found {}", other),
    }
}

fn remove_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(ref filename) => {
            fs::remove_file(filename).ok();
        }
        other => panic!("file_remove() expected str, found {}", other),
    }

    Nil
}
fn read_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(filename) => match fs::read_to_string(filename) {
            Ok(v) => Str(v),
            Err(_) => Nil,
        },
        other => panic!("file_read() str, found {}", other),
    }
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
            Constant::NativeFun { .. } => "native fn",
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
        other => panic!("Expected a string or a symbol, found {}", other),
    };

    match str.parse::<f64>() {
        Ok(n) => Constant::Num(n),
        Err(_) => Constant::Nil,
    }
}

fn get_args(_args: &[Constant]) -> Constant {
    use Constant::*;

    let mut args = list::List::new();
    for i in env::args().into_iter().rev() {
        args = args.prepend(Constant::Str(i.to_owned()));
    }

    List(args)
}

fn str_split(args: &[Constant]) -> Constant {
    use Constant::*;

    let str = match &args[0] {
        Str(ref str) => str,
        other => panic!("split(), expected str, found {}", other),
    };

    let pat = match &args[1] {
        Str(ref pat) => pat,
        other => panic!("split() expected str, found {}", other),
    };

    let splited = str.split(pat).collect::<Vec<&str>>();

    let mut list = list::List::new();
    for i in splited.into_iter().rev() {
        list = list.prepend(Str(i.to_string()));
    }

    List(list)
}

pub fn prelude() -> Table {
    let mut prelude = Table::new();
    macro_rules! insert_fn {
        ($name: expr, $fn: expr) => {
            insert_fn!($name, $fn, 1)
        };
        ($name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Constant::NativeFun {
                    arity: $arity,
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
    insert_fn!("fread", read_file);
    insert_fn!("fwrite", write_file, 2);
    insert_fn!("remove", remove_file);
    insert_fn!("creat", create_file);
    insert_fn!("exists", exists_file);
    insert_fn!("system", system, 2);
    insert_fn!("getargs", get_args);
    insert_fn!("getenv", getenv);
    insert_fn!("setenv", setenv, 2);
    insert_fn!("split", str_split, 2);
    prelude
}
