use std::fmt::Display;

use super::{tuple::Tuple, TryGet};
use crate::{gc::GcRef, List, Symbol, Value, YexModule};

pub mod methods;
#[derive(Debug, Clone)]
/// A Struct is a collection of fields.
pub struct YexStruct {
    /// The fields of the struct.
    pub items: List,
    /// The module of the struct.
    pub module: GcRef<YexModule>,
}

impl Default for YexStruct {
    fn default() -> Self {
        Self {
            items: List::new(),
            module: GcRef::new(YexModule::struct_()),
        }
    }
}

impl YexStruct {
    #[inline]
    /// Creates a new struct.
    pub fn new(module: GcRef<YexModule>) -> YexStruct {
        let items = List::new();
        Self { items, module }
    }

    #[inline(always)]
    fn from_list(items: List) -> YexStruct {
        YexStruct {
            items,
            module: GcRef::new(YexModule::struct_()),
        }
    }

    #[inline]
    #[must_use]
    /// Creates a new struct with the given field + the current ones.
    pub fn insert(self, key: Symbol, val: Value) -> Self {
        Self::from_list(
            self.items
                .prepend(Value::Tuple(vec![key.into(), val].into())),
        )
    }

    #[inline]
    #[must_use]
    /// gets the value of the given field.
    pub fn get(&self, key: Symbol) -> Value {
        self.items
            .iter()
            .find(|item| {
                let list: Tuple = item.get().unwrap();
                list.0[0] == Value::Sym(key.into())
            })
            .map(|x| {
                let xs: Tuple = x.get().unwrap();
                xs.0[1].clone()
            })
            .unwrap_or(Value::Nil)
    }
}

impl Display for YexStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}{{", self.module.name.as_str())?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            let list: Tuple = item.get().unwrap();
            let key: Symbol = list.0[0].get().unwrap();
            write!(f, "{}", key.as_str())?;
            write!(f, ":")?;
            write!(f, " {}", list.0[1])?;
        }
        write!(f, "}}")
    }
}

impl PartialEq for YexStruct {
    fn eq(&self, other: &Self) -> bool {
        self.items
            .iter()
            .all(|item| other.items.iter().any(|other_item| item == other_item))
    }
}
