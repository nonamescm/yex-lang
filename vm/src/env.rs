use std::{
    alloc::{alloc, Layout},
    ptr::null_mut,
};

use crate::{literal::Constant, Symbol};
use smallvec::{smallvec, SmallVec};

// const MAX_TABLE_ENTRIES: usize = 256;

type Key = Symbol;
type Value = Constant;

#[derive(Debug, PartialEq, Clone)]
struct Entry {
    pub key: Option<Key>,
    pub value: Value,
}

#[derive(Debug, PartialEq, Clone)]
/// A table of key-value pairs
pub struct EnvTable {
    entries: *mut Entry,
    capacitity: usize,
    count: usize,
}

impl EnvTable {
    /// Creates a new table
    pub fn new() -> Self {
        Self {
            capacitity: 0,
            count: 0,
            entries: null_mut(),
        }
    }

    unsafe fn find_entry(&self, key: &Key) -> *mut Entry {
        let mut index = 0;
        let mut tombstone: *mut Entry = null_mut();
        while index < self.count {
            let entry = self.entries.add(index);
            if (*entry).key.as_ref() == Some(key) {
                return entry;
            } else {
                index += 1;
                tombstone = entry;
            }
        }
        tombstone
    }

    /// Inserts an item in the table
    pub fn insert(&mut self, key: Symbol, value: Constant) {
        if self.count + 1 > self.capacitity {
            let old_capacitity = self.capacitity;
            self.capacitity += (self.capacitity + 1) * 2;
            unsafe { self.resize(old_capacitity) };
        }
        unsafe {
            let entry = self.find_entry(&key);
            *entry = Entry {
                key: Some(key),
                value,
            }
        };
        self.count += 1;
    }

    /// Indexes an item in the table
    pub fn get(&self, key: &Symbol) -> Option<Constant> {
        unsafe {
            self.find_entry(key)
                .as_ref()
                .map(|entry| entry.value.clone())
        }
    }

    unsafe fn resize(&mut self, old_capacitity: usize) {
        let entries = alloc(Layout::array::<Entry>(self.capacitity).unwrap()) as *mut Entry;

        for index in 0..(self.capacitity) {
            let entry = entries.offset(index as isize);
            (*entry) = Entry {
                key: None,
                value: Constant::Nil,
            };
        }

        for index in 0..(old_capacitity as isize) {
            let new_entry = entries.offset(index);
            let old_entry = self.entries.offset(index);

            match (*old_entry).key {
                Some(..) => {
                    new_entry.swap(old_entry)
                }
                None => continue,
            }
        }
    }

    /// Remove an item from the table
    pub fn remove(&mut self, key: &Symbol) {
        unsafe {
            self.find_entry(key).as_mut().map(|entry| {
                *entry = Entry {
                    key: None,
                    value: Constant::Nil,
                }
            })
        };
        self.count -= 1;
    }

    /// Returns the underline table length
    pub fn len(&self) -> usize {
        self.count
    }

    /// Checks if the table is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl std::fmt::Display for EnvTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
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
