mod convs;
use crate::{
    literal::nil, literal::FFIFunction, literal::FFINoArgFunction, panic, stackvec, Constant,
    GcRef, InterpretResult, VirtualMachine,
};
use dlopen::raw::Library;

pub fn dlclose(vm: &mut VirtualMachine, args: &[Constant]) -> InterpretResult<Constant> {
    use Constant::Str;
    vm.dlopen_libs.remove(match &args[0] {
        Str(libname) => &**libname,
        other => return panic!("dlclose()[0] expected str, found {}", other),
    });
    Ok(nil())
}

pub fn dlopen(vm: &mut VirtualMachine, args: &[Constant]) -> InterpretResult<Constant> {
    use Constant::*;

    let libname = match &args[0] {
        Str(libname) => &*libname,
        other => return panic!("dlopen()[0] expected str, found {}", other),
    };

    let fn_name = match &args[1] {
        Str(fn_name) => &*fn_name,
        other => return panic!("dlopen()[1] expected str, found {}", other),
    };
    let number_of_args = match &args[2] {
        Num(number_of_args) => number_of_args,
        other => return panic!("dlopen()[2] expected int, found {}", other),
    };
    let typeof_fun = match &args[3] {
        Sym(ty) => ty.to_str().to_string(),
        other => return panic!("dlopen()[2] expected sym, found {}", other),
    };

    let libname = libname.to_string();
    let lib = match vm.dlopen_libs.get(&libname) {
        Some(lib) => lib,
        None => match Library::open(libname.clone()) {
            Ok(val) => {
                vm.dlopen_libs.insert(libname.clone(), GcRef::new(val));
                vm.dlopen_libs.get(&libname).unwrap()
            }
            Err(err) => return panic!("{}", err),
        },
    };

    unsafe {
        if *number_of_args == 0.0 {
            let func = match lib.symbol::<FFINoArgFunction>(fn_name) {
                Ok(func) => func,
                Err(err) => return panic!("{}", err),
            };

            let call = Constant::Fun(GcRef::new(crate::literal::Fun {
                arity: *number_of_args as i64 as usize,
                args: stackvec![ExternalFunctionNoArg(func), Str(GcRef::new(typeof_fun))],
                body: GcRef::new(crate::Either::Right(|_, mut args| {
                    args.reverse();
                    match &args[0] {
                        ExternalFunctionNoArg(f) => {
                            let r = f();

                            convs::c_ptr_to_cont(
                                r,
                                match &args[1] {
                                    Str(s) => s.as_str(),
                                    _ => unreachable!(),
                                },
                            )
                        }
                        _ => unreachable!(),
                    }
                })),
            }));
            Ok(call)
        } else {
            let func = match lib.symbol::<FFIFunction>(fn_name) {
                Ok(func) => func,
                Err(err) => return panic!("{}", err),
            };
            let call = Constant::Fun(GcRef::new(crate::literal::Fun {
                arity: *number_of_args as i64 as usize,
                args: stackvec![Str(GcRef::new(typeof_fun)), ExternalFunction(func)],
                body: GcRef::new(crate::Either::Right(|_, args| match &args[0] {
                    ExternalFunction(fn_ptr) => {
                        let typeof_fun = match &args[1] {
                            Constant::Str(s) => s,
                            _ => unreachable!(),
                        };
                        let mut c_args = vec![];
                        let yex_args = &args[2..];
                        for arg in yex_args {
                            c_args.push(match convs::to_c_ptr(arg) {
                                Ok(a) => a,
                                Err(err) => return panic!("{}", err),
                            });
                        }
                        c_args.shrink_to_fit();
                        let mut c_args = c_args
                            .into_iter()
                            .map(|s| Box::into_raw(Box::new(s)))
                            .collect::<Vec<_>>();

                        c_args.shrink_to_fit();
                        let ptr = c_args.as_mut_ptr();
                        let r = fn_ptr(c_args.len(), ptr as *mut u8);
                        convs::c_ptr_to_cont(r, typeof_fun)
                    }
                    _ => unreachable!(),
                })),
            }));
            Ok(call)
        }
    }
}
