use std::{
    fmt::Formatter,
    hash::{Hash, Hasher},
};

/// Symbol struct, contains the symbol string and a pre-hashed value for faster comparison
#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    string: &'static str,
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

struct FnvHasher(u64);
impl Default for FnvHasher {
    fn default() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 ^= *byte as u64;
            self.0 = self.0.wrapping_mul(16777619);
        }
    }
}

impl Symbol {
    /// Creates a new symbol
    pub fn new<T: Into<String>>(str: T) -> Self {
        let string = str.into();

        let mut hash = FnvHasher::default();
        string.hash(&mut hash);
        let hash = hash.finish();

        Self { string: Box::leak(string.into_boxed_str()), hash }
    }
}
