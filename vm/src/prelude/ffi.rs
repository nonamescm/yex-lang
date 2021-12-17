use crate::err_tuple;
use crate::literal::nil;
use crate::literal::FFIFunction;
use crate::literal::FFINoArgFunction;
use crate::ok_tuple;
use crate::Constant;
use crate::GcRef;
use crate::VirtualMachine;
use dlopen::raw::Library;

use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;

fn _conv(ptr: *mut c_void, fun_ty: &str) -> Constant {
    match fun_ty {
        "int" => {
            //let int: *mut f32 = ptr.cast();

            if ptr.is_null() {
                return nil();
            };
            let f: *mut f64 = ptr.cast();

            unsafe {
                let float_value = f.read();

                return Constant::Num(float_value);
            }
        }
        "str" => unsafe {
            let c_str = CStr::from_ptr(ptr as *const i8);
            match c_str.to_str() {
                Ok(s) => Constant::Str(GcRef::new(s.to_string())),
                Err(err) => err_tuple!("{}", err),
            }
        },
        "void" => nil(),
        ty => err_tuple!("unknown C_Type: {}", ty),
    }
}
fn _conv_to_c(cont: &Constant) -> Result<*mut c_void, String> {
    match cont {
        Constant::Str(s) => {
            // let str = CString::new(s.clone().to_string()).unwrap();
            // let ptr = str.into_raw();
            // std::mem::forget(ptr);
            let str = s.clone().to_string();
            let ptr = Box::into_raw(Box::new(str));
            std::mem::forget(ptr);            
            Ok(ptr as *mut c_void)
        }
        Constant::Num(num) => {
            let c_float = *num;
            unsafe { Ok(std::mem::transmute(c_float)) }
        }
        inco => Err(format!("{} ins't not supported yet.", inco)),
    }
}
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
    let typeof_fun = match &args[3] {
        Sym(ty) => ty.to_str().to_string(),
        other => err_tuple!("dlopen()[2] expected sym, found {}", other),
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
                args: vec![ExternalFunctionNoArg(func), Str(GcRef::new(typeof_fun))],
                body: GcRef::new(crate::Either::Right(|_, mut args| {
                    args.reverse();
                    match &args[0] {
                        ExternalFunctionNoArg(f) => {
                            let r = f();

                            _conv(
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
            return ok_tuple!(call);
        } else {
            let func = match lib.symbol::<FFIFunction>(fn_name) {
                Ok(func) => func,
                Err(err) => err_tuple!("{}", err),
            };
            let call = Constant::Fun(GcRef::new(crate::literal::Fun {
                arity: *number_of_args as i64 as usize,
                args: vec![Str(GcRef::new(typeof_fun)), ExternalFunction(func)],
                body: GcRef::new(crate::Either::Right(|_, args| {
                    println!("-> {:?}", args);
                    match &args[0] {
                        ExternalFunction(fn_ptr) => {
                            let typeof_fun = match &args[1] {
                                Constant::Str(s) => s,
                                _ => unreachable!(),
                            };
                            let mut c_args = vec![];
                            let yex_args = &args[2..];
                            for arg in yex_args {
                                c_args.push(match _conv_to_c(&arg) {
                                    Ok(a) => a,
                                    Err(err) => err_tuple!("{}", err),
                                });
                            }
                            c_args.shrink_to_fit();
                            let mut c_args = c_args
                                .into_iter()
                                .map(|s| Box::into_raw(Box::new(s)))
                                .collect::<Vec<_>>();
                            let ptr = c_args.as_mut_ptr();
                            std::mem::forget(ptr);

                            let r = fn_ptr(c_args.len(), ptr as *mut c_void);
                            return _conv(r, &typeof_fun);
                        }
                        _ => unreachable!(),
                    }
                })),
            }));
            return ok_tuple!(call);
        };
    }
}
