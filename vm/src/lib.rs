#![deny(missing_docs)]
//! Virtual Machine implementation for the yex programming language
mod env;
mod list;
mod literal;
mod opcode;
mod prelude;
mod stack;
#[cfg(test)]
mod tests;

use either::Either;
use std::{cmp::Ordering, mem};

use crate::{
    env::{Env, Table},
    literal::NativeFun,
    stack::StackVec,
};

pub use crate::{
    list::List,
    literal::{symbol::Symbol, Constant},
    opcode::{OpCode, OpCodeMetadata},
};

const STACK_SIZE: usize = 512;
const RECURSION_LIMIT: usize = 768;

static mut LINE: usize = 1;
static mut COLUMN: usize = 1;

#[macro_export]
#[doc(hidden)]
macro_rules! panic {
    ($($tt:tt)+) => {
        unsafe {
            let msg = format!($($tt)+);
            std::eprintln!("[{}:{}] {}", $crate::LINE, $crate::COLUMN, msg);
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
    constants: Vec<Constant>,
    call_stack: CallStack,
    stack: Stack,
    variables: Env,
    globals: Table,
}

impl VirtualMachine {
    /// Reset the instruction pointer and the stack
    pub fn reset(&mut self) {
        self.call_stack = StackVec::new();
        self.stack = StackVec::new();
    }

    /// sets the constants for execution
    pub fn set_consts(&mut self, constants: Vec<Constant>) {
        self.constants = constants;
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

                Savg(val) => {
                    let value = self.pop();
                    self.globals.insert(val, value)
                }

                Load(val) => {
                    let val = match self.variables.get(&val) {
                        Some(v) => v.clone(),
                        None => match self.globals.get(&val) {
                            Some(v) => v.clone(),
                            None => panic!("unknown variable {}", val),
                        },
                    };

                    self.push(val);
                }

                Drop(val) => {
                    self.variables.remove(&val);
                }

                Drpg(val) => self.globals.remove(&val),

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

                Prep => {
                    let val = self.pop();
                    let xs = match self.pop() {
                        Constant::List(xs) => xs,
                        other => panic!("Expected a list, found a `{}`", other),
                    };
                    self.push(Constant::List(xs.prepend(val)))
                }

                Index => {
                    let index = match self.pop() {
                        Constant::Num(n) if n.fract() == 0.0 && n >= 0.0 => n as usize,
                        other => panic!("Expected a integer to use as index, found a `{}`", other),
                    };
                    let xs = match self.pop() {
                        Constant::List(xs) => xs,
                        other => panic!("Expected a list to index, found a `{}`", other),
                    };

                    self.push(xs.index(index))
                }

                Rev => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a);
                    self.push(b);
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
                Len => {
                    let len = self.pop().len();
                    self.push(Constant::Num(len as f64))
                }
                Not => {
                    let right = self.pop();
                    self.push(!right)
                }
            }
        }

        self.call_stack.pop();

        Constant::Nil
    }

    fn call_helper(
        &mut self,
        carity: usize,
    ) -> (Either<Bytecode, NativeFun>, usize, Vec<Constant>) {
        let mut fargs = vec![];

        let (farity, body) = match self.pop() {
            Constant::Fun { arity, body } => (arity, Either::Left(body)),
            Constant::PartialFun { arity, body, args } => {
                fargs = args;
                (arity, body)
            }
            Constant::NativeFun { arity, fp } => (arity, Either::Right(fp)),
            other => panic!("Can't call {}", other),
        };

        let mut old_fargs_len = 0;
        let len = fargs.len();
        while fargs.len() - len < carity {
            fargs.insert(old_fargs_len, self.pop());
            old_fargs_len += 1;
        }

        (body, farity, fargs)
    }

    fn call(&mut self, carity: usize) {
        let (body, farity, fargs) = self.call_helper(carity);
        match carity.cmp(&farity) {
            Ordering::Greater => panic!(
                "function expected {} arguments, but received {}",
                farity, carity
            ),
            Ordering::Less => self.push(Constant::PartialFun {
                arity: farity - carity,
                body,
                args: fargs,
            }),
            Ordering::Equal => {
                let curr_env = mem::replace(&mut self.variables, Env::new());
                match body {
                    Either::Left(bytecode) => {
                        fargs.into_iter().for_each(|it| self.push(it));
                        self.run(bytecode);
                    }
                    Either::Right(fp) => {
                        let ret = fp(fargs);
                        self.push(ret)
                    }
                }
                self.variables = curr_env;
            }
        }
    }

    fn tcall(&mut self, carity: usize) {
        let (body, farity, fargs) = self.call_helper(carity);
        match carity.cmp(&farity) {
            Ordering::Greater => panic!(
                "function expected {} arguments, but received {}",
                farity, carity
            ),

            Ordering::Less => {
                panic!("Can't use partial application in a tail call")
            }
            Ordering::Equal => {
                fargs.into_iter().for_each(|it| self.push(it));

                if body.as_ref() == Either::Left(self.bytecode()) {
                    *self.ip() = 0;
                } else {
                    panic!("Can't use tail calls with different functions")
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
        let prelude = prelude::prelude();

        Self {
            constants: vec![],
            call_stack: StackVec::new(),
            stack: StackVec::new(),
            globals: prelude,
            variables: Env::new(),
        }
    }
}
