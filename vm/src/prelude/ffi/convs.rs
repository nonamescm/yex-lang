use libc::c_void;

use crate::{Constant, gc::GcRef, err_tuple, literal::nil};
use std::{mem, ffi::{CString, CStr}};

pub fn c_ptr_to_cont(ptr: *mut c_void, fun_ty: &str) -> Constant {
    match fun_ty {
        "num" => {
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
        ty => err_tuple!("unknown C Type: {}", ty),
    }
}
pub unsafe fn to_c_ptr(cont: &Constant) -> Result<*mut u8, String> {
    use Constant::*;
    match cont {
        Num(num) => Ok(mem::transmute(*num)),
        Str(s) => {
            let mut str = s.to_string();
            if str.ends_with("\0") {
                str.pop();
            }
            str.shrink_to_fit();
            assert_eq!(str.len(), str.capacity());
            let strlen = str.len();
            let c_string = CString::new(str).unwrap();
            let ptr = libc::malloc(strlen);
            libc::strcpy(ptr as *mut i8, c_string.as_ptr());
            mem::forget(ptr);
            assert!(!ptr.is_null());
            Ok(ptr as *mut u8)
        }
        other => Err(format!("FFI FAIL: {} can't tranformed into a C ptr", other))
    }
}
