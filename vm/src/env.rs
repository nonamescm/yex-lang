use crate::{literal::ConstantRef, StackVec, Symbol, GcRef};

const MAX_TABLE_ENTRIES: usize = 256;

type Key = Symbol;
type Value = ConstantRef;

#[derive(Debug, PartialEq)]
struct Entry {
    pub key: Key,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
/// A table of key-value pairs
pub struct Table {
    entries: StackVec<Entry, MAX_TABLE_ENTRIES>,
}

impl Table {
    /// Creates a new table
    pub fn new() -> Self {
        Self {
            entries: StackVec::new(),
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
    pub fn insert(&mut self, key: Symbol, value: ConstantRef) {
        match self.find_entry_mut(&key) {
            Some(entry) => *entry = Entry { key, value },
            None => self.entries.push(Entry { key, value }),
        }
    }

    /// Indexes an item in the table
    pub fn get(&self, key: &Symbol) -> Option<ConstantRef> {
        match self.find_entry(key) {
            Some(entry) => Some(entry.value.clone()),
            None => None,
        }
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

    /// Iterates over the table
    pub fn iter(&self) -> impl Iterator<Item = (Key, Value)> + '_ {
        self.entries.iter().map(|it| (it.key, GcRef::clone(&it.value)))
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (len, (key, value)) in self.iter().enumerate() {
            if len != self.len() - 1 {
                write!(f, "{} = {}, ", key, value.get())?;
            } else {
                write!(f, "{} = {}", key, value.get())?;
            }
        }
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub(crate) struct Env {
    entries: Vec<Table>,
}

impl Env {
    pub fn nsc(&mut self) {
        self.entries.push(Table::new());
    }

    pub fn esc(&mut self) {
        self.entries.pop();
    }

    fn top(&mut self) -> &mut Table {
        self.entries.last_mut().unwrap()
    }

    pub fn new() -> Self {
        Self {
            entries: vec![Table::new()],
        }
    }

    pub fn insert(&mut self, key: Symbol, value: ConstantRef) -> Option<()> {
        self.top().insert(key, value);
        Some(())
    }

    pub fn get(&mut self, key: &Symbol) -> Option<ConstantRef> {
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
