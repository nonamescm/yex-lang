use crate::{GcRef, Symbol, Value};

type Key = Symbol;

#[derive(Debug, PartialEq, Clone)]
struct Entry {
    pub key: Key,
    pub value: Value,
}

#[derive(Debug, PartialEq, Clone)]
/// A table of key-value pairs
pub struct Table {
    entries: Vec<GcRef<Entry>>,
}

impl Table {
    /// Creates a new table
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn find_entry_idx(&self, key: &Symbol) -> Option<usize> {
        for (idx, entry) in self.entries.iter().enumerate() {
            if &entry.key == key {
                return Some(idx);
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
    #[must_use]
    pub fn insert(&self, key: Symbol, value: Value) -> Self {
        let mut new = self.clone();

        match new.find_entry_idx(&key) {
            Some(idx) => {
                new.entries[idx] = GcRef::new(Entry { key, value });
                new
            }

            None => {
                new.entries.push(GcRef::new(Entry { key, value }));
                new
            }
        }
    }

    /// Indexes an item in the table
    pub fn get(&self, key: &Symbol) -> Option<Value> {
        self.find_entry(key).map(|entry| entry.value.clone())
    }

    /// Returns the underline table length
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the table is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over the table
    pub fn iter(&self) -> impl Iterator<Item = (Key, Value)> + '_ {
        self.entries.iter().map(|it| (it.key, it.value.clone()))
    }
}

impl std::fmt::Display for Table {
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

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
