#![feature(option_result_unwrap_unchecked)]
#![deny(missing_docs)]
//! Virtual Machine implementation for the yex programming language

mod literal;
#[cfg(test)]
mod tests;
pub use crate::literal::{symbol::Symbol, Constant};
use std::{collections::HashMap, hint::unreachable_unchecked, mem};

const STACK_SIZE: usize = 512;
const NIL: Constant = Constant::Nil;

static mut LINE: usize = 1;
static mut COLUMN: usize = 1;

macro_rules! panic {
    ($($tt:tt)+) => {
        unsafe {
            let msg = format!($($tt)+);
            std::eprintln!("[{}:{}] {}", LINE, COLUMN, msg);
            std::panic::set_hook(Box::new(|_| {}));
            std::panic!()
        }
    }
}

/// OpCodes for the virtualMachine
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum OpCode {
    /// Stops the virtual machine
    Halt,

    /// Push a value by it's index on the constant table on-to the stack
    Push(usize), // pointer to constant table

    /// Pop a value from the stack
    Pop,

    /// Read a value from a variable, receives the index of the variable name in the constant table as
    /// argument
    Load(usize),

    /// Save a value to a variable, receives the index of the variable name in the constant table as
    /// argument
    Save(usize),

    /// Drops a variable, receives the index of the variable name in the constant table as argument
    Drop(usize),

    /// Jump if the value on the stack top is false
    Jmf(usize),

    /// Unconditional jump
    Jmp(usize),

    /// Add the two values on the stack top
    Add,

    /// Subtract the two values on the stack top
    Sub,

    /// Multiplicate the two values on the stack top
    Mul,

    /// Divide the two values on the stack top
    Div,

    /// Negates the value on the stack top
    Neg,

    /// Apply a unary not to the stack top
    Not,

    /// Apply a xor operation on the two values on the stack top
    Xor,

    /// Apply shift-right operation on the two values on the stack top
    Shr,

    /// Apply shift-left operation on the two values on the stack top
    Shl,

    /// Apply bit-and operation on the two values on the stack top
    BitAnd,

    /// Apply bit-or operation on the two values on the stack top
    BitOr,

    /// Check if the two values on the stack tops are equal
    Eq,
}

/// Stocks the [`crate::OpCode`] with the line and the column of it on the original source code,
/// make it possible to be used for error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpCodeMetadata {
    /// Source's code line
    pub line: usize,
    /// Source's code column
    pub column: usize,

    /// Actual opcode
    pub opcode: OpCode,
}

/// Bytecode for the virtual machine, contains the instructions to be executed and the constants to
/// be loaded
#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode {
    /// the instructions, made of [`crate::OpCodeMetadata`]
    pub instructions: Vec<OpCodeMetadata>,
}

/// Implements the Yex virtual machine, which runs the [`crate::OpCode`] instructions in a stack
/// model
pub struct VirtualMachine {
    constants: Vec<Constant>,
    bytecode: Bytecode,
    ip: usize, // instruction pointer
    stack: [Constant; STACK_SIZE],
    stack_ptr: usize,
    variables: HashMap<Symbol, Constant>,
}

impl VirtualMachine {
    /// Reset the instruction pointer and the stack
    pub fn reset(&mut self) {
        self.ip = 0;
        self.stack = [NIL; 512];
        self.stack_ptr = 0;
        self.constants = vec![];
    }

    /// sets the constants for execution
    pub fn set_consts(&mut self, constants: Vec<Constant>) {
        self.constants = constants;
    }

    /// Pop's the last value on the stack
    pub fn pop_last(&self) -> &Constant {
        &self.stack[self.stack_ptr - 1]
    }

    /// Executes a given set of bytecode instructions
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

            unsafe {
                LINE = inst.line;
                COLUMN = inst.column;
            }

            use OpCode::*;
            match inst.opcode {
                Halt => break 'main,
                Push(n) => {
                    let val = self.constants[n].clone();
                    self.push(val)
                }
                Pop => {
                    self.pop();
                }

                Save(n) => {
                    let val = self.get_val(n);

                    if self.variables.contains_key(&val) {
                        panic!("Can't shadow value");
                    } else {
                        let value = self.pop();
                        self.variables.insert(val, value);
                    }
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
                inst.opcode,
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
        match &self.constants[idx] {
            Constant::Val(v) => v.clone(),
            _ => unsafe { unreachable_unchecked() },
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
            constants: vec![],
            bytecode: Bytecode {
                instructions: vec![],
            },
            ip: 0,
            stack: [NIL; STACK_SIZE],
            stack_ptr: 0,
            variables: HashMap::new(),
        }
    }
}
