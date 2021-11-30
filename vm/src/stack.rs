use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct Stack<T, const S: usize> {
    len: usize,
    array: [MaybeUninit<T>; S],
}

impl<T, const S: usize> Stack<T, S> {
    pub fn new() -> Self {
        Self {
            len: 0,
            array: [(); S].map(|_| MaybeUninit::uninit()),
        }
    }

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

    #[track_caller]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.array[0..self.len]
            .iter_mut()
            .filter_map(|it| unsafe { Some(it.assume_init_mut()) })
    }

    #[track_caller]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.array[0..self.len]
            .iter()
            .filter_map(|it| unsafe { Some(it.assume_init_ref()) })
    }

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
}

impl<T, const S: usize> std::ops::Index<usize> for Stack<T, S> {
    type Output = T;
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

impl<T: std::fmt::Display, const S: usize> std::fmt::Display for Stack<T, S> {
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
