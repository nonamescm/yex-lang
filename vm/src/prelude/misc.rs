use crate::err_tuple;
use crate::literal::nil;
use crate::panic;
use crate::{
    gc::GcRef,
    list,
    VirtualMachine,
    literal::{err, ok, Constant},
};
use std::process::exit;

pub fn yex_panic(args: &[Constant]) -> Constant {
    use Constant::*;
    let msg = match &args[0] {
        Str(msg) => msg.get(),
        other => err_tuple!("panic() expected str, found {}", other),
    };

    let err: Result<(), _> =  panic!("{}", msg);
    eprintln!("{}", err.unwrap_err());
    exit(1);
}

/*
 * returns an error array
*/
pub fn yex_error(args: &[Constant]) -> Constant {
    let mut list = list::List::new();
    list = list.prepend(args[0].clone());
    list = list.prepend(err());
    Constant::List(GcRef::new(list))
}
/*
 * returns an Ok array
*/
pub fn yex_ok(args: &[Constant]) -> Constant {
    let mut list = list::List::new();
    list = list.prepend(args[0].clone());
    list = list.prepend(ok());
    Constant::List(GcRef::new(list))
}
pub fn set(vm: &mut VirtualMachine, args: &[Constant]) -> Constant {
    let key = match &args[0] {
        Constant::Sym(s) => *s,
        other => err_tuple!("set()[0] expected a symbol, found {}", other),
    };
    let val = args[1].clone();
    vm.variables.remove(&key);
    vm.variables.insert(key, val);
    nil()
}