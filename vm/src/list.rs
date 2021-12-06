use crate::Constant;

type Link = Option<Box<Node>>;
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
    pub fn prepend(&self, elem: Constant) -> Self {
        let node = Box::new(Node {
            elem,
            next: self.head.clone(),
        });
        Self { head: Some(node) }
    }

    /// Returns the list tail
    pub fn tail(&self) -> Self {
        let tail = self.head.as_ref().map(|node| node.as_ref().next.clone());
        let tail = match tail {
            Some(v) => v,
            None => None,
        };

        Self { head: tail }
    }

    /// Returns the current element
    pub fn head(&self) -> Option<&Constant> {
        self.head.as_ref().map(|node| Some(&node.as_ref().elem))?
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = loop {
            let mut str = String::from('[');
            str.push_str(&match self.head() {
                Some(s) => if self.tail().is_empty() {
                    format!("{}", s)
                } else {
                    format!("{}, ", s)
                }
                None => break str + "]",
            });

            let mut head = self.tail();
            while head.head() != None {
                if head.tail().is_empty() {
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