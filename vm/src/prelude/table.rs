use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{nil, Value},
    panic,
};

pub fn get(args: &[Value]) -> InterpretResult<Value> {
    let key = match &args[0] {
        Value::Sym(key) => key,
        other => panic!("get[1] expected a string, but found {}", other)?,
    };

    match &args[1] {
        Value::Table(xs) => Ok(xs.get(key).unwrap_or_else(nil)),
        other => panic!("get() expected a map, found {}", other),
    }
}

pub fn insert(args: &[Value]) -> InterpretResult<Value> {
    let key = match &args[0] {
        Value::Sym(s) => *s,
        other => return panic!("insert()[1] expected a symbol, found {}", other),
    };
    let value = args[1].clone();

    match &args[2] {
        Value::Table(ts) => Ok(Value::Table(GcRef::new(ts.insert(key, value)))),
        other => panic!("insert()[0] expected a table, found {}", other),
    }
}
