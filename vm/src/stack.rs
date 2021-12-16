use std::mem::MaybeUninit;

#[derive(Debug)]
/// A wrapper around an array armazenated on the stack
pub struct StackVec<T, const S: usize> {
    len: usize,
    array: [MaybeUninit<T>; S],
}

impl<T, const S: usize> StackVec<T, S> {
    const UNINIT: MaybeUninit<T> = MaybeUninit::uninit();
    const ARRAY_INIT: [MaybeUninit<T>; S] = [Self::UNINIT; S];
    pub fn new() -> Self {
        Self {
            len: 0,
            array: Self::ARRAY_INIT,
        }
    }

    #[track_caller]
    pub fn push(&mut self, new_value: T) {
        if self.len >= S {
            panic!(
                "Index out of bounds, the len is {} but the index is {}",
                S, self.len
            )
        }
        self.array[self.len].write(new_value);
        self.len += 1;
    }

    #[track_caller]
    pub fn pop(&mut self) -> T {
        if self.len() == 0 {
            panic!("Called pop() on a array with no elements");
        }

        self.len -= 1;
        unsafe { std::mem::replace(&mut self.array[self.len], MaybeUninit::uninit()).assume_init() }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    #[track_caller]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + DoubleEndedIterator {
        self.array[0..self.len]
            .iter_mut()
            .map(|it| unsafe { it.assume_init_mut() })
    }

    #[track_caller]
    pub fn iter(&self) -> impl Iterator<Item = &T> + DoubleEndedIterator {
        self.array[0..self.len]
            .iter()
            .map(|it| unsafe { it.assume_init_ref() })
    }

    #[allow(dead_code)]
    #[track_caller]
    pub fn remove(&mut self, index: usize) {
        if index >= S {
            panic!(
                "Index out of bounds, the len is {} but the index is {}",
                S, self.len
            )
        }

        self.array[index..self.len].rotate_left(1);
        self.pop();
    }

    #[track_caller]
    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(&self[self.len - 1])
        }
    }

    #[allow(dead_code)]
    #[track_caller]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        } else {
            let idx = self.len - 1;
            Some(&mut self[idx])
        }
    }
}

impl<T, const S: usize> std::ops::Index<usize> for StackVec<T, S> {
    type Output = T;
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        if self.len <= index {
            panic!(
                "index out of bounds, the len is `{}` but the index is `{}`",
                self.len, index
            )
        }
        unsafe { self.array[index].assume_init_ref() }
    }
}

impl<T, const S: usize> std::ops::IndexMut<usize> for StackVec<T, S> {
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut T {
        if self.len <= index {
            panic!(
                "index out of bounds, the len is `{}` but the index is `{}`",
                self.len, index
            )
        }
        unsafe { self.array[index].assume_init_mut() }
    }
}

impl<T: std::fmt::Display, const S: usize> std::fmt::Display for StackVec<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, value) in self.iter().enumerate() {
            write!(f, "{}", value)?;
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
