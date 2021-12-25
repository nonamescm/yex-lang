use crate::{literal::Constant, Symbol};
use smallvec::{SmallVec, smallvec};

// const MAX_TABLE_ENTRIES: usize = 256;

type Key = Symbol;
type Value = Constant;

#[derive(Debug, PartialEq, Clone)]
struct Entry {
    pub key: Key,
    pub value: Value,
}

#[derive(Debug, PartialEq, Clone)]
/// A table of key-value pairs
pub struct EnvTable {
    entries: Vec<Entry>,
}

impl EnvTable {
    /// Creates a new table
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn find_entry_idx(&mut self, key: &Symbol) -> Option<usize> {
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            if &entry.key == key {
                return Some(idx);
            }
        }
        None
    }

    fn find_entry_mut(&mut self, key: &Symbol) -> Option<&mut Entry> {
        for entry in self.entries.iter_mut() {
            if &entry.key == key {
                return Some(entry);
            }
        }
        None
    }

    fn find_entry(&self, key: &Symbol) -> Option<&Entry> {
        for entry in self.entries.iter() {
            if &entry.key == key {
                return Some(entry);
            }
        }
        None
    }

    /// Inserts an item in the table
    pub fn insert(&mut self, key: Symbol, value: Constant) {
        match self.find_entry_mut(&key) {
            Some(entry) => *entry = Entry { key, value },
            None => self.entries.push(Entry { key, value }),
        }
    }

    /// Indexes an item in the table
    pub fn get(&self, key: &Symbol) -> Option<Constant> {
        self.find_entry(key).map(|entry| entry.value.clone())
    }

    /// Remove an item from the table
    pub fn remove(&mut self, key: &Symbol) {
        if let Some(idx) = self.find_entry_idx(key) {
            self.entries.remove(idx);
        }
    }

    /// Returns the underline table length
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the table is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over the table
    pub fn iter(&self) -> impl Iterator<Item = (Key, Value)> + '_ {
        self.entries.iter().map(|it| (it.key, it.value.clone()))
    }
}

impl std::fmt::Display for EnvTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (len, (key, value)) in self.iter().enumerate() {
            if len != self.len() - 1 {
                write!(f, "{} = {}, ", key, value)?;
            } else {
                write!(f, "{} = {}", key, value)?;
            }
        }
        write!(f, "}}")
    }
}

impl Default for EnvTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) struct Env {
    entries: SmallVec<[EnvTable; 4]>,
}

impl Env {
    pub fn nsc(&mut self) {
        self.entries.push(EnvTable::new());
    }

    pub fn esc(&mut self) {
        self.entries.pop();
    }

    fn top(&mut self) -> &mut EnvTable {
        self.entries.last_mut().unwrap()
    }

    pub fn new() -> Self {
        Self {
            entries: smallvec![EnvTable::new()],
        }
    }

    pub fn insert(&mut self, key: Symbol, value: Constant) -> Option<()> {
        self.top().insert(key, value);
        Some(())
    }

    pub fn get(&mut self, key: &Symbol) -> Option<Constant> {
        for entry in self.entries.iter_mut().rev() {
            if let Some(value) = entry.get(key) {
                return Some(value);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &Symbol) {
        self.top().remove(key);
    }
}
