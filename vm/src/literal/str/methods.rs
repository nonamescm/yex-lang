use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{nil, TryGet},
    raise, Value, VirtualMachine, List,
};

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[0].get()?;
    let index: f64 = args[1].get()?;

    if index < 0.0 || index.fract() != 0.0 {
        raise!(
            "get[1] expected a valid positive integer, but found {}",
            index
        )?;
    }

    let char = string
        .chars()
        .nth(index as usize)
        .map(|c| c.to_string().into())
        .unwrap_or_else(nil);
    Ok(char)
}

pub fn split(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[0].get()?;
    let separator: String = args[1].get()?;

    let list: List = string.split(&separator).map(|str| str.to_owned().into()).collect();

    Ok(list.into())
}

pub fn init(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(String::from(""))))
}
