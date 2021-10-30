#![feature(option_result_unwrap_unchecked)]
mod literal;
#[cfg(test)]
mod tests;
pub use crate::literal::{symbol::Symbol, Constant};
use std::{collections::HashMap, mem};

const STACK_SIZE: usize = 512;
const NIL: Constant = Constant::Nil;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum OpCode {
    Halt,
    Push(usize), // pointer to constant table
    Pop,         // pop's stack (needed for execution)
    Load(usize), // loads a variable
    Save(usize), // saves value to variable
    Drop(usize), // deletes a variable
    Jmf(usize),  // jump if false
    Jmp(usize),

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
}

pub struct Bytecode {
    pub instructions: Vec<OpCode>,
    pub constants: Vec<Constant>,
}

pub struct VirtualMachine {
    bytecode: Bytecode,
    ip: usize, // instruction pointer
    stack: [Constant; STACK_SIZE],
    stack_ptr: usize,
    variables: HashMap<Symbol, Constant>,
}

impl VirtualMachine {
    pub fn reset(&mut self) {
        self.ip = 0;
        self.stack = [NIL; 512];
        self.stack_ptr = 0;
    }

    pub fn pop_last(&self) -> &Constant {
        &self.stack[self.stack_ptr - 1]
    }

    pub fn run(&mut self, bytecode: Bytecode) -> Constant {
        self.ip = 0;
        self.bytecode = bytecode;
        macro_rules! binop {
            ($op:tt) => {{
                let right = self.pop();
                let left = self.pop();

                self.push(self.try_do(left $op right))
            }}
        }

        macro_rules! unaop {
            ($op:tt) => {{
                let right = self.pop();

                self.push(self.try_do($op right))
            }};
        }

        'main: while self.ip < self.bytecode.instructions.len() {
            let inst_ip = self.ip;
            let inst = self.bytecode.instructions[inst_ip];
            self.ip += 1;

            use OpCode::*;
            match inst {
                Halt => break 'main,
                Push(n) => {
                    let val = self.bytecode.constants[n].clone();
                    self.push(val)
                }
                Pop => {
                    self.pop();
                }

                Save(n) => {
                    let val = self.get_val(n);

                    if self.variables.contains_key(&val) {
                        panic!("Can't shadow value")
                    } else {
                        let value = self.pop();
                        self.variables.insert(val, value)
                    };
                }

                Load(n) => {
                    let val = self.get_val(n);

                    self.push(match self.variables.get(&val) {
                        Some(v) => v.clone(),
                        None => panic!("unknown variable {}", val),
                    });
                }

                Drop(n) => {
                    let val = self.get_val(n);

                    self.variables.remove(&val);
                }
                Jmf(offset) => {
                    if Into::<bool>::into(!self.pop()) {
                        self.ip = offset;
                        continue;
                    }
                }
                Jmp(offset) => {
                    self.ip = offset;
                    continue;
                }

                Add => binop!(+),
                Sub => binop!(-),
                Mul => binop!(*),
                Div => binop!(/),
                Xor => binop!(^),
                Shl => binop!(>>),
                Shr => binop!(<<),
                BitAnd => binop!(&),
                BitOr => binop!(|),

                Eq => {
                    let right = self.pop();
                    let left = self.pop();
                    self.push(Constant::Bool(left == right))
                }

                Neg => unaop!(-),
                Not => {
                    let right = self.pop();
                    self.push(!right)
                }
            }

            #[cfg(debug_assertions)]
            eprintln!(
                "STACK: {:?}\nINSTRUCTION: {:?}\nSTACK_PTR: {}\n",
                self.stack
                    .iter()
                    .rev()
                    .skip_while(|it| *it == &NIL)
                    .collect::<Vec<&Constant>>(),
                inst,
                self.stack_ptr
            );
        }

        Constant::Nil
    }

    fn push(&mut self, constant: Constant) {
        self.stack[self.stack_ptr] = constant;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> Constant {
        self.stack_ptr -= 1;
        mem::replace(&mut self.stack[self.stack_ptr], Constant::Nil)
    }

    fn get_val(&self, idx: usize) -> Symbol {
        match &self.bytecode.constants[idx] {
            Constant::Val(v) => v.clone(),
            _ => unreachable!(),
        }
    }

    fn try_do(&self, res: Result<Constant, impl std::fmt::Display>) -> Constant {
        match res {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self {
            bytecode: Bytecode {
                instructions: vec![],
                constants: vec![],
            },
            ip: 0,
            stack: [NIL; STACK_SIZE],
            stack_ptr: 0,
            variables: HashMap::new(),
        }
    }
}
