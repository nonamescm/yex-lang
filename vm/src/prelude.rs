use crate::{
    env::Table,
    gc::GcRef,
    list::{self, List},
    literal::{nil, ConstantRef},
    panic, Constant, VirtualMachine,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

fn puts(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::Str(s) => println!("{}", s),
        other => println!("{}", other),
    };
    nil()
}

fn print(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::Str(s) => print!("{}", s),
        other => print!("{}", other),
    };
    nil()
}

fn input(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
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
    GcRef::new(Constant::Str(input))
}

fn head(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::List(xs) => match xs.head() {
            Some(x) => x,
            None => nil(),
        },
        other => panic!("head() expected a list, found {}", other),
    }
}

fn tail(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::List(xs) => GcRef::new(Constant::List(xs.tail())),
        other => panic!("tail() expected a list, found {}", other),
    }
}

fn str(args: &[ConstantRef]) -> ConstantRef {
    GcRef::new(Constant::Str(format!("{}", args[0].get())))
}

fn create_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(ref filename) => {
            fs::File::create(filename).ok();
        }
        other => panic!("file_create() expected str, found {}", other),
    }
    nil()
}

fn write_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let content = match args[1].get() {
        Str(ref content) => content,
        other => panic!("file_write() expected str, found {}", other),
    };
    match args[0].get() {
        Str(ref filename) => {
            fs::write(filename, content).ok();
        }
        other => panic!("file_write()[1] expected str, found {}", other),
    }
    nil()
}

fn getenv(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(env_var) => {
            if let Ok(evar) = env::var(env_var) {
                return GcRef::new(Str(evar));
            }
        }
        other => panic!("getenv() expected str, found {}", other),
    }
    nil()
}

fn setenv(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let var = match args[0].get() {
        Str(var) => var,
        other => panic!("getenv() expected str, found {}", other),
    };

    match args[0].get() {
        Str(value) => {
            env::set_var(var, value);
        }
        other => panic!("getenv()[1] expected str, found {}", other),
    }

    nil()
}

fn system(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;
    let mut cmd = match args[0].get() {
        Str(command) => {
            let mut command_pieces = command.split_whitespace();
            let command = match command_pieces.next() {
                Some(v) => v,
                _ => return nil(),
            };
            Command::new(command)
        }
        other => panic!("system() expected str, found {}", other),
    };

    let args = match args[1].get() {
        List(list) => list
            .to_vec()
            .into_iter()
            .map(|it| match it.get() {
                Str(s) => s.to_string(),
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
        let list = list.prepend(GcRef::new(Str(stderr)));
        let list = list.prepend(GcRef::new(Str(stdout)));

        return GcRef::new(List(list));
    }
    nil()
}

fn exists_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => GcRef::new(Bool(fs::File::open(filename).is_ok())),
        other => panic!("file_exists() expected str, found {}", other),
    }
}

fn remove_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => {
            fs::remove_file(filename).ok();
        }
        other => panic!("file_remove() expected str, found {}", other),
    }

    nil()
}
fn read_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => match fs::read_to_string(filename) {
            Ok(v) => GcRef::new(Str(v)),
            Err(_) => nil(),
        },
        other => panic!("file_read() str, found {}", other),
    }
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
        other => panic!("Expected a string or a symbol, found {}", other),
    };

    match str.parse::<f64>() {
        Ok(n) => GcRef::new(Constant::Num(n)),
        Err(_) => nil(),
    }
}

fn get_args(_: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let mut args = list::List::new();
    for i in env::args().into_iter().rev() {
        args = args.prepend(GcRef::new(Constant::Str(i.to_owned())));
    }

    GcRef::new(List(args))
}

fn str_split(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let str = match args[0].get() {
        Str(str) => str,
        other => panic!("split() expected str, found {}", other),
    };

    let pat = match args[1].get() {
        Str(pat) => pat,
        other => panic!("split() expected str, found {}", other),
    };

    let mut list = list::List::new();
    for i in str.rsplit(pat) {
        list = list.prepend(GcRef::new(Str(i.to_string())));
    }

    GcRef::new(List(list))
}

fn map(vm: &mut VirtualMachine, args: &[ConstantRef]) -> ConstantRef {
    let fun = GcRef::clone(&args[0]);
    let xs = match args[1].get() {
        Constant::List(xs) => xs,
        other => panic!("map[1] expected a list, but found `{}`", other),
    };

    let xs = xs
        .iter()
        .map(|it| {
            vm.push_gc_ref(it);
            vm.push_gc_ref(GcRef::clone(&fun));
            vm.call(1);
            vm.pop()
        })
        .collect::<List>();

    GcRef::new(Constant::List(xs.rev()))
}

fn fold(vm: &mut VirtualMachine, args: &[ConstantRef]) -> ConstantRef {
    let mut acc = GcRef::clone(&args[0]);
    let fun = GcRef::clone(&args[1]);
    let xs = match args[2].get() {
        Constant::List(xs) => xs,
        other => panic!("fold[2] expected a list, but found `{}`", other),
    };

    for it in xs.iter() {
        vm.push_gc_ref(acc);
        vm.push_gc_ref(it);
        vm.push_gc_ref(GcRef::clone(&fun));
        vm.call(2);
        acc = vm.pop()
    }

    acc
}

fn starts_with(args: &[ConstantRef]) -> ConstantRef {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match args[0].get() {
        Str(string) => string,
        other => panic!("starts_with() expected str, found {}", other),
    };
    let pattern = match args[1].get() {
        Str(pat) => pat,
        other => panic!("starts_with() expected str, found {}", other),
    };
    GcRef::new(Bool(str.starts_with(pattern)))
}

fn ends_with(args: &[ConstantRef]) -> ConstantRef {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match args[0].get() {
        Str(string) => string,
        other => panic!("ends_with() expected str, found {}", other),
    };
    let pattern = match args[1].get() {
        Str(pat) => pat,
        other => panic!("ends_with() expected str, found {}", other),
    };
    GcRef::new(Bool(str.ends_with(pattern)))
}

fn rev(args: &[ConstantRef]) -> ConstantRef {
    let xs = match args[2].get() {
        Constant::List(xs) => xs,
        other => panic!("rev[0] expected a list, but found `{}`", other),
    };
    GcRef::new(Constant::List(xs.rev()))
}
fn replace(args: &[ConstantRef]) -> ConstantRef {
    let str = match args[0].get() {
        Constant::Str(str) => str,
        other => panic!("replace()[0] expected a str, but found `{}`", other),
    };
    let s_match = match args[1].get() {
        Constant::Str(str) => str,
        other => panic!("replace()[1] expected a str, but found `{}`", other),
    };

    let s_match2 = match args[2].get() {
        Constant::Str(str) => str,
        other => panic!("replace()[2] expected a str, but found `{}`", other),
    };

    GcRef::new(Constant::Str(str.replace(s_match, s_match2)))
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
