use crate::{gc::GcRef, EnvTable, Value, YexType};

#[derive(Debug, PartialEq)]
pub struct Instance {
    pub ty: GcRef<YexType>,
    pub fields: EnvTable,
}

impl Instance {
    /// Create a new instance
    pub fn new(ty: GcRef<YexType>, fields: EnvTable) -> Self {
        Instance { ty, fields }
    }

    /// Get the field value
    pub fn get_field<S: AsRef<str>>(&self, name: S) -> Value {
        self.fields.get(&name.as_ref().into()).unwrap()
    }
}
