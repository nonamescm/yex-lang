use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use crate::gc::GcRef;
#[derive(Debug, Clone)]
pub struct UserData(pub GcRef<Box<dyn Any>>);

impl PartialEq for UserData {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Deref for UserData {
    type Target = dyn Any;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[test]
fn userdata_ne() {
    let item = UserData(GcRef::new(Box::new(())));
    assert_ne!(item, item);
}
