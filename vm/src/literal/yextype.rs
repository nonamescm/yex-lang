use crate::{env::EnvTable, Symbol};

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
