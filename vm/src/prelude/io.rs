use crate::err_tuple;

use crate::{
    gc::GcRef,
    list,
    literal::{nil, ConstantRef, ok},
    Constant,
};
use std::env;
use std::fs;
use std::process::Command;

pub fn create_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(ref filename) => match fs::File::create(filename) {
            Ok(_) => ok(),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_create() expected str, found {}", other),
    }
}

pub fn write_file(args: &[ConstantRef]) -> ConstantRef {
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
        Err(e) => err_tuple!("{:?}", e.kind()),
    }
}

pub fn getenv(args: &[ConstantRef]) -> ConstantRef {
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

pub fn setenv(args: &[ConstantRef]) -> ConstantRef {
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

pub fn system(args: &[ConstantRef]) -> ConstantRef {
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

            GcRef::new(List(list))
        }
        Err(e) => err_tuple!("{:?}", e.kind()),
    }
}

pub fn exists_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => GcRef::new(Bool(fs::File::open(filename).is_ok())),
        other => err_tuple!("file_exists() expected str, found {}", other),
    }
}

pub fn remove_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => match fs::remove_file(filename) {
            Ok(_) => ok(),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_remove() expected str, found {}", other),
    }
}

pub fn read_file(args: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    match args[0].get() {
        Str(filename) => match fs::read_to_string(filename) {
            Ok(v) => GcRef::new(Str(v)),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_read() expected str, found {}", other),
    }
}

pub fn get_args(_: &[ConstantRef]) -> ConstantRef {
    use Constant::*;

    let mut args = list::List::new();
    for i in env::args().into_iter().rev() {
        args = args.prepend(GcRef::new(Constant::Str(i.to_owned())));
    }

    GcRef::new(List(args))
}
