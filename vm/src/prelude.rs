use crate::{
    env::Table,
    gc::GcRef,
    list::{self, List},
    literal::{nil, ConstantRef},
    Constant, VirtualMachine,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

fn ok() -> ConstantRef {
    GcRef::new(Constant::Sym(crate::Symbol::new("ok")))
}

fn err() -> ConstantRef {
    GcRef::new(Constant::Sym(crate::Symbol::new("err")))
}

macro_rules! tuple {
    (@err $reason: expr) => {{
        let mut xs = $crate::List::new();
        xs = xs.prepend(GcRef::new(Constant::Str($reason.into())));
        xs = xs.prepend(err());
        GcRef::new(Constant::List(xs))
    }};

    (@ok $reason: expr) => {{
        let mut xs = $crate::List::new();
        xs = xs.prepend($reason);
        xs = xs.prepend(ok());
        GcRef::new(Constant::List(xs))
    }};
}

macro_rules! err_tuple {
    ($($tt:tt)+) => {{
        let msg = format!($($tt)+);
        return tuple!(@err msg)
    }}
}

macro_rules! ok_tuple {
    ($ret:expr) => {
        return tuple!(@ok $ret)
    };
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

    if io::stdout().flush().is_err() {
        err_tuple!("Error flushing stdout");
    }

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        err_tuple!("Error reading line");
    }

    input.pop();
    ok_tuple!(GcRef::new(Constant::Str(input)))
}

fn head(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::List(xs) => match xs.head() {
            Some(x) => x,
            None => nil(),
        },
        other => err_tuple!("head() expected a list, found {}", other),
    }
}

fn tail(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::List(xs) => GcRef::new(Constant::List(xs.tail())),
        other => err_tuple!("tail() expected a list, found {}", other),
    }
}

fn str(args: &[ConstantRef]) -> ConstantRef {
    GcRef::new(Constant::Str(format!("{}", args[0].get())))
}

fn create_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(ref filename) => match fs::File::create(filename) {
            Ok(_) => ok(),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_create() expected str, found {}", other),
    }
}

fn write_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let content = match args[1].get() {
        Str(ref content) => content,
        other => err_tuple!("file_write()[0] expected str, found {}", other),
    };
    let res = match args[0].get() {
        Str(ref filename) => fs::write(filename, content),
        other => err_tuple!("file_write()[1] expected str, found {}", other),
    };
    match res {
        Ok(_) => ok(),
        Err(e) => err_tuple!("{:?}", e),
    }
}

fn getenv(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(env_var) => {
            if let Ok(evar) = env::var(env_var) {
                return GcRef::new(Str(evar));
            }
        }
        other => err_tuple!("getenv() expected str, found {}", other),
    }
    nil()
}

fn setenv(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let var = match args[0].get() {
        Str(var) => var,
        other => err_tuple!("getenv() expected str, found {}", other),
    };

    match args[0].get() {
        Str(value) => {
            env::set_var(var, value);
        }
        other => err_tuple!("getenv()[1] expected str, found {}", other),
    }

    nil()
}

fn system(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;
    let mut cmd = match args[0].get() {
        Str(command) => Command::new(command),
        other => err_tuple!("system() expected str, found {}", other),
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
        other => err_tuple!("system()[1] expected a list, found {}", other),
    };

    match cmd.args(&args).output() {
        Ok(out) => {
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
        Err(e) => err_tuple!("{:?}", e),
    }
}

fn exists_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => GcRef::new(Bool(fs::File::open(filename).is_ok())),
        other => err_tuple!("file_exists() expected str, found {}", other),
    }
}

fn remove_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => match fs::remove_file(filename) {
            Ok(_) => ok(),
            Err(e) => err_tuple!("{:?}", e),
        },
        other => err_tuple!("file_remove() expected str, found {}", other),
    }
}
fn read_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => match fs::read_to_string(filename) {
            Ok(v) => GcRef::new(Str(v)),
            Err(e) => err_tuple!("{:?}", e),
        },
        other => err_tuple!("file_read() expected str, found {}", other),
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
        other => err_tuple!("Expected a string or a symbol, found {}", other),
    };

    match str.parse::<f64>() {
        Ok(n) => GcRef::new(Constant::Num(n)),
        Err(e) => err_tuple!("{:?}", e),
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
        other => err_tuple!("split() expected str, found {}", other),
    };

    let pat = match args[1].get() {
        Str(pat) => pat,
        other => err_tuple!("split() expected str, found {}", other),
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
        other => err_tuple!("map[1] expected a list, but found `{}`", other),
    };

    let xs = xs
        .iter()
        .map(|it| {
            vm.push_gc_ref(it);
            vm.push_gc_ref(GcRef::clone(&fun));
            if let Err(e) = vm.call(1) {
                err_tuple!("{}", e)
            }
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
        other => err_tuple!("fold[2] expected a list, but found `{}`", other),
    };

    for it in xs.iter() {
        vm.push_gc_ref(acc);
        vm.push_gc_ref(it);
        vm.push_gc_ref(GcRef::clone(&fun));
        if let Err(e) = vm.call(1) {
            err_tuple!("{}", e)
        }
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
        other => err_tuple!("starts_with()[0] expected str, found {}", other),
    };
    let pattern = match args[1].get() {
        Str(pat) => pat,
        other => err_tuple!("starts_with()[1] expected str, found {}", other),
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
        other => err_tuple!("ends_with() expected str, found {}", other),
    };
    let pattern = match args[1].get() {
        Str(pat) => pat,
        other => err_tuple!("ends_with() expected str, found {}", other),
    };
    GcRef::new(Bool(str.ends_with(pattern)))
}

fn rev(args: &[ConstantRef]) -> ConstantRef {
    let xs = match args[2].get() {
        Constant::List(xs) => xs,
        other => err_tuple!("rev[0] expected a list, but found `{}`", other),
    };
    GcRef::new(Constant::List(xs.rev()))
}

fn replace(args: &[ConstantRef]) -> ConstantRef {
    let str = match args[0].get() {
        Constant::Str(str) => str,
        other => err_tuple!("replace()[0] expected a str, but found `{}`", other),
    };
    let s_match = match args[1].get() {
        Constant::Str(str) => str,
        other => err_tuple!("replace()[1] expected a str, but found `{}`", other),
    };

    let s_match2 = match args[2].get() {
        Constant::Str(str) => str,
        other => err_tuple!("replace()[2] expected a str, but found `{}`", other),
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
