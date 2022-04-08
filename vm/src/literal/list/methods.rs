use crate::{
    error::InterpretResult,
    literal::{nil, TryGet, Value},
    raise, VirtualMachine,
};

use super::List;

pub fn rev(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[0].get()?;
    Ok(Value::List(xs.rev()))
}

pub fn map(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };
    let xs: List = args[0].get()?;
    let fun = &args[1];

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

    let xs: List = args[0].get()?;
    let mut acc = args[1].clone();
    let fun = args[2].clone();

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

    let xs: List = args[0].get()?;
    let fun = &args[1];

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
    let xs: List = args[0].get()?;
    let n: f64 = args[1].get()?;

    if n.fract() != 0.0 || n < 0.0 {
        raise!(ValueError)?;
    }

    let n = n as usize;

    Ok(xs.index(n))
}

pub fn drop(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[0].get()?;
    let n: f64 = args[1].get()?;

    if n.fract() != 0.0 || n < 0.0 {
        raise!(ValueError)?;
    }

    let n = n as usize;

    Ok(xs.drop(n).into())
}

pub fn join(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs: List = args[0].get()?;
    let sep: String = args[1].get()?;

    Ok(xs.join(&sep).into())
}

pub fn find(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs: List = args[0].get()?;

    let fun = &args[1];

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

pub fn init(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(List::new().into())
}
