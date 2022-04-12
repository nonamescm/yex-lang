use std::{mem::{self, MaybeUninit}, ops::Deref};

/// A wrapper around an array armazenated on the stack
pub struct StackVec<T, const S: usize> {
    len: usize,
    pub(super) array: [MaybeUninit<T>; S],
}

impl<T, const S: usize> StackVec<T, S> {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();
    const ARRAY_INIT: [MaybeUninit<T>; S] = [Self::UNINIT; S];
    /// Creates a new StackVec
    pub const fn new() -> Self {
        Self {
            len: 0,
            array: Self::ARRAY_INIT,
        }
    }

    #[track_caller]
    #[inline]
    /// Push a new element to the array
    pub fn push(&mut self, new_value: T) {
        self.array[self.len].write(new_value);
        self.len += 1;
    }

    #[track_caller]
    #[inline]
    /// Pop's the last element
    pub fn pop(&mut self) -> T {
        self.len -= 1;
        unsafe { mem::replace(&mut self.array[self.len], MaybeUninit::uninit()).assume_init() }
    }

    /// Returns the StackVec length
    pub fn len(&self) -> usize {
        self.len
    }

    /// checks if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[track_caller]
    /// Returns an iterator of mutable references to the elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + DoubleEndedIterator {
        self.array[0..self.len]
            .iter_mut()
            .map(|it| unsafe { it.assume_init_mut() })
    }

    #[track_caller]
    /// Returns an iterator of references to the elements
    pub fn iter(&self) -> impl Iterator<Item = &T> + DoubleEndedIterator {
        self.array[0..self.len]
            .iter()
            .map(|it| unsafe { it.assume_init_ref() })
    }

    #[track_caller]
    #[inline]
    /// Removes the element at the given index
    pub fn remove(&mut self, index: usize) {
        self.array[index..self.len].rotate_left(1);
        self.pop();
    }

    /// Reverses the StackVec in place
    #[must_use]
    #[track_caller]
    pub fn reverse(self) -> Self {
        self.into_iter().rev().collect()
    }

    #[track_caller]
    /// Returns a reference to the last element
    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(self.array[self.len - 1].assume_init_ref()) }
        }
    }

    #[track_caller]
    /// Returns a mutable reference to the last element
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            let idx = self.len - 1;
            unsafe { Some(self.array[idx].assume_init_mut()) }
        }
    }

    #[track_caller]
    /// Inserts an element at a given index
    /// # Safety
    /// This function is unsafe because it does not check if the index is out of bounds, it's up to
    /// the caller to make sure that the index is valid or to manually resize the array if needed
    pub unsafe fn insert_at(&mut self, idx: usize, value: T) {
        self.array[idx].write(value);
    }

    #[track_caller]
    /// Manually updates the length of the StackVec
    /// # Safety
    /// This function doesn't check if the new length is valid, it's up to the caller to ensure
    /// that the new length is valid and all the elements are initialized
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }
}

impl<T: std::fmt::Debug, const S: usize> std::fmt::Debug for StackVec<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, value) in self.iter().enumerate() {
            write!(f, "{:?}", value)?;
            if index < self.len - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl<T: PartialEq, const S: usize> PartialEq for StackVec<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.iter()
            .zip(other.iter())
            .fold(true, |acc, (this, other)| acc && this == other)
            && self.len() == other.len()
    }
}

pub struct IntoIter<T, const S: usize> {
    array: StackVec<T, S>,
    next: usize,
}

impl<T, const S: usize> Iterator for IntoIter<T, S> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.array.len() {
            let item = unsafe {
                (self.array.array.get_unchecked_mut(self.next).as_ptr() as *const T).read()
            };
            self.next += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T, const S: usize> DoubleEndedIterator for IntoIter<T, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.array.is_empty() {
            Some(self.array.pop())
        } else {
            None
        }
    }
}

#[macro_export]
/// Creates a new StackVec
macro_rules! stackvec {
    ($ty: ty; $len: expr) => {
        $crate::StackVec::<$ty, $len>::new()
    };

    () => {
        $crate::StackVec::new()
    };

    ($($elems: expr),*) => {{
        let mut stackvec = $crate::StackVec::new();
        $({
            stackvec.push($elems)
        });*
        stackvec
    }}
}

impl<T, const S: usize> FromIterator<T> for StackVec<T, S> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut stackvec = Self::new();
        for it in iter {
            stackvec.push(it)
        }
        stackvec
    }
}

impl<T: Clone, const S: usize> Clone for StackVec<T, S> {
    fn clone(&self) -> Self {
        self.iter().cloned().collect::<Self>()
    }
}

impl<T, const S: usize> Default for StackVec<T, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const S: usize> IntoIterator for StackVec<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T, S>;

    /// Consumes the StackVec, returing an iterator over it's elements
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            array: self,
            next: 0,
        }
    }
}

impl<T, const S: usize> From<StackVec<T, S>> for Vec<T> {
    fn from(stackvec: StackVec<T, S>) -> Self {
        stackvec.into_iter().collect()
    }
}

impl<T, const S: usize> Deref for StackVec<T, S> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.array[0..self.len]) }
    }
}
