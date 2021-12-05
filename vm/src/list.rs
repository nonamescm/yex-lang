use crate::Constant;

type Link = Option<*const Node>;
#[derive(Clone, Debug, PartialEq)]
/// Yex lists implementation
pub struct List {
    head: Link,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    elem: Constant,
    next: Link,
}

fn raw_alloc<T>(t: T) -> *const T {
    Box::into_raw(Box::new(t))
}

impl List {
    /// Creates a List
    pub fn new() -> Self {
        Self { head: None }
    }

    /// Prepends a value to the end, returning the list
    pub fn prepend(&self, elem: Constant) -> Self {
        let node = raw_alloc(Node {
            elem,
            next: self.head.clone(),
        });
        Self { head: Some(node) }
    }

    /// Returns the list tail
    pub fn tail(&self) -> Self {
        let tail = self
            .head
            .as_ref()
            .map(|node| unsafe { node.as_ref()?.next.clone() });
        let tail = match tail {
            Some(v) => v,
            None => None,
        };

        Self { head: tail }
    }

    /// Returns the current element
    pub fn head(&self) -> Option<&Constant> {
        self.head
            .as_ref()
            .map(|node| unsafe { Some(&node.as_ref()?.elem) })?
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = loop {
            let mut str = String::from('[');
            str.push_str(&match self.head() {
                Some(s) => format!("{}, ", s),
                None => break str,
            });

            let mut head = self.tail();
            while head.head() != None {
                if head.tail() == Self::new() {
                    str.push_str(&format!("{}", head.head().unwrap()));
                } else {
                    str.push_str(&format!("{}, ", head.head().unwrap()));
                }
                head = head.tail();
            }

            str.push(']');

            break str;
        };

        write!(f, "{}", str)
    }
}
