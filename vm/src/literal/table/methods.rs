use crate::{error::InterpretResult, literal::TryGet, Symbol, Value, VirtualMachine};

use super::Table;

pub fn init(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Table(Table::new()))
}
pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let table: Table = args[0].get()?;
    let key: Symbol = args[1].get()?;
    Ok(table.get(key).unwrap_or(Value::Nil))
}
pub fn insert(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let table: Table = args[0].get()?;
    let key: Symbol = args[1].get()?;
    let value = args[2].clone();
    Ok(Value::Table(table.insert(key, value)))
}
