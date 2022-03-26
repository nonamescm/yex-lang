use crate::{env::EnvTable, gc::GcRef, Symbol, Value};

use super::{fun::Fun, instance::Instance, list};

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

    /// Creates a new List type.
    pub fn list() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("head"),
            Value::Fun(GcRef::new(Fun::new_native(0, list::methods::head))),
        );

        methods.insert(
            Symbol::from("tail"),
            Value::Fun(GcRef::new(Fun::new_native(0, list::methods::tail))),
        );

        methods.insert(
            Symbol::from("map"),
            Value::Fun(GcRef::new(Fun::new_native(1, list::methods::map))),
        );

        methods.insert(
            Symbol::from("filter"),
            Value::Fun(GcRef::new(Fun::new_native(1, list::methods::filter))),
        );

        methods.insert(
            Symbol::from("fold"),
            Value::Fun(GcRef::new(Fun::new_native(2, list::methods::fold))),
        );

        methods.insert(
            Symbol::from("rev"),
            Value::Fun(GcRef::new(Fun::new_native(1, list::methods::rev))),
        );

        methods.insert(
            Symbol::from("get"),
            Value::Fun(GcRef::new(Fun::new_native(1, list::methods::get))),
        );

        Self::new(Symbol::from("List"), methods, vec![])
    }

    /// Creates a new Num type.
    pub fn num() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Num"), methods, vec![])
    }

    /// Creates a new Sym type.
    pub fn sym() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Sym"), methods, vec![])
    }

    /// Creates a new Str type.
    pub fn str() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Str"), methods, vec![])
    }

    /// Creates a new Bool type.
    pub fn bool() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Bool"), methods, vec![])
    }

    /// Creates a new Fun type.
    pub fn fun() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Fun"), methods, vec![])
    }

    /// Creates a new Nil type.
    pub fn nil() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Nil"), methods, vec![])
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
