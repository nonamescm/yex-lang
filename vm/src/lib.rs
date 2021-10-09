mod literal;
#[cfg(test)]
mod tests;
use crate::symbol::Symbol;
pub use literal::{symbol, Literal};
use std::{collections::HashMap, rc::Rc};

#[derive(PartialEq, Debug)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Not,
    Xor,
    Shr,
    Shl,
    BitAnd,
    BitOr,
    Eq,
    Push(Literal),
    Load(Symbol), // loads a variable
    Save(Symbol), // saves value to variable
    Ret,
    Halt,
}

macro_rules! panic_vm {
    ($($tt:tt)*) => {{
        eprintln!($($tt)*);
        std::panic::set_hook(Box::new(|_| {}));
        panic!()
    }}
}

pub struct VirtualMachine {
    stack: Vec<Rc<Literal>>,
    line: usize,
    column: usize,
    vars: HashMap<Symbol, Rc<Literal>>,
}

impl VirtualMachine {
    pub fn run(&mut self, code: Vec<Instruction>) -> Literal {
        for intr in code {
            if let Some(v) = self.run_instruction(intr) {
                return (*v).clone();
            }
        }
        Literal::Nil
    }

    fn pop(&mut self) -> Rc<Literal> {
        self.stack.pop().unwrap_or(Rc::new(Literal::Nil))
    }

    fn push(&mut self, literal: Rc<Literal>) {
        self.stack.push(literal)
    }

    fn try_do<E: std::fmt::Display>(&self, result: Result<Literal, E>) -> Literal {
        match result {
            Ok(l) => l,
            Err(e) => panic_vm!("[{}:{}] {}", self.line, self.column, e),
        }
    }

    fn run_instruction(&mut self, intr: Instruction) -> Option<Rc<Literal>> {
        match intr {
            Instruction::Add => {
                let right = self.pop();
                let left = self.pop();

                self.push(Rc::new(self.try_do(&*left + &*right)));
            }
            Instruction::Sub => {
                let right = self.pop();
                let left = self.pop();

                self.push(Rc::new(self.try_do(&*left - &*right)));
            }
            Instruction::Mul => {
                let right = self.pop();
                let left = self.pop();

                self.push(Rc::new(self.try_do(&*left * &*right)));
            }
            Instruction::Div => {
                let right = self.pop();
                let left = self.pop();

                self.push(Rc::new(self.try_do(&*left / &*right)));
            }
            Instruction::Neg => {
                let left = self.pop();
                self.push(Rc::new(self.try_do(-&*left)));
            }
            Instruction::Push(lit) => {
                self.push(Rc::new(lit));
            }

            Instruction::Xor => {
                let right = self.pop();
                let left = self.pop();
                self.push(Rc::new(self.try_do(&*left ^ &*right)))
            }
            Instruction::BitAnd => {
                let right = self.pop();
                let left = self.pop();
                self.push(Rc::new(self.try_do(&*left & &*right)))
            }
            Instruction::BitOr => {
                let right = self.pop();
                let left = self.pop();
                self.push(Rc::new(self.try_do(&*left | &*right)))
            }
            Instruction::Shr => {
                let right = self.pop();
                let left = self.pop();
                self.push(Rc::new(self.try_do(&*left >> &*right)))
            }
            Instruction::Shl => {
                let right = self.pop();
                let left = self.pop();
                self.push(Rc::new(self.try_do(&*left << &*right)))
            }
            Instruction::Eq => {
                let right = self.pop();
                let left = self.pop();

                self.push(Rc::new(Literal::Bool(left == right)))
            }

            Instruction::Not => {
                let left = self.pop();
                self.push(Rc::new(self.try_do(!&*left)))
            }
            Instruction::Load(s) => {
                let val = self
                    .vars
                    .get(&s)
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Literal::Nil));
                self.push(val)
            }
            Instruction::Save(s) => {
                let val = self.pop();
                self.vars.insert(s, val);
            }

            Instruction::Ret => return Some(self.pop()),
            Instruction::Halt => {
                self.stack = vec![];
            }
        };
        None
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self {
            stack: vec![],
            line: 1,
            column: 1,
            vars: HashMap::new(),
        }
    }
}
