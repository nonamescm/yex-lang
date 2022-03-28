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

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.ty.name)?;
        for (len, (_, value)) in self.fields.iter().enumerate() {
            write!(f, "{}", value)?;
            if len != self.fields.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, ")")
    }
}
