use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{nil, Constant},
    panic,
};

pub fn get(args: &[Constant]) -> InterpretResult<Constant> {
    let key = match &args[0] {
        Constant::Sym(key) => key,
        other => panic!("get[1] expected a string, but found {}", other)?,
    };

    match &args[1] {
        Constant::Table(xs) => Ok(xs.get(key).unwrap_or_else(nil)),
        other => panic!("get() expected a map, found {}", other),
    }
}

pub fn insert(args: &[Constant]) -> InterpretResult<Constant> {
    let key = match &args[0] {
        Constant::Sym(s) => *s,
        other => return panic!("insert()[1] expected a symbol, found {}", other),
    };
    let value = args[1].clone();

    match &args[2] {
        Constant::Table(ts) => Ok(Constant::Table(GcRef::new(ts.insert(key, value)))),
        other => panic!("insert()[0] expected a table, found {}", other),
    }
}
