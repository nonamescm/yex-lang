use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{result, TryGet},
    Value, VirtualMachine,
};

use super::Ffi;

pub fn open(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let path: String = args[0].get()?;
    let res = unsafe { Ffi::open(path) };
    //TODO: Create a error type for this
    match res.map_err(|err| result::fail(vec![Value::Str(GcRef::new(err.to_string()))])) {
        Ok(f) => Ok(Value::FFI(f)),
        Err(e) => Ok(e),
    }
}

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let mut this: Ffi = args[1].get()?;
    let identifier: String = args[0].get()?;
    Ok(this.get(identifier).unwrap_or(Value::Nil))
}
