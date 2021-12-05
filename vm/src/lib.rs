#![deny(missing_docs)]
//! Virtual Machine implementation for the yex programming language
#[cfg(test)]
mod tests;

mod env;
mod literal;
mod opcode;
mod stack;
mod list;

use std::cmp::Ordering;

use crate::{env::Env, stack::StackVec};
pub use crate::{
    literal::{symbol::Symbol, Constant},
    opcode::{OpCode, OpCodeMetadata},
    list::List,
};

const STACK_SIZE: usize = 512;
const RECURSION_LIMIT: usize = 768;

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

type CallStack = StackVec<CallFrame, RECURSION_LIMIT>;
type Stack = StackVec<Constant, STACK_SIZE>;

/// Bytecode for the virtual machine, contains the instructions to be executed and the constants to
/// be loaded
pub type Bytecode = Vec<OpCodeMetadata>;

/// Implements the Yex virtual machine, which runs the [`crate::OpCode`] instructions in a stack
/// model
pub struct VirtualMachine {
    constants: Vec<&'static Constant>,
    call_stack: CallStack,
    stack: Stack,
    variables: Env,
}

impl VirtualMachine {
    /// Reset the instruction pointer and the stack
    pub fn reset(&mut self) {
        self.call_stack = StackVec::new();
        self.stack = StackVec::new();
    }

    /// sets the constants for execution
    pub fn set_consts(&mut self, constants: Vec<Constant>) {
        self.constants = vec![];
        for cnst in constants.into_iter() {
            self.constants.push(Box::leak(Box::new(cnst)))
        }
    }

    /// Pop's the last value on the stack
    pub fn pop_last(&self) -> &Constant {
        self.stack.last().unwrap_or(&Constant::Nil)
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

                Save(val) => {
                    let value = self.pop();
                    self.variables.insert(val, value);
                }

                Load(val) => {
                    let val = match self.variables.get(&val) {
                        Some(v) => v.clone(),
                        None => panic!("unknown variable {}", val),
                    };

                    self.push(val);
                }

                Drop(val) => {
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

                Call(carity) => self.call(carity),
                TCall(carity) => self.tcall(carity),

                Cnll(fun) => {
                    let ret = fun(self.pop());
                    self.push(ret)
                }

                Prep => {
                    let val = self.pop();
                    let xs = match self.pop() {
                        Constant::List(xs) => xs,
                        other => panic!("Expected a list, found a `{}`", other)
                    };
                    self.push(Constant::List(xs.prepend(val)))
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

    fn call(&mut self, carity: usize) {
        let mut f_args = vec![];

        let (farity, body) = match self.pop() {
            Constant::Fun { arity, body } => (arity, body),
            Constant::PartialFun { arity, body, args } => {
                f_args = args;
                (arity, body)
            }
            other => panic!("Can't call {}", other),
        };

        while f_args.len() < carity {
            f_args.push(self.pop())
        }

        match carity.cmp(&farity) {
            Ordering::Greater => panic!(
                "function expected {} arguments, but received {}",
                carity, farity
            ),
            Ordering::Less => self.push(Constant::PartialFun {
                arity: farity - carity,
                body,
                args: f_args,
            }),
            Ordering::Equal => {
                f_args.into_iter().for_each(|it| self.push(it));

                self.variables.nsc();
                self.run(body);
                self.variables.esc();
            }
        }
    }

    fn tcall(&mut self, carity: usize) {
        let mut f_args = vec![];

        let (farity, body) = match self.pop() {
            Constant::Fun { arity, body } => (arity, body),
            Constant::PartialFun { arity, body, args } => {
                f_args = args;
                (arity, body)
            }
            other => panic!("Can't call {}", other),
        };

        while f_args.len() < carity {
            f_args.push(self.pop())
        }

        match carity.cmp(&farity) {
            Ordering::Greater => panic!(
                "function expected {} arguments, but received {}",
                carity, farity
            ),

            Ordering::Less => {
                panic!("Can't use partial application in a tail call")
            }
            Ordering::Equal => {
                f_args.into_iter().for_each(|it| self.push(it));

                if &body == self.bytecode() {
                    *self.ip() = 0;
                } else {
                    // useful for doing some optimizations with high-order-functions
                    self.variables.nsc();
                    self.run(body);
                    self.variables.esc();
                }
            }
        }
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
            "stack: {:#?}\nnext instruction: {:?}\n",
            self.stack.iter().rev().collect::<Vec<&Constant>>(),
            stack.bytecode.get(stack.ip).map(|it| it.opcode),
        );
    }

    #[cfg(not(debug_assertions))]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self) {}

    #[track_caller]
    fn push(&mut self, constant: Constant) {
        self.stack.push(constant)
    }

    fn ip(&mut self) -> &mut usize {
        let idx = self.call_stack.len();
        &mut self.call_stack[idx - 1].ip
    }

    fn bytecode(&mut self) -> &Bytecode {
        &self.call_stack.last().unwrap().bytecode
    }

    #[track_caller]
    fn pop(&mut self) -> Constant {
        self.stack.pop()
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
            Constant::Fun {
                arity: 1,
                body: vecop![OpCode::Cnll(|c| {
                    match c {
                        Constant::Str(s) => println!("{}", s),
                        other => println!("{}", other)
                    };
                    Constant::Nil
                })],
            },
        );

        prelude.insert(
            Symbol::new("print"),
            Constant::Fun {
                arity: 1,
                body: vecop![OpCode::Cnll(|c| {
                    match c {
                        Constant::Str(s) => print!("{}", s),
                        other => print!("{}", other)
                    };
                    Constant::Nil
                })],
            },
        );

        Self {
            constants: vec![],
            call_stack: StackVec::new(),
            stack: StackVec::new(),
            variables: prelude,
        }
    }
}
