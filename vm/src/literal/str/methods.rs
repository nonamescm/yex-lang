use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{nil, TryGet},
    List, Value, VirtualMachine,
};

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[1].get()?;
    let index: usize = args[0].get()?;

    let char = string
        .chars()
        .nth(index)
        .map(|c| c.to_string().into())
        .unwrap_or_else(nil);
    Ok(char)
}

pub fn split(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[1].get()?;
    let separator: String = args[0].get()?;

    let list: List = string
        .split(&separator)
        .map(|str| str.to_owned().into())
        .collect();

    Ok(list.rev().into())
}

pub fn len(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let str: String = args[0].get()?;

    Ok((str.len() as f64).into())
}

pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(String::from(""))))
}
