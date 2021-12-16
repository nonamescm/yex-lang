
use crate::err_tuple;
use crate::literal::FFINoArgFunction;
use crate::literal::FFIFunction;
use crate::literal::nil;
use crate::Constant;
use crate::GcRef;
use crate::VirtualMachine;
use crate::ok_tuple;
use dlopen::raw::Library;

pub fn dlopen(vm: &mut VirtualMachine, args: &[Constant]) -> Constant {
    use Constant::*;

    let libname = match &args[0] {
        Str(libname) => libname.get(),
        other => err_tuple!("dlopen()[0] expected str, found {}", other),
    };

    let fn_name = match &args[1] {
        Str(fn_name) => fn_name.get(),
        other => err_tuple!("dlopen()[1] expected str, found {}", other),
    };
    let number_of_args = match &args[2] {
        Num(number_of_args) => number_of_args,
        other => err_tuple!("dlopen()[2] expected int, found {}", other),
    };
    let libname = libname.to_string();
    let lib = match vm.dlopen_libs.get(&libname) {
        Some(lib) => lib,
        None => match Library::open(libname.clone()) {
            Ok(val) => {
                vm.dlopen_libs.insert(libname.clone(), GcRef::new(val));
                vm.dlopen_libs.get(&libname).unwrap()
            }
            Err(err) => err_tuple!("{}", err),
        },
    };
    
    unsafe {
        if *number_of_args == 0.0 {
            let func = match lib.symbol::<FFINoArgFunction>(fn_name) {
                Ok(func) => func,
                Err(err) => err_tuple!("{}", err),
            };
            
            let call = Constant::Fun(GcRef::new(crate::literal::Fun {
                arity: *number_of_args as i64 as usize,
                args: vec![ExternalFunctionNoArg(func)],
                body: GcRef::new(crate::Either::Right(|_, args| {
                    match &args[0] {
                        ExternalFunctionNoArg(f) => f(),
                        
                        _ => unreachable!(),
                    }
                    nil()
                })),
            }));
            return ok_tuple!(call);
        } else {
            let _func = match lib.symbol::<FFIFunction>(fn_name) {
                Ok(func) => func,
                Err(err) => err_tuple!("{}", err),
            };
            todo!()
        };
    }
}
