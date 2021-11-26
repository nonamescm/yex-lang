#![deny(missing_docs)]
//! Virtual Machine implementation for the yex programming language
#[cfg(test)]
mod tests;

mod env;
mod literal;
mod opcode;

use crate::env::Env;
pub use crate::{
    literal::{symbol::Symbol, Constant},
    opcode::{OpCode, OpCodeMetadata},
};
use std::mem;

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

struct CallFrame {
    pub ip: usize,
    pub bytecode: Bytecode,
}

impl CallFrame {
    pub(crate) fn new(bytecode: Bytecode) -> Self {
        Self { ip: 0, bytecode }
    }
}

type CallStack = Vec<CallFrame>;
type Stack = [Constant; STACK_SIZE];

/// Bytecode for the virtual machine, contains the instructions to be executed and the constants to
/// be loaded
pub type Bytecode = Vec<OpCodeMetadata>;

/// Implements the Yex virtual machine, which runs the [`crate::OpCode`] instructions in a stack
/// model
pub struct VirtualMachine {
    constants: Vec<Constant>,
    call_stack: CallStack,
    stack: Stack,
    stack_ptr: usize,
    variables: Env,
}

impl VirtualMachine {
    /// Reset the instruction pointer and the stack
    pub fn reset(&mut self) {
        self.call_stack = vec![];
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
        self.call_stack.push(CallFrame::new(bytecode));

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

        'main: while *self.ip() < self.bytecode().len() {
            self.debug_stack();

            let inst_ip = *self.ip();
            let inst = self.bytecode()[inst_ip];
            *self.ip() += 1;

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
                        *self.ip() = offset;
                        continue;
                    }
                }
                Jmp(offset) => {
                    *self.ip() = offset;
                    continue;
                }

                Nsc => self.variables.nsc(),

                Esc => self.variables.esc(),

                Call => {
                    let arg = self.pop();
                    let fun = match self.pop() {
                        Constant::Fun(fun) => fun,
                        other => panic!("Can't call {}", other),
                    };
                    self.push(arg);

                    self.variables.nsc();
                    self.run(fun);
                    self.variables.esc();
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

        self.call_stack.pop();

        Constant::Nil
    }

    #[cfg(debug_assertions)]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self) {
        let default = CallFrame {
            ip: 0,
            bytecode: vec![],
        };

        let stack = self.call_stack.last().unwrap_or(&default);

        eprintln!(
            "stack: {:#?}\nnext instruction: {:?}\nstack pointer: {}\n",
            self.stack
                .iter()
                .rev()
                .skip_while(|it| *it == &NIL)
                .collect::<Vec<&Constant>>(),
            stack.bytecode.get(stack.ip).map(|it| it.opcode),
            self.stack_ptr
        );
    }

    #[cfg(not(debug_assertions))]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self) {}

    fn push(&mut self, constant: Constant) {
        self.stack[self.stack_ptr] = constant;
        self.stack_ptr += 1;
    }

    fn ip(&mut self) -> &mut usize {
        &mut self.call_stack.last_mut().unwrap().ip
    }

    fn bytecode(&mut self) -> &Bytecode {
        &self.call_stack.last().unwrap().bytecode
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
        let mut prelude = Env::new();

        macro_rules! vecop {
            ($($elems: expr),*) => {{
                let mut vec = Vec::new();
                $({
                    vec.push($elems)
                })*;
                vec.into_iter().map(|opcode| OpCodeMetadata {
                    line: 0,
                    column: 0,
                    opcode,
                }
                ).collect::<Vec<_>>()
            }}
        }

        prelude.insert(
            Symbol::new("puts"),
            Constant::Fun(vecop![OpCode::Cnll(|c| {
                println!("{}", c);
                Constant::Nil
            })]),
        );

        prelude.insert(
            Symbol::new("print"),
            Constant::Fun(vecop![OpCode::Cnll(|c| {
                print!("{}", c);
                Constant::Nil
            })]),
        );

        Self {
            constants: vec![],
            call_stack: vec![],
            stack: [NIL; STACK_SIZE],
            stack_ptr: 0,
            variables: prelude,
        }
    }
}
