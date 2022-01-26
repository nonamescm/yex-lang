use crate::{
    error::InterpretResult,
    gc::GcRef,
    list::List,
    literal::{nil, Constant},
    panic, VirtualMachine,
};

pub fn rev(args: &[Constant]) -> InterpretResult<Constant> {
    let xs = match &args[0] {
        Constant::List(xs) => xs,
        other => return panic!("rev[0] expected a list, but found `{}`", other),
    };
    Ok(Constant::List(GcRef::new(xs.rev())))
}

pub fn map(vm: &mut VirtualMachine, args: &[Constant]) -> InterpretResult<Constant> {
    let fun = &args[0];
    let xs = match &args[1] {
        Constant::List(xs) => xs,
        other => return panic!("map[1] expected a list, but found `{}`", other),
    };

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

    Ok(Constant::List(GcRef::new(xs.rev())))
}

pub fn fold(vm: &mut VirtualMachine, args: &[Constant]) -> InterpretResult<Constant> {
    let mut acc = args[0].clone();
    let fun = args[1].clone();
    let xs = match &args[2] {
        Constant::List(xs) => xs,
        other => panic!("fold[2] expected a list, but found `{}`", other)?,
    };

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

pub fn filter(vm: &mut VirtualMachine, args: &[Constant]) -> InterpretResult<Constant> {
    let fun = &args[0];
    let xs = match &args[1] {
        Constant::List(xs) => xs,
        other => panic!("filter[1] expected a list, but found `{}`", other)?,
    };

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

    Ok(Constant::List(GcRef::new(ys.rev())))
}

pub fn head(args: &[Constant]) -> InterpretResult<Constant> {
    match &args[0] {
        Constant::List(xs) => Ok(match xs.head() {
            Some(x) => x,
            None => nil(),
        }),
        other => panic!("head() expected a list, found {}", other),
    }
}

pub fn tail(args: &[Constant]) -> InterpretResult<Constant> {
    match &args[0] {
        Constant::List(xs) => Ok(Constant::List(GcRef::new(xs.tail()))),
        other => panic!("tail() expected a list, found {}", other),
    }
}

pub fn insert(args: &[Constant]) -> InterpretResult<Constant> {
    let key = match &args[0] {
        Constant::Sym(s) => *s,
        other => return panic!("insert()[1] expected a symbol, found {}", other),
    };
    let value = args[1].clone();

    match &args[2] {
        Constant::Table(ts) => Ok(Constant::Table(GcRef::new(ts.insert(key, value)))),
        other => panic!("insert()[0] expected a table, found {}", other),
    }
}
