use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{instance::Instance, TryGet},
    EnvTable, Value, VirtualMachine, YexModule,
};

pub fn create(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let arg: GcRef<Instance> = args[0].get()?;
    let arg: String = arg.get_field("path").get()?;

    File::create(arg).map(|_| Value::Nil).map_err(|e| e.into())
}

pub fn read(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let arg: GcRef<Instance> = args[0].get()?;
    let arg: String = arg.get_field("path").get()?;

    let file = File::open(arg);

    file.and_then(|mut file| {
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(buf.into())
    })
    .map_err(Into::into)
}

pub fn append(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let arg: GcRef<Instance> = args[0].get()?;
    let arg: String = arg.get_field("path").get()?;

    let file = OpenOptions::new().write(true).append(true).open(arg);

    let content: String = args[1].get()?;

    file.and_then(|mut file| file.write(content.as_bytes()))
        .map(|_| Value::Nil)
        .map_err(Into::into)
}

pub fn write(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let arg: GcRef<Instance> = args[0].get()?;
    let arg: String = arg.get_field("path").get()?;

    let file = OpenOptions::new().truncate(true).write(true).open(arg);

    let arg: String = args[1].get()?;
    let content = arg.as_bytes();

    file.and_then(|mut file| file.write(content))
        .map(|_| Value::Nil)
        .map_err(Into::into)
}

pub fn delete(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let arg: GcRef<Instance> = args[0].get()?;
    let arg: String = arg.get_field("path").get()?;

    std::fs::remove_file(arg)
        .map(|_| Value::Nil)
        .map_err(|e| e.into())
}

pub fn new(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let path: String = args[0].get()?;

    let mut envtable = EnvTable::new();
    envtable.insert("path".into(), path.into());

    Ok(Instance::new(GcRef::new(YexModule::file()), envtable).into())
}
