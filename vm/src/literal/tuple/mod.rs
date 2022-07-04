pub mod methods;

use crate::{gc::GcRef, Value};

#[derive(Debug, PartialEq, Clone)]
/// A yex tuple
pub struct Tuple(pub GcRef<Box<[Value]>>);

impl From<Vec<Value>> for Tuple {
    fn from(vec: Vec<Value>) -> Self {
        Tuple(GcRef::new(vec.into_boxed_slice()))
    }
}

impl Tuple {
    /// Returns the length of the tuple
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// checks if self is `unit` (a.k.a. empty tuple)
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.0
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
