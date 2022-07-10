use crate::{env::EnvTable, gc::GcRef, literal, Symbol, Value};

use super::{fun::Fn, list, str, tuple};

#[derive(Debug, PartialEq, Default)]
/// A Yex user-defined type.
pub struct YexModule {
    /// Module name.
    pub name: Symbol,
    /// Module functions.
    pub fields: EnvTable,
}
#[macro_export]
/// Add Fields/Methods to a `YexModule`
macro_rules! fields {
    ($sname:expr => {
        $(
          $name:expr => $arg_count:expr$(,)?
        ),*
    }, $methods:ident) => {
        $(
            $methods.insert(
                Symbol::from(stringify!($name)),
                $crate::literal::Value::Fn($crate::gc::GcRef::new($crate::literal::Fn::new_native($arg_count, |_, args| {
                    let this: GcRef<YexModule> = GcRef::new(YexModule::default());
                    let tup = Tuple(GcRef::new(args.into_boxed_slice()));
                    let value = Value::Tagged(this, Symbol::from(concat!(stringify!($sname), ".", stringify!($name))), tup);
                    Ok(value)
                }))),
            );
         )*
    };
    ($sname:expr => {
        $(
          $name:tt @ $func:expr => $arg_count:expr$(,)?
        ),*
    }, $methods:ident) => {
        $(
            $methods.insert(
                Symbol::from(stringify!($name)),
                Value::Fn(GcRef::new(Fn::new_native($arg_count, $func))),
            );
         )*
    };}

impl YexModule {
    /// Creates a new Yex type.
    #[must_use]
    pub fn new(name: Symbol, fields: EnvTable) -> Self {
        Self { name, fields }
    }

    /// Creates a new List type.
    #[must_use]
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
            Value::Fn(GcRef::new(Fn::new_native(1, list::methods::rev))),
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

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, list::methods::show))),
        );

        methods.insert(
            Symbol::from("toList"),
            Value::Fn(GcRef::new(Fn::new_native(1, list::methods::to_list))),
        );

        Self::new(Symbol::from("List"), methods)
    }

    /// Creates a new Tuple type.
    #[must_use]
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

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, tuple::methods::show))),
        );

        Self::new(Symbol::from("Tuple"), methods)
    }

    /// Creates a new Num type.
    #[must_use]
    pub fn num() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );

        Self::new(Symbol::from("Num"), methods)
    }

    /// Creates a new Sym type.
    #[must_use]
    pub fn sym() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );

        Self::new(Symbol::from("Sym"), methods)
    }

    /// Creates a new Str type.
    #[must_use]
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
            Symbol::new("toList"),
            Value::Fn(GcRef::new(Fn::new_native(1, str::methods::chars))),
        );

        methods.insert(
            Symbol::new("len"),
            Value::Fn(GcRef::new(Fn::new_native(1, str::methods::len))),
        );

        methods.insert(
            Symbol::new("new"),
            Value::Fn(GcRef::new(Fn::new_native(0, str::methods::new))),
        );

        methods.insert(
            Symbol::new("ord"),
            Value::Fn(GcRef::new(Fn::new_native(1, str::methods::ord))),
        );

        methods.insert(
            Symbol::new("chr"),
            Value::Fn(GcRef::new(Fn::new_native(1, str::methods::chr))),
        );

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );

        Self::new(Symbol::from("Str"), methods)
    }

    /// Creates a new Bool type.
    #[must_use]
    pub fn bool() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );

        Self::new(Symbol::from("Bool"), methods)
    }

    /// Creates a new Fn type.
    #[must_use]
    pub fn fun() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );

        Self::new(Symbol::from("Fn"), methods)
    }
    /// Creates a new Result type
    #[must_use]
    pub fn result() -> Self {
        let mut methods = EnvTable::new();
        fields!(Result => {
            ok   @  literal::result::vm_ok => 1,
            fail @  literal::result::vm_fail => 1,
        }, methods);
        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );
        Self::new(Symbol::from("Result"), methods)
    }
    /// Generates a new FFI type
    #[must_use]
    pub fn ffi() -> Self {
        let mut methods = EnvTable::new();
        fields!(FFI => {
            open @ literal::ffi::methods::open => 1,
            get  @ literal::ffi::methods::get => 2,
        }, methods);
        Self::new(Symbol::from("FFI"), methods)
    }
    /// Creates a new Nil type.
    #[must_use]
    pub fn nil() -> Self {
        let mut methods = EnvTable::new();

        methods.insert(
            Symbol::from("show"),
            Value::Fn(GcRef::new(Fn::new_native(1, |vm, x| {
                super::show(vm, x).map(std::convert::Into::into)
            }))),
        );

        Self::new(Symbol::from("Nil"), methods)
    }
}
