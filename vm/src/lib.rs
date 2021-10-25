#![feature(option_result_unwrap_unchecked)]
#[cfg(test)]
mod tests;

mod literal;
pub use crate::literal::{symbol::Symbol, Constant};

const STACK_SIZE: usize = 512;
const NIL: Constant = Constant::Nil;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum OpCode {
    Halt = 0x00,
    Push = 0x01,
    Load = 0x02, // loads a variable
    Save = 0x03, // saves value to variable
    Ret = 0x04,

    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Neg = 0x14,
    Not = 0x15,
    Xor = 0x16,
    Shr = 0x17,
    Shl = 0x18,
    BitAnd = 0x19,
    BitOr = 0x20,
    Eq = 0x21,
}

pub struct Bytecode {
    pub instructions: Vec<OpCode>,
    pub constants: Vec<Constant>,
}

pub struct VirtualMachine {
    bytecode: Bytecode,
    ip: usize, // instruction pointer
    cp: usize, // constant pointer
    stack: [Constant; STACK_SIZE],
    stack_ptr: usize,
}

impl VirtualMachine {
    pub fn reset_ip(&mut self) {
        self.ip = 0;
    }

    pub fn reset_cp(&mut self) {
        self.cp = 0;
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
            #[cfg(debug_assertions)]
            eprintln!("{:?}", self.stack);

            let inst = self.ip;
            self.ip += 1;

            use OpCode::*;
            match self.bytecode.instructions[inst] {
                Halt => break 'main,
                Push => {
                    self.cp += 1;
                    let val = self.bytecode.constants[self.cp - 1].clone();
                    self.push(val)
                }
                Save => todo!(),
                Load => todo!(),
                Ret => return (self.pop()).clone(),

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
                Not => unaop!(!),
            }
        }

        Constant::Nil
    }

    fn push(&mut self, constant: Constant) {
        self.stack[self.stack_ptr] = constant;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> Constant {
        let val = std::mem::replace(&mut self.stack[self.stack_ptr - 1], Constant::Nil);
        self.stack_ptr -= 1;
        val
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
            cp: 0,
            stack: [NIL; 512],
            stack_ptr: 0,
        }
    }
}
