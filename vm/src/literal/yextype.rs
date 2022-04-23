use crate::{env::EnvTable, error::InterpretResult, gc::GcRef, Symbol, Value, VirtualMachine};

use super::{file, fun::Fn, instance::Instance, list, nil, str, table, tuple};

#[derive(Debug, PartialEq)]
/// A Yex user-defined type.
pub struct YexType {
    /// The name of the type.
    pub name: Symbol,
    /// The fields of the type (methods).
    pub fields: EnvTable,
    /// The parameters that the type needs to be instantiated.
    pub params: Vec<Symbol>,
    /// The method that runs after the type is instantiated.
    pub initializer: Option<GcRef<Fn>>,
}

impl YexType {
    /// Creates a new Yex type.
    pub fn new(name: Symbol, fields: EnvTable, params: Vec<Symbol>) -> Self {
        Self {
            name,
            fields,
            params,
            initializer: None,
        }
    }

    /// Creates a new List type.
    pub fn list() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("head"),
            Value::Fn(GcRef::new(Fn::new_native(1, list::methods::head))),
        );

        methods.insert(
            Symbol::from("tail"),
            Value::Fn(GcRef::new(Fn::new_native(1, list::methods::tail))),
        );

        methods.insert(
            Symbol::from("map"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::map))),
        );

        methods.insert(
            Symbol::from("filter"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::filter))),
        );

        methods.insert(
            Symbol::from("fold"),
            Value::Fn(GcRef::new(Fn::new_native(3, list::methods::fold))),
        );

        methods.insert(
            Symbol::from("rev"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::rev))),
        );

        methods.insert(
            Symbol::from("get"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::get))),
        );

        methods.insert(
            Symbol::new("drop"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::drop))),
        );

        methods.insert(
            Symbol::new("join"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::join))),
        );

        methods.insert(
            Symbol::from("find"),
            Value::Fn(GcRef::new(Fn::new_native(2, list::methods::find))),
        );

        methods.insert(
            Symbol::from("len"),
            Value::Fn(GcRef::new(Fn::new_native(1, list::methods::len))),
        );

        methods.insert(
            Symbol::from("new"),
            Value::Fn(GcRef::new(Fn::new_native(0, list::methods::new))),
        );

        Self::new(Symbol::from("List"), methods, vec![])
    }

    /// Creates a new Table type.
    pub fn table() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("get"),
            Value::Fn(GcRef::new(Fn::new_native(2, table::methods::get))),
        );

        methods.insert(
            Symbol::from("insert"),
            Value::Fn(GcRef::new(Fn::new_native(3, table::methods::insert))),
        );

        methods.insert(
            Symbol::from("new"),
            Value::Fn(GcRef::new(Fn::new_native(0, table::methods::new))),
        );

        Self::new(Symbol::from("Table"), methods, vec![])
    }

    /// Creates a new Tuple type.
    pub fn tuple() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("get"),
            Value::Fn(GcRef::new(Fn::new_native(2, tuple::methods::get))),
        );

        methods.insert(
            Symbol::from("new"),
            Value::Fn(GcRef::new(Fn::new_native(0, tuple::methods::new))),
        );

        Self::new(Symbol::from("Tuple"), methods, vec![])
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
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::new("get"),
            Value::Fn(GcRef::new(Fn::new_native(2, str::methods::get))),
        );

        methods.insert(
            Symbol::new("split"),
            Value::Fn(GcRef::new(Fn::new_native(2, str::methods::split))),
        );

        methods.insert(
            Symbol::new("fmt"),
            Value::Fn(GcRef::new(Fn::new_native(2, str::methods::fmt))),
        );

        methods.insert(
            Symbol::new("len"),
            Value::Fn(GcRef::new(Fn::new_native(1, str::methods::len))),
        );

        methods.insert(
            Symbol::new("new"),
            Value::Fn(GcRef::new(Fn::new_native(0, str::methods::new))),
        );

        Self::new(Symbol::from("Str"), methods, vec![])
    }

    /// Creates a new Bool type.
    pub fn bool() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Bool"), methods, vec![])
    }

    /// Creates a new Fn type.
    pub fn fun() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Fn"), methods, vec![])
    }

    /// Creates a new Nil type.
    pub fn nil() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Nil"), methods, vec![])
    }

    /// Creates a new File type.
    pub fn file() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::new("read"),
            Value::Fn(GcRef::new(Fn::new_native(1, file::methods::read))),
        );

        methods.insert(
            Symbol::new("write"),
            Value::Fn(GcRef::new(Fn::new_native(2, file::methods::write))),
        );

        methods.insert(
            Symbol::new("append"),
            Value::Fn(GcRef::new(Fn::new_native(2, file::methods::append))),
        );

        methods.insert(
            Symbol::new("delete"),
            Value::Fn(GcRef::new(Fn::new_native(1, file::methods::delete))),
        );

        methods.insert(
            Symbol::new("create"),
            Value::Fn(GcRef::new(Fn::new_native(1, file::methods::create))),
        );

        methods.insert(
            Symbol::new("new"),
            Value::Fn(GcRef::new(Fn::new_native(1, file::methods::new))),
        );

        Self::new("File".into(), methods, vec!["path".into()])
    }
}

/// Instantiates a type with the given parameters.
/// Push the new instance to the stack.
pub fn instantiate(vm: &mut VirtualMachine, ty: GcRef<YexType>) -> InterpretResult<()> {
    let mut fields = EnvTable::new();
    for entry in ty.params.iter() {
        fields.insert(*entry, nil());
    }

    let inst = Value::Instance(GcRef::new(Instance::new(ty, fields)));
    vm.push(inst);

    Ok(())
}
