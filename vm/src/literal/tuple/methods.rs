use std::fmt::Write;

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

pub fn show(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: Tuple = args[0].get()?;

    let mut s = String::from('(');

    for x in xs.0.iter() {
        write!(s, "{}, ", super::super::show(vm, vec![x.clone()])?).unwrap();
    }

    if xs.0.len() > 0 {
        s.pop();
        s.pop();
    }

    s.push(')');

    Ok(s.into())
}

pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(vec![].into())
}
