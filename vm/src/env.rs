use crate::{literal::ConstantRef, StackVec, Symbol};

const MAX_TABLE_ENTRIES: usize = 256;

#[derive(Debug)]
struct Entry {
    pub key: Symbol,
    pub value: ConstantRef,
}

#[derive(Debug)]
pub struct Table {
    entries: StackVec<Entry, MAX_TABLE_ENTRIES>,
}

impl Table {
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

    fn find_entry(&mut self, key: &Symbol) -> Option<&mut Entry> {
        for entry in self.entries.iter_mut() {
            if &entry.key == key {
                return Some(entry);
            }
        }
        None
    }

    pub fn insert(&mut self, key: Symbol, value: ConstantRef) {
        match self.find_entry(&key) {
            Some(entry) => *entry = Entry { key, value },
            None => self.entries.push(Entry { key, value }),
        }
    }

    pub fn get(&mut self, key: &Symbol) -> Option<ConstantRef> {
        match self.find_entry(key) {
            Some(entry) => Some(entry.value.clone()),
            None => None,
        }
    }

    pub fn remove(&mut self, key: &Symbol) {
        if let Some(idx) = self.find_entry_idx(key) {
            self.entries.remove(idx);
        }
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
