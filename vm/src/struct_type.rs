//all YexType related things


use crate::{env::EnvTable, Symbol};
use crate::literal::Value;


/// A struct type.
#[derive(PartialEq, Debug, Clone)]
pub struct YexType {
    fields: EnvTable,
    name: Symbol
}

impl YexType {
    /// creates a new struct type
    pub fn new(fields: EnvTable, name: Symbol) -> YexType {
        Self {
            fields,
            name: name
        }
    }
    /// gets a field from it, including functions
    pub fn get_field(&self, f: Symbol) -> Option<Value> {
        self.fields.get(&f)
    }
    /// gets the name of the type   
    pub fn get_name(&self) -> &str {
        self.name.to_str()
    }
}

impl std::fmt::Display for YexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name);
        write!(f, "{}", self.fields)        
    }
}