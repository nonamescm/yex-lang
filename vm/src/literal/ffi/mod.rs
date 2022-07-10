pub mod methods;
pub mod userdata;

use std::fmt::{Debug, Display};

use crate::{gc::GcRef, EnvTable, Symbol, Value};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
#[derive(WrapperApi)]
pub struct Api {
    init: fn() -> EnvTable,
}
#[derive(Clone)]
pub struct Ffi {
    module: GcRef<Container<Api>>,
    table: Option<EnvTable>,
    path: String,
}
impl Ffi {
    /// Open a external library
    /// # SAFETY:
    ///   This depends on safety of `dlopen`.
    pub(crate) unsafe fn open(path: String) -> Result<Self, dlopen::Error> {
        let module: Container<Api> = Container::load(&path)?;
        Ok(Self {
            module: GcRef::new(module),
            table: None,
            path,
        })
    }

    pub fn get(&mut self, val: impl Into<Symbol>) -> Option<Value> {
        let table = self.table.get_or_insert(self.module.init());
        table.get(&val.into())
    }
}
impl Debug for Ffi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FFI {{ path: {}, module: ?? }}", self.path)
    }
}
impl Display for Ffi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.path)
    }
}
impl PartialEq for Ffi {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
