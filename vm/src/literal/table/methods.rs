use crate::{
    error::InterpretResult, gc::GcRef, literal::TryGet, Symbol, Value, VirtualMachine, YexModule,
};

use super::YexStruct;

pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Struct(YexStruct::new(
        GcRef::new(YexModule::struct_()),
    )))
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
