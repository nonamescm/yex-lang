use std::fmt::Display;

use super::TryGet;
use crate::{List, Symbol, Value};

pub mod methods;
#[derive(PartialEq, Debug, Clone)]
pub struct Table {
    pub items: List,
}
impl Default for Table {
    fn default() -> Self {
        Self { items: List::new() }
    }
}
impl Table {
    #[inline]
    pub fn new() -> Table {
        Self::default()
    }
    #[inline(always)]
    fn from_list(items: List) -> Table {
        Table { items }
    }
    #[inline]
    #[must_use]
    pub fn insert(self, key: Symbol, val: Value) -> Self {
        Self::from_list(self.items.prepend(Value::List(
            List::new().prepend(val).prepend(Value::Sym(key.into())),
        )))
    }
    #[inline]
    #[must_use]
    pub fn get(&self, key: Symbol) -> Option<Value> {
        self.items
            .iter()
            .find(|item| {
                let list: List = item.get().unwrap();
                list.index(0) == Value::Sym(key.into())
            })
            .map(|x| {
                let xs: List = x.get().unwrap();
                xs.index(1)
            })
    }
}
impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            let list: List = item.get().unwrap();
            write!(f, "{}", list.index(0))?;
            write!(f, "=")?;
            write!(f, "{}", list.index(1))?;
        }
        write!(f, "}}")
    }
}
