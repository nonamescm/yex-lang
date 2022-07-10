use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{nil, TryGet},
    raise, List, Value, VirtualMachine,
};

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[1].get()?;
    let index: usize = args[0].get()?;

    let char = string
        .chars()
        .nth(index)
        .map_or_else(nil, |c| c.to_string().into());
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

pub fn chars(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let str: String = args[0].get()?;
    let iter = str.chars().map(|c| c.to_string().into());

    Ok(iter.rev().collect::<List>().into())
}

pub fn ord(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let str: String = args[0].get()?;

    if str.len() != 1 {
        raise!(ValueError, "Expected a character for 'ord'")?;
    }

    Ok(Value::Num(str.as_bytes()[0].into()))
}

pub fn chr(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let char_code: usize = args[0].get()?;

    let code = char_code.try_into().ok().and_then(char::from_u32);
    let code = match code {
        Some(ch) => ch.to_string(),
        None => raise!(ValueError, "Can't convert the number to a char")?,
    };

    Ok(code.into())
}
pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(String::from(""))))
}
