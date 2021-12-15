use std::{cell::Cell, ptr::NonNull};

#[derive(Debug)]
struct Ref<T> {
    pub(in crate::gc) inner: T,
    pub(in crate::gc) count: Cell<usize>,
}

#[derive(Debug)]
pub struct GcRef<T> {
    inner: NonNull<Ref<T>>,
}

impl<T> GcRef<T> {
    pub fn new(constant: T) -> Self {
        Self {
            inner: NonNull::new(Box::into_raw(Box::new(Ref {
                inner: constant,
                count: Cell::new(1),
            })))
            .unwrap(),
        }
    }

    fn from_inner(inner: NonNull<Ref<T>>) -> Self {
        Self { inner }
    }

    fn inc_ref(&self) {
        unsafe {
            let ref_count = self.ref_count();
            self.inner.as_ref().count.set(ref_count + 1)
        }
    }

    fn dec_ref(&self) {
        unsafe {
            let ref_count = self.ref_count();
            self.inner.as_ref().count.set(ref_count - 1)
        }
    }

    fn ref_count(&self) -> usize {
        unsafe { self.inner.as_ref().count.get() }
    }

    pub fn get(&self) -> &T {
        unsafe { &self.inner.as_ref().inner }
    }
}

impl<T> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        self.inc_ref();
        Self::from_inner(self.inner)
    }
}

impl<T> Drop for GcRef<T> {
    fn drop(&mut self) {
        self.dec_ref();

        if self.ref_count() == 0 {
            unsafe { drop(Box::from_raw(self.inner.as_ptr())) };
        }
    }
}

impl<T: PartialEq> PartialEq for GcRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for GcRef<T> {}
