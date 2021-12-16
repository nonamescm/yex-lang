use crate::err_tuple;
use crate::ok_tuple;
use crate::{
    gc::GcRef,
    list,
    literal::{nil, ok, symbol::Symbol, Constant},
    table,
};
use std::env;
use std::fs;
use std::process::Command;

pub fn create_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(ref filename) => match fs::File::create(filename.get()) {
            Ok(_) => ok(),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_create() expected str, found {}", other),
    }
}

pub fn write_file(args: &[Constant]) -> Constant {
    use Constant::*;

    let content = match &args[1] {
        Str(ref content) => content.get(),
        other => err_tuple!("file_write()[0] expected str, found {}", other),
    };
    let res = match &args[0] {
        Str(ref filename) => fs::write(filename.get(), content),
        other => err_tuple!("file_write()[1] expected str, found {}", other),
    };
    match res {
        Ok(_) => ok(),
        Err(e) => err_tuple!("{:?}", e.kind()),
    }
}

pub fn getenv(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(env_var) => {
            if let Ok(evar) = env::var(env_var.get()) {
                return Str(GcRef::new(evar));
            }
        }
        other => err_tuple!("getenv() expected str, found {}", other),
    }
    nil()
}

pub fn setenv(args: &[Constant]) -> Constant {
    use Constant::*;

    let var = match &args[0] {
        Str(var) => var,
        other => err_tuple!("getenv() expected str, found {}", other),
    };

    match &args[0] {
        Str(value) => {
            env::set_var(var.get(), value.get());
        }
        other => err_tuple!("getenv()[1] expected str, found {}", other),
    }

    nil()
}

pub fn system(args: &[Constant]) -> Constant {
    use Constant::*;
    let mut cmd = match &args[0] {
        Str(command) => Command::new(command.get()),
        other => err_tuple!("system() expected str, found {}", other),
    };

    let args = match &args[1] {
        List(list) => list
            .iter()
            .map(|it| match it {
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
            let list = list.prepend(Str(GcRef::new(stderr)));
            let list = list.prepend(Str(GcRef::new(stdout)));

            ok_tuple!(List(GcRef::new(list)))
        }
        Err(e) => err_tuple!("{:?}", e.kind()),
    }
}

pub fn exists_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(filename) => Bool(fs::File::open(filename.get()).is_ok()),
        other => err_tuple!("file_exists() expected str, found {}", other),
    }
}

pub fn remove_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(filename) => match fs::remove_file(filename.get()) {
            Ok(_) => ok(),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_remove() expected str, found {}", other),
    }
}

pub fn read_file(args: &[Constant]) -> Constant {
    use Constant::*;

    match &args[0] {
        Str(filename) => match fs::read_to_string(filename.get()) {
            Ok(v) => Str(GcRef::new(v)),
            Err(e) => err_tuple!("{:?}", e.kind()),
        },
        other => err_tuple!("file_read() expected str, found {}", other),
    }
}

pub fn get_args(_: &[Constant]) -> Constant {
    use Constant::*;

    let mut args = list::List::new();
    for i in env::args().into_iter().rev() {
        args = args.prepend(Str(GcRef::new(i.to_owned())));
    }

    List(GcRef::new(args))
}

pub fn read_dir(args: &[Constant]) -> Constant {
    use Constant::*;
    /*
     * name    |  type  | description
     * path    |  str   | path to read
     */

    let path = match &args[0] {
        Str(path) => path.get(),
        other => err_tuple!("readdir() expected str, found {}", other),
    };

    /*
     * Return:
     * Array of tables
     * [{ filename: str, isdir: bool }]
     */
    if !std::path::Path::new(path).is_dir() {
        // returns nil when path is not a dir
        return Constant::Nil;
    }
    let mut dirs: crate::List = list::List::new();
    match fs::read_dir(path) {
        Ok(dir_result) => {
            for file in dir_result {
                let entry = file.unwrap();
                let mut table = table::Table::new();
                table = table.insert(
                    Symbol::new("filename"),
                    Constant::Str(GcRef::new(entry.file_name().into_string().unwrap())),
                );
                table = table.insert(
                    Symbol::new("isdir"),
                    Constant::Bool(entry.metadata().unwrap().is_dir()),
                );
                dirs = dirs.prepend(Constant::Table(GcRef::new(table)));
            }
            Constant::List(GcRef::new(dirs))
        }
        Err(err) => err_tuple!("{:?}", err.kind()),
    }
}

pub fn remove_dir(args: &[Constant]) -> Constant {
    use Constant::*;
    /*
     * name    |  type  | description
     * path    |  str   | path to delete
     */

    let path = match &args[0] {
        Str(path) => path.get(),
        other => err_tuple!("readdir() expected str, found {}", other),
    };

    if !std::path::Path::new(path).is_dir() {
        err_tuple!("NotADirectory");
    }
    match fs::remove_dir(path) {
        Ok(_) => ok(),
        Err(err) => err_tuple!("{:?}", err.kind()),
    }
}

pub fn make_dir(args: &[Constant]) -> Constant {
    use Constant::*;
    /*
     * name    |  type  | description
     * path    |  str   | path to create
     */

    let path = match &args[0] {
        Str(path) => path.get(),
        other => err_tuple!("readdir() expected str, found {}", other),
    };

    if std::path::Path::new(path).exists() {
        return nil();
    }
    match fs::create_dir(path) {
        Ok(_) => ok(),
        Err(err) => err_tuple!("{:?}", err.kind()),
    }
}
