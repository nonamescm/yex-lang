use crate::{
    err_tuple,
    gc::GcRef,
    list::List,
    literal::{nil, ConstantRef},
    Constant, VirtualMachine,
};

pub fn rev(args: &[ConstantRef]) -> ConstantRef {
    let xs = match args[2].get() {
        Constant::List(xs) => xs,
        other => err_tuple!("rev[0] expected a list, but found `{}`", other),
    };
    GcRef::new(Constant::List(xs.rev()))
}

pub fn map(vm: &mut VirtualMachine, args: &[ConstantRef]) -> ConstantRef {
    let fun = GcRef::clone(&args[0]);
    let xs = match args[1].get() {
        Constant::List(xs) => xs,
        other => err_tuple!("map[1] expected a list, but found `{}`", other),
    };

    let xs = xs
        .iter()
        .map(|it| {
            vm.push_gc_ref(it);
            vm.push_gc_ref(GcRef::clone(&fun));
            if let Err(e) = vm.call(1) {
                err_tuple!("{}", e)
            }
            vm.pop()
        })
        .collect::<List>();

    GcRef::new(Constant::List(xs.rev()))
}

pub fn fold(vm: &mut VirtualMachine, args: &[ConstantRef]) -> ConstantRef {
    let mut acc = GcRef::clone(&args[0]);
    let fun = GcRef::clone(&args[1]);
    let xs = match args[2].get() {
        Constant::List(xs) => xs,
        other => err_tuple!("fold[2] expected a list, but found `{}`", other),
    };

    for it in xs.iter() {
        vm.push_gc_ref(acc);
        vm.push_gc_ref(it);
        vm.push_gc_ref(GcRef::clone(&fun));
        if let Err(e) = vm.call(1) {
            err_tuple!("{}", e)
        }
        acc = vm.pop()
    }

    acc
}

pub fn head(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::List(xs) => match xs.head() {
            Some(x) => x,
            None => nil(),
        },
        other => err_tuple!("head() expected a list, found {}", other),
    }
}

pub fn tail(args: &[ConstantRef]) -> ConstantRef {
    match args[0].get() {
        Constant::List(xs) => GcRef::new(Constant::List(xs.tail())),
        other => err_tuple!("tail() expected a list, found {}", other),
    }
}
