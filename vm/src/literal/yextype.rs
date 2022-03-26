use crate::{env::EnvTable, gc::GcRef, Symbol, Value};

use super::instance::Instance;

#[derive(Debug, PartialEq)]
/// A Yex user-defined type.
pub struct YexType {
    /// The name of the type.
    pub name: Symbol,
    /// The fields of the type (methods).
    pub fields: EnvTable,
    /// The parameters that the type needs to be instantiated.
    pub params: Vec<Symbol>,
}

impl YexType {
    /// Creates a new Yex type.
    pub fn new(name: Symbol, fields: EnvTable, params: Vec<Symbol>) -> Self {
        Self {
            name,
            fields,
            params,
        }
    }
}

/// Instantiates a type with the given parameters.
pub fn instantiate(ty: GcRef<YexType>, args: Vec<Value>) -> Value {
    if args.len() != ty.params.len() {
        panic!("Wrong number of arguments for type instantiation");
    }

    let mut fields = EnvTable::new();
    for (i, arg) in args.iter().enumerate() {
        fields.insert(ty.params[i], arg.clone());
    }

    Value::Instance(GcRef::new(Instance::new(ty, fields)))
}
