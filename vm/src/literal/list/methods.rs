use crate::{
    error::InterpretResult,
    literal::{nil, TryGet, Value},
    VirtualMachine,
};

use super::List;

pub fn rev(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[0].get()?;
    Ok(Value::List(xs.rev()))
}

pub fn map(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };
    let xs: List = args[1].get()?;
    let fun = &args[0];

    let xs: InterpretResult<List> = xs
        .iter()
        .map(|it| {
            vm.push(it);
            vm.push(fun.clone());
            vm.call(1)?;
            Ok(vm.pop())
        })
        .try_fold(List::new(), |xs, x| match x {
            Ok(x) => Ok(xs.prepend(x)),
            Err(e) => Err(e),
        });

    Ok(xs?.rev().into())
}

pub fn fold(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs: List = args[2].get()?;
    let mut acc = args[0].clone();
    let fun = args[1].clone();

    for it in xs.iter() {
        vm.push(acc);
        vm.push(it);
        vm.push(fun.clone());

        vm.call(2)?;

        acc = vm.pop();
    }

    Ok(acc)
}

pub fn filter(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs: List = args[1].get()?;
    let fun = &args[0];

    let mut ys = List::new();

    for x in xs.iter() {
        vm.push(x.clone());
        vm.push(fun.clone());

        vm.call(1)?;

        let res = vm.pop();
        if res.to_bool() {
            ys = ys.prepend(x);
        }
    }

    Ok(ys.rev().into())
}

pub fn head(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[0].get()?;
    Ok(xs.head().unwrap_or(Value::Nil))
}

pub fn tail(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    args[0].get().map(|xs: List| xs.tail().into())
}

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[1].get()?;
    let n: usize = args[0].get()?;

    Ok(xs.index(n))
}

pub fn drop(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[1].get()?;
    let n: usize = args[0].get()?;

    Ok(xs.drop(n).into())
}

pub fn join(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[1].get()?;
    let sep: String = args[0].get()?;

    Ok(xs.join(&sep).into())
}

pub fn find(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs: List = args[1].get()?;

    let fun = &args[0];

    for x in xs.iter() {
        vm.push(x.clone());
        vm.push(fun.clone());

        vm.call(1)?;

        if vm.pop().to_bool() {
            return Ok(x);
        }
    }

    Ok(nil())
}

pub fn len(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[1].get()?;

    Ok((xs.len() as f64).into())
}

pub fn new(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(List::new().into())
}
