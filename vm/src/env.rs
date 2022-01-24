use core::slice;
use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::null_mut,
};

use crate::{
    literal::{nil, Constant},
    Symbol,
};

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
    capacity: usize,
    count: usize,
    entries: *mut Entry,
}

impl EnvTable {
    const BASE_VALUE: usize = 4;

    /// Creates a new table
    pub fn new() -> Self {
        Self::with_capacity(Self::BASE_VALUE)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let entries = unsafe {
            let entries = alloc(Layout::array::<Entry>(capacity).unwrap()) as *mut Entry;
            for index in 0..capacity {
                entries.add(index).write(Entry {
                    key: None,
                    value: nil(),
                })
            }
            entries
        };
        Self {
            capacity,
            count: 0,
            entries,
        }
    }

    unsafe fn find_entry(entries: *mut Entry, capacity: usize, key: &Symbol) -> (*mut Entry, bool) {
        let mut index = key.hash & (capacity - 1);
        let mut last_null: *mut Entry = null_mut();

        loop {
            let entry = entries.add(index);
            index = (index + 1) & (capacity - 1);
            match (*entry).key {
                Some(k) if k == *key => return (&mut *entry, true),
                None if (*entry).value.is_nil() => {
                    return if !last_null.is_null() {
                        (last_null, false)
                    } else {
                        (entry, false)
                    }
                }
                None if last_null.is_null() => {
                    last_null = entry;
                }
                _ => continue,
            }
        }
    }

    /// Inserts an item in the table
    pub fn insert(&mut self, key: Symbol, value: Constant) {
        if self.count + (self.capacity / Self::BASE_VALUE) >= self.capacity {
            let len = self.capacity * 2;
            self.realloc(len)
        }

        let (entry, init) = unsafe { Self::find_entry(self.entries, self.capacity, &key) };

        unsafe {
            if !init {
                self.count += 1;
            }
            (*entry).key = Some(key);
            (*entry).value = value;
        }
    }

    fn realloc(&mut self, len: usize) {
        let entries = unsafe { alloc(Layout::array::<Entry>(len).unwrap()) as *mut Entry };

        for index in 0..len {
            unsafe {
                entries.add(index).write(Entry {
                    key: None,
                    value: nil(),
                })
            }
        }

        for index in 0..self.capacity {
            unsafe {
                let entry = self.entries.add(index);

                match (*entry).key {
                    Some(k) => {
                        let new = Self::find_entry(entries, len, &k).0;
                        new.swap(entry);
                    }
                    None => continue,
                }
            }
        }

        unsafe {
            dealloc(
                self.entries as *mut u8,
                Layout::array::<Entry>(self.capacity).unwrap(),
            );
        }

        self.entries = entries;
        self.capacity = len;
    }

    /// Indexes an item in the table
    pub fn get(&self, key: &Symbol) -> Option<Constant> {
        unsafe {
            let (entry, init) = Self::find_entry(self.entries, self.capacity, key);
            if init {
                Some((*entry).value.clone())
            } else {
                None
            }
        }
    }

    /// Returns the table length
    pub fn len(&self) -> usize {
        self.count
    }

    /// Checks if the table is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Iterates over the table
    fn iter(&self) -> impl Iterator<Item = (Key, Value)> + '_ {
        unsafe {
            slice::from_raw_parts(self.entries, self.capacity)
                .iter()
                .filter(|it| it.key.is_some())
                .map(|it| (it.key.unwrap(), it.value.clone()))
        }
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

impl Drop for EnvTable {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.entries as *mut u8,
                Layout::array::<Entry>(self.capacity).unwrap(),
            );
        }
    }
}
