use std::fmt::Write;

use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{tuple::Tuple, TryGet},
    Symbol, Value, VirtualMachine, YexModule,
};

use super::YexStruct;

pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Struct(YexStruct::new(GcRef::new(
        YexModule::struct_(),
    ))))
}

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let table: YexStruct = args[1].get()?;
    let key: Symbol = args[0].get()?;
    Ok(table.get(key))
}

pub fn insert(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let table: YexStruct = args[2].get()?;
    let key: Symbol = args[0].get()?;
    let value = args[1].clone();
    Ok(Value::Struct(table.insert(key, value)))
}

pub fn show(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let table: YexStruct = args[0].get()?;

    let mut str = String::from("%Struct{");

    for entry in table.items.iter() {
        let entry: Tuple = entry.get()?;
        write!(
            str,
            "{}: {}, ",
            TryGet::<Symbol>::get(&entry.0[0])?.as_str(),
            super::super::show(vm, vec![entry.0[1].clone()])?
        )
        .ok();
    }
    str.pop();
    str.pop();

    str.push('}');

    Ok(str.into())
}
