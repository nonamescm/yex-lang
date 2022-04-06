use crate::{
    env::EnvTable, error::InterpretResult, gc::GcRef, raise, Symbol, Value, VirtualMachine,
};

use super::{fun::Fn, instance::Instance, list, str, table, file};

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

    #[must_use]
    /// Add a initializer to the type.
    pub fn with_initializer(mut self, initializer: GcRef<Fn>) -> Self {
        self.initializer = Some(initializer);
        self
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

        Self::new(Symbol::from("List"), methods, vec![])
            .with_initializer(GcRef::new(Fn::new_native(1, list::methods::init)))
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
        Self::new(Symbol::from("Table"), methods, vec![])
            .with_initializer(GcRef::new(Fn::new_native(1, table::methods::init)))
    }

    /// Creates a new Num type.
    pub fn num() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Num"), methods, vec![])
            .with_initializer(GcRef::new(Fn::new_native(1, |_, _| Ok(Value::Num(0.0)))))
    }

    /// Creates a new Sym type.
    pub fn sym() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Sym"), methods, vec![]).with_initializer(GcRef::new(
            Fn::new_native(1, |_, _| Ok(Symbol::from(":").into())),
        ))
    }

    /// Creates a new Str type.
    pub fn str() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::new("get"),
            Value::Fn(GcRef::new(Fn::new_native(2, str::methods::get))),
        );

        Self::new(Symbol::from("Str"), methods, vec![])
            .with_initializer(GcRef::new(Fn::new_native(1, str::methods::init)))
    }

    /// Creates a new Bool type.
    pub fn bool() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Bool"), methods, vec![])
            .with_initializer(Fn::new_native(1, |_, _| Ok(Value::Bool(false))).to_gcref())
    }

    /// Creates a new Fn type.
    pub fn fun() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Fn"), methods, vec![]).with_initializer(GcRef::new(Fn::new_native(
            1,
            |_, _| {
                Ok(Value::Fn(GcRef::new(Fn::new_native(0, |_, _| {
                    Ok(Value::Nil)
                }))))
            },
        )))
    }

    /// Creates a new Nil type.
    pub fn nil() -> Self {
        let methods = EnvTable::new();
        Self::new(Symbol::from("Nil"), methods, vec![])
            .with_initializer(GcRef::new(Fn::new_native(1, |_, _| Ok(Value::Nil))))
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

        Self::new("File".into(), methods, vec!["path".into()])
    }
}

/// Instantiates a type with the given parameters.
/// Push the new instance to the stack.
pub fn instantiate(
    vm: &mut VirtualMachine,
    ty: GcRef<YexType>,
    args: Vec<Value>,
) -> InterpretResult<()> {
    if args.len() != ty.params.len() {
        raise!("Wrong number of arguments for type instantiation")?;
    }

    let mut fields = EnvTable::new();
    for (i, arg) in args.iter().enumerate() {
        fields.insert(ty.params[i], arg.clone());
    }

    let inst = Value::Instance(GcRef::new(Instance::new(ty.clone(), fields)));
    vm.push(inst);

    if let Some(initializer) = &ty.initializer {
        vm.push(Value::Fn(initializer.clone()));
        vm.call(1)?;
    }

    Ok(())
}
