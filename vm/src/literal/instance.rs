use crate::{gc::GcRef, EnvTable, Value, YexModule};

#[derive(Debug, PartialEq)]
pub struct Instance {
    pub ty: GcRef<YexModule>,
    pub fields: EnvTable,
}

impl Instance {
    /// Create a new instance
    pub fn new(ty: GcRef<YexModule>, fields: EnvTable) -> Self {
        Instance { ty, fields }
    }

    /// Get the field value
    pub fn get_field<S: AsRef<str>>(&self, name: S) -> Value {
        self.fields.get(&name.as_ref().into()).unwrap()
    }
}
