use std::any::Any;

use crate::gc::GcRef;
#[derive(Debug, Clone)]
pub struct UserData(pub GcRef<Box<dyn Any>>);
impl UserData {
    pub fn new<T>(val: T) -> Self
    where
        T: Any,
    {
        Self(GcRef::new(Box::new(val)))
    }
}
impl PartialEq for UserData {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[test]
fn userdata_ne() {
    let item = UserData::new(10);
    assert_ne!(item, item);
}
#[test]
fn userdata_basic() {
    let item = UserData::new(10_i32);
    assert!(item.0.downcast_ref::<i32>().is_some());
}
#[test]
fn userdata_cmp() {
    let item = UserData::new(String::from("Hello"));
    assert_eq!(item.0.downcast_ref(), Some(&String::from("Hello")));
}
