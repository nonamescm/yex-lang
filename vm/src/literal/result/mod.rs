use crate::{error::InterpretResult, gc::GcRef, Symbol, Tuple, Value, VirtualMachine, YexModule};

#[must_use]
pub fn ok(args: Vec<Value>) -> Value {
    let this: GcRef<YexModule> = GcRef::new(YexModule::default());
    let tup = Tuple(GcRef::new(args.into_boxed_slice()));
    Value::Tagged(this, Symbol::from("Result.ok"), tup)
}

#[must_use]
pub fn fail(args: Vec<Value>) -> Value {
    let this: GcRef<YexModule> = GcRef::new(YexModule::default());
    let tup = Tuple(GcRef::new(args.into_boxed_slice()));
    Value::Tagged(this, Symbol::from("Result.fail"), tup)
}

pub fn vm_ok(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    Ok(ok(args))
}

pub fn vm_fail(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    Ok(fail(args))
}
