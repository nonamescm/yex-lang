use std::{collections::HashMap, mem};
use crate::{Symbol, Constant};

#[derive(Debug)]
pub(crate) struct Env {
    current: HashMap<Symbol, Constant>,
    over: Option<Box<Env>>,
}

impl Env {
    pub fn nsc(&mut self) {
        let old = mem::replace(self, Env::new(None));
        *self = Env::new(Some(Box::new(old)));
    }

    pub fn esc(&mut self) {
        let over = mem::replace(&mut self.over, None);
        *self = *over.unwrap();
    }

    pub fn new(over: Option<Box<Env>>) -> Self {
        Self {
            current: HashMap::new(),
            over,
        }
    }

    pub fn insert(&mut self, key: Symbol, value: Constant) -> Option<()> {
        self.current.insert(key, value);
        Some(())
    }

    pub fn get(&mut self, key: &Symbol) -> Option<Constant> {
        if let Some(v) = self.current.get(key) {
            Some(v.clone())
        } else {
            match &mut self.over {
                Some(sup) => sup.get(key),
                None => None,
            }
        }
    }

    pub fn remove(&mut self, key: &Symbol) {
        self.current.remove(key);
    }
}

