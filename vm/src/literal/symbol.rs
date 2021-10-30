use std::{
    collections::hash_map::DefaultHasher,
    fmt::Formatter,
    hash::{Hash, Hasher},
};

/// Symbol struct, contains the symbol string and a pre-hashed value for faster comparison
#[derive(Debug, Clone)]
pub struct Symbol {
    string: String,
    hash: u64,
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
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
        let sym = ":".to_string() + &self.string;
        write!(f, "{}", sym)?;
        Ok(())
    }
}

impl Symbol {
    /// Creates a new symbol
    pub fn new<T: Into<String>>(str: T) -> Self {
        let string = str.into();

        let mut hash = DefaultHasher::new();
        string.hash(&mut hash);
        let hash = hash.finish();

        Self { string, hash }
    }
}
