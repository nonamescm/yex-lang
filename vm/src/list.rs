use crate::{
    gc::GcRef,
    literal::{nil, ConstantRef},
};

type Link = Option<GcRef<Node>>;
#[derive(Clone, Debug, PartialEq)]
/// Yex lists implementation
pub struct List {
    head: Link,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    elem: ConstantRef,
    next: Link,
}

impl List {
    /// Creates a List
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Checks if the list is empty
    pub fn is_empty(&self) -> bool {
        self.head == None
    }

    /// Prepends a value to the end, returning the list
    pub fn prepend(&self, elem: ConstantRef) -> Self {
        let node = GcRef::new(Node {
            elem,
            next: self.head.clone(),
        });
        Self { head: Some(node) }
    }

    /// Returns the list tail
    pub fn tail(&self) -> Self {
        let tail = self.head.as_ref().map(|node| node.get().next.clone());
        let tail = match tail {
            Some(v) => v,
            None => None,
        };

        Self { head: tail }
    }

    /// Returns the current element
    pub fn head(&self) -> Option<ConstantRef> {
        self.head
            .as_ref()
            .map(|node| GcRef::clone(&node.get().elem))
    }

    /// Returns a index into the list
    pub fn index(&self, index: usize) -> ConstantRef {
        if index == 0 {
            self.head().unwrap_or_else(nil)
        } else {
            let tail = self.tail();
            if tail.is_empty() {
                nil()
            } else {
                tail.index(index - 1)
            }
        }
    }

    /// Returns the list length
    pub fn len(&self) -> usize {
        let mut xs = self.head.as_ref();
        let mut count = 0;
        while xs != None {
            xs = xs.unwrap().get().next.as_ref();
            count += 1;
        }
        count
    }

    /// Converts list to Vec
    pub fn to_vec(&self) -> Vec<ConstantRef> {
        let mut vec = vec![];
        let mut head = self.clone();
        while head.head().is_some() {
            vec.push(head.head().unwrap().to_owned());
            head = head.tail();
        }
        vec
    }

    /// Iterate over all elements of `self`
    pub fn iter(&self) -> Iter {
        Iter {
            next: self.head.as_ref().map(|node| GcRef::clone(node)),
        }
    }

    /// Reverses `self` without consuming it
    pub fn rev(&self) -> Self {
        let mut node = self.head.as_ref();
        let mut list = Self::new();
        while let Some(elem) = node {
            list = list.prepend(GcRef::clone(&elem.get().elem));
            node = elem.get().next.as_ref()
        }
        list
    }
}

impl std::fmt::Display for List {
    #[allow(clippy::never_loop)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = loop {
            let mut str = String::from('[');
            str.push_str(&match self.head() {
                Some(s) => {
                    if self.tail().is_empty() {
                        format!("{}", s.get())
                    } else {
                        format!("{}, ", s.get())
                    }
                }
                None => break str + "]",
            });

            let mut head = self.tail();
            while head.head() != None {
                if head.tail().is_empty() {
                    str.push_str(&format!("{}", head.head().unwrap().get()));
                } else {
                    str.push_str(&format!("{}, ", head.head().unwrap().get()));
                }
                head = head.tail();
            }

            str.push(']');

            break str;
        };

        write!(f, "{}", str)
    }
}

pub struct Iter {
    next: Option<GcRef<Node>>,
}

impl Iterator for Iter {
    type Item = ConstantRef;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.clone().map(|node| {
            self.next = node.get().next.clone();
            node.get().elem.clone()
        })
    }
}

impl FromIterator<ConstantRef> for List {
    fn from_iter<T: IntoIterator<Item = ConstantRef>>(iter: T) -> Self {
        let mut list = Self::new();
        for item in iter.into_iter() {
            list = list.prepend(item)
        }
        list
    }
}
