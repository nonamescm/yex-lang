use crate::{
    err_tuple,
    gc::GcRef,
    list::List,
    literal::{nil, Constant},
    VirtualMachine,
};

pub fn rev(args: &[Constant]) -> Constant {
    let xs = match &args[0] {
        Constant::List(xs) => xs,
        other => err_tuple!("rev[0] expected a list, but found `{}`", other),
    };
    Constant::List(GcRef::new(xs.rev()))
}

pub fn map(vm: &mut VirtualMachine, args: &[Constant]) -> Constant {
    let fun = args[0].clone();
    let xs = match &args[1] {
        Constant::List(xs) => xs,
        other => err_tuple!("map[1] expected a list, but found `{}`", other),
    };

    let xs = xs
        .iter()
        .map(|it| {
            vm.push(it);
            vm.push(fun.clone());
            if let Err(e) = vm.call(1) {
                err_tuple!("{}", e)
            }
            vm.pop()
        })
        .collect::<List>();

    Constant::List(GcRef::new(xs.rev()))
}

pub fn fold(vm: &mut VirtualMachine, args: &[Constant]) -> Constant {
    let mut acc = args[0].clone();
    let fun = args[1].clone();
    let xs = match &args[2] {
        Constant::List(xs) => xs,
        other => err_tuple!("fold[2] expected a list, but found `{}`", other),
    };

    for it in xs.iter() {
        vm.push(acc);
        vm.push(it);
        vm.push(fun.clone());
        if let Err(e) = vm.call(2) {
            err_tuple!("{}", e)
        }
        acc = vm.pop();
    }

    acc
}

pub fn head(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::List(xs) => match xs.head() {
            Some(x) => x,
            None => nil(),
        },
        other => err_tuple!("head() expected a list, found {}", other),
    }
}

pub fn tail(args: &[Constant]) -> Constant {
    match &args[0] {
        Constant::List(xs) => Constant::List(GcRef::new(xs.tail())),
        other => err_tuple!("tail() expected a list, found {}", other),
    }
}

pub fn insert(args: &[Constant]) -> Constant {
    let key = match &args[1] {
        Constant::Sym(s) => *s,
        other => err_tuple!("insert()[1] expected a symbol, found {}", other),
    };
    let value = args[2].clone();

    match &args[0] {
        Constant::Table(ts) => {
            let mut ts = (*ts).clone();
            ts.insert(key, value);
            Constant::Table(ts)
        }
        other => err_tuple!("insert()[0] expected a table, found {}", other),
    }
}
