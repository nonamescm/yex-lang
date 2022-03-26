use crate::{gc::GcRef, EnvTable, YexType};

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
}
