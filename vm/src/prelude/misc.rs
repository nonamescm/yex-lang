use crate::err_tuple;
use crate::error::InterpretError;
use crate::panic;
use crate::{
    gc::GcRef,
    list,
    literal::{err, nil, ok, Constant},
};
use std::process::exit;

pub fn yex_panic(args: &[Constant]) -> Constant {
    use Constant::*;
    let msg = match &args[0] {
        Str(msg) => msg.get(),
        other => err_tuple!("panic() expected str, found {}", other),
    };

    let a: Result<(), InterpretError> = panic!("{}", msg);
    if let Err(e) = a {
        println!("{}", e.to_string());
        exit(1);
    }
    nil()
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
