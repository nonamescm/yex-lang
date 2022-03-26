use crate::{
    error::InterpretResult,
    list::List,
    literal::{nil, Value},
    panic, VirtualMachine,
};

pub fn rev(args: &[Value]) -> InterpretResult<Value> {
    let xs = match &args[0] {
        Value::List(xs) => xs,
        other => return panic!("rev[0] expected a list, but found `{}`", other),
    };
    Ok(Value::List(xs.rev()))
}

pub fn map(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };
    let xs = match &args[0] {
        Value::List(xs) => xs,
        other => return panic!("map[1] expected a list, but found `{}`", other),
    };
    let fun = &args[1];

    let xs = xs
        .iter()
        .map(|it| {
            vm.push(it);
            vm.push(fun.clone());
            if let Err(e) = vm.call(1) {
                panic!("{}", e)?
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
        other => panic!("fold[2] expected a list, but found `{}`", other)?,
    };
    let mut acc = args[1].clone();
    let fun = args[2].clone();

    for it in xs.iter() {
        vm.push(acc);
        vm.push(it);
        vm.push(fun.clone());
        if let Err(e) = vm.call(2) {
            return panic!("{}", e);
        }
        acc = vm.pop();
    }

    Ok(acc)
}

pub fn filter(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let xs = match &args[0] {
        Value::List(xs) => xs,
        other => panic!("filter[1] expected a list, but found `{}`", other)?,
    };
    let fun = &args[1];

    let mut ys = List::new();

    for x in xs.iter() {
        vm.push(x.clone());
        vm.push(fun.clone());

        if let Err(e) = vm.call(1) {
            return panic!("{}", e);
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
        other => panic!("head() expected a list, found {}", other),
    }
}

pub fn tail(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    match &args[0] {
        Value::List(xs) => Ok(Value::List(xs.tail())),
        other => panic!("tail() expected a list, found {}", other),
    }
}

pub fn nth(args: &[Value]) -> InterpretResult<Value> {
    let n = match &args[0] {
        Value::Num(n) if n.fract() == 0.0 && *n >= 0.0 => *n as usize,
        other => panic!(
            "nth[1] expected a valid positive integer, but found {}",
            other
        )?,
    };

    match &args[1] {
        Value::List(xs) => Ok(xs.index(n)),
        other => panic!("nth() expected a list, found {}", other),
    }
}
