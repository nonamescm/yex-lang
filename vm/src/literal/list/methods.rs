use crate::{
    error::InterpretResult,
    literal::{nil, Value},
    raise, VirtualMachine,
};

use super::List;

pub fn rev(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let xs = match &args[0] {
        Value::List(xs) => xs,
        _ => unreachable!(),
    };
    Ok(Value::List(xs.rev()))
}

pub fn map(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };
    let xs = match &args[0] {
        Value::List(xs) => xs,
        _ => unreachable!(),
    };
    let fun = &args[1];

    let xs = xs
        .iter()
        .map(|it| {
            vm.push(it);
            vm.push(fun.clone());
            if let Err(e) = vm.call(1) {
                raise!("{}", e)?
            }
            Ok(vm.pop())
        })
        .try_fold(List::new(), |xs, x| match x {
            Ok(x) => Ok(xs.prepend(x)),
            Err(e) => Err(e),
        })?;

    Ok(Value::List(xs.rev()))
}

pub fn fold(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs = match &args[0] {
        Value::List(xs) => xs,
        other => raise!("fold[2] expected a list, but found `{}`", other)?,
    };
    let mut acc = args[1].clone();
    let fun = args[2].clone();

    for it in xs.iter() {
        vm.push(acc);
        vm.push(it);
        vm.push(fun.clone());
        if let Err(e) = vm.call(2) {
            return raise!("{}", e);
        }
        acc = vm.pop();
    }

    Ok(acc)
}

pub fn filter(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs = match &args[0] {
        Value::List(xs) => xs,
        _ => unreachable!(),
    };
    let fun = &args[1];

    let mut ys = List::new();

    for x in xs.iter() {
        vm.push(x.clone());
        vm.push(fun.clone());

        if let Err(e) = vm.call(1) {
            return raise!("{}", e);
        }

        let res = vm.pop();
        if res.to_bool() {
            ys = ys.prepend(x);
        }
    }

    Ok(Value::List(ys.rev()))
}

pub fn head(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    match &args[0] {
        Value::List(xs) => Ok(match xs.head() {
            Some(x) => x,
            None => nil(),
        }),
        _ => unreachable!(),
    }
}

pub fn tail(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    match &args[0] {
        Value::List(xs) => Ok(Value::List(xs.tail())),
        _ => unreachable!(),
    }
}

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let n = match &args[1] {
        Value::Num(n) if n.fract() == 0.0 && *n >= 0.0 => *n as usize,
        other => raise!(
            "nth[1] expected a valid positive integer, but found {}",
            other
        )?,
    };

    match &args[0] {
        Value::List(xs) => Ok(xs.index(n)),
        _ => unreachable!(),
    }
}

pub fn drop(_:  *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let n = match &args[1] {
        Value::Num(n) if n.fract() == 0.0 && *n >= 0.0 => *n as usize,
        other => raise!(
            "drop[1] expected a valid positive integer, but found {}",
            other
        )?,
    };

    match &args[0] {
        Value::List(xs) => Ok(xs.drop(n).into()),
        _ => unreachable!(),
    }
}

pub fn init(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::List(List::new()))
}
