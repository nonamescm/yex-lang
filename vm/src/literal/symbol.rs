use std::{
    collections::hash_map::DefaultHasher,
    fmt::Formatter,
    hash::{Hash, Hasher},
};

#[derive(Debug)]
pub struct Symbol {
    string: String,
    hash: u64,
}

impl std::cmp::PartialEq for Symbol {
    fn eq(&self, rhs: &Self) -> bool {
        rhs.hash == rhs.hash
    }
}

impl std::cmp::PartialEq<bool> for Symbol {
    fn eq(&self, rhs: &bool) -> bool {
        (self == &Self::sym_true()) == *rhs
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sym = ":".to_string() + &self.string;
        write!(f, "{}", sym)?;
        Ok(())
    }
}

impl Symbol {
    pub fn new<T: Into<String>>(str: T) -> Self {
        let string = str.into();

        let mut hash = DefaultHasher::new();
        string.hash(&mut hash);
        let hash = hash.finish();

        let a = Self { string, hash };
        println!("{:?}", a);
        a
    }

    pub fn sym_true() -> Self {
        Self::new("true")
    }

    pub fn sym_false() -> Self {
        Self::new("false")
    }
}
