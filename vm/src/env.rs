use crate::{Constant, Symbol};

#[derive(Debug)]
struct Entry {
    pub key: Symbol,
    pub value: Constant,
}

#[derive(Debug)]
struct Table {
    entries: Vec<Entry>,
}

impl Table {
    pub fn new() -> Self {
        Self { entries: vec![] }
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

    pub fn insert(&mut self, key: Symbol, value: Constant) {
        match self.find_entry(&key) {
            Some(entry) => *entry = Entry { key, value },
            None => self.entries.push(Entry { key, value }),
        }
    }

    pub fn get(&mut self, key: &Symbol) -> Option<&Constant> {
        match self.find_entry(&key) {
            Some(entry) => Some(&entry.value),
            None => None,
        }
    }

    pub fn remove(&mut self, key: &Symbol) {
        match self.find_entry_idx(key) {
            Some(idx) => {
                self.entries.remove(idx);
            }
            None => {}
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
        Self { entries: vec![Table::new()] }
    }

    pub fn insert(&mut self, key: Symbol, value: Constant) -> Option<()> {
        self.top().insert(key, value);
        Some(())
    }

    pub fn get(&mut self, key: &Symbol) -> Option<&Constant> {
        for entry in self.entries.iter_mut().rev() {
            if let Some(value) = entry.get(key) {
                return Some(value)
            }
        }
        None
    }

    pub fn remove(&mut self, key: &Symbol) {
        self.top().remove(key);
    }
}
