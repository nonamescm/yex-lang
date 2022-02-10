use std::{
    fmt::Formatter,
    hash::{Hash, Hasher},
};

/// Symbol struct, contains the symbol string and a pre-hashed value for faster comparison
#[derive(Clone, Copy)]
pub struct Symbol {
    string: &'static str,
    pub(crate) hash: usize,
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash)
    }
}

impl std::cmp::PartialEq for Symbol {
    fn eq(&self, rhs: &Self) -> bool {
        self.hash == rhs.hash
    }
}

impl Eq for Symbol {}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sym = ":".to_string() + self.string;
        write!(f, "{}", sym)?;
        Ok(())
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({})", self.string)
    }
}

impl Symbol {
    /// Creates a new symbol
    pub fn new<T: Into<String>>(str: T) -> Self {
        let str = str.into();

        let mut hash: usize = 2166136261;
        for b in str.bytes() {
            hash ^= b as usize;
            hash = hash.wrapping_mul(16777619);
        }

        Self {
            string: Box::leak(str.into_boxed_str()),
            hash,
        }
    }

    /// Returns the intern symbol str
    pub fn to_str(&self) -> &str {
        self.string
    }
}

impl<T: AsRef<str>> From<T> for Symbol {
    fn from(str: T) -> Self {
        Self::new(str.as_ref())
    }
}
