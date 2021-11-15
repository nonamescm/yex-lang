#![feature(option_result_unwrap_unchecked)]
#![deny(missing_docs)]
//! Virtual Machine implementation for the yex programming language

mod literal;
#[cfg(test)]
mod tests;
pub use crate::literal::{symbol::Symbol, Constant};
use std::{collections::HashMap, mem};

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

struct Env {
    current: HashMap<Symbol, Constant>,
    pub over: Option<Box<Env>>,
}

impl Env {
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

    /// Creates a new scope
    Nsc,

    /// Ends a scope
    Esc,

    /// Calls the value on the top of the stack
    Call,

    /// Calls a native rust function
    Cnll(fn(Constant) -> Constant),

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
pub type Bytecode = Vec<OpCodeMetadata>;

/// Implements the Yex virtual machine, which runs the [`crate::OpCode`] instructions in a stack
/// model
pub struct VirtualMachine {
    constants: Vec<Constant>,
    bytecode: Bytecode,
    ip: usize, // instruction pointer
    stack: [Constant; STACK_SIZE],
    stack_ptr: usize,
    variables: Env,
}

impl VirtualMachine {
    /// Reset the instruction pointer and the stack
    pub fn reset(&mut self) {
        self.ip = 0;
        self.stack = [NIL; 512];
        self.stack_ptr = 0;
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

        'main: while self.ip < self.bytecode.len() {
            self.debug_stack();

            let inst_ip = self.ip;
            let inst = self.bytecode[inst_ip];
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

                    let value = self.pop();
                    self.variables.insert(val, value);
                }

                Load(n) => {
                    let val = self.get_val(n);

                    let val = match self.variables.get(&val) {
                        Some(v) => v.clone(),
                        None => panic!("unknown variable {}", val),
                    };

                    self.push(val);
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

                Nsc => {
                    let old = mem::replace(&mut self.variables, Env::new(None));

                    self.variables = Env::new(Some(Box::new(old)));
                }

                Esc => {
                    let over = mem::replace(&mut self.variables.over, None);
                    self.variables = *over.unwrap();
                }

                Call => {
                    let arg = self.pop();
                    let fun = self.pop();
                    self.push(arg);

                    self.run(match fun {
                        Constant::Fun(body) => body,
                        _ => todo!("Better error message"),
                    });
                }

                Cnll(fun) => {
                    let ret = fun(self.pop());
                    self.push(ret)
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
        }

        Constant::Nil
    }

    fn debug_stack(&self) {
        #[cfg(debug_assertions)]
        eprintln!(
            "STACK: {:?}\nINSTRUCTION: {:?}\nSTACK_PTR: {}\n",
            self.stack
                .iter()
                .rev()
                .skip_while(|it| *it == &NIL)
                .collect::<Vec<&Constant>>(),
            self.bytecode[self.ip].opcode,
            self.stack_ptr
        );
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
            _ => panic!("Tried to access a value that is not variable"),
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
        let mut prelude = Env::new(None);
        prelude.insert(
            Symbol::new("puts"),
            Constant::Fun(vec![OpCodeMetadata {
                line: 0,
                column: 0,
                opcode: OpCode::Cnll(|c| {
                    println!("{}", c);
                    Constant::Nil
                }),
            }]),
        );

        prelude.insert(
            Symbol::new("print"),
            Constant::Fun(vec![OpCodeMetadata {
                line: 0,
                column: 0,
                opcode: OpCode::Cnll(|c| {
                    print!("{}", c);
                    Constant::Nil
                }),
            }]),
        );
        Self {
            constants: vec![],
            bytecode: vec![],
            ip: 0,
            stack: [NIL; STACK_SIZE],
            stack_ptr: 0,
            variables: prelude,
        }
    }
}
