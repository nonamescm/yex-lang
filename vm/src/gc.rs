use std::{cell::Cell, fmt::Debug, ptr::NonNull};

struct Ref<T> {
    pub(in crate::gc) inner: T,
    pub(in crate::gc) count: Cell<usize>,
}

pub struct GcRef<T> {
    inner: NonNull<Ref<T>>,
}

impl<T> GcRef<T> {
    pub fn new(constant: T) -> Self {
        // SAFETY:
        // We pass the box to into_raw after the allocation, everything is properly aligned and
        // nothing can be null
        unsafe {
            Self {
                inner: NonNull::new_unchecked(Box::into_raw(Box::new(Ref {
                    inner: constant,
                    count: Cell::new(1),
                }))),
            }
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

// impl<T> std::ops::DerefMut for GcRef<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { &mut self.inner.as_mut().inner }
//     }
// }

impl<T> std::ops::Deref for GcRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &self.inner.as_ref().inner }
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

impl<T: Debug> Debug for GcRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.get())
    }
}
