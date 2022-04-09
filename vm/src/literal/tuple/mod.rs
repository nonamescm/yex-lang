use crate::{gc::GcRef, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Tuple(GcRef<Box<[Value]>>);

impl From<Vec<Value>> for Tuple {
    fn from(vec: Vec<Value>) -> Self {
        Tuple(GcRef::new(vec.into_boxed_slice()))
    }
}

impl Tuple {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl std::fmt::Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({})", self.0.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "))
    }
}
