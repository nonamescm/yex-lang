use crate::{
    error::InterpretResult,
    literal::{nil, TryGet},
    Value, VirtualMachine,
};

use super::Tuple;

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let tup: Tuple = args[1].get()?;
    let idx: usize = args[0].get()?;

    Ok(tup.0.get(idx).cloned().unwrap_or_else(nil))
}

pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(vec![].into())
}
