#![deny(missing_docs)]
//! Virtual Machine implementation for the yex programming language
mod env;
mod error;

#[doc(hidden)]
pub mod gc;

mod list;
mod literal;
mod opcode;
mod prelude;
mod stack;
#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Clone)]
/// Either left or right value
pub enum Either<L, R> {
    /// The left value
    Left(L),
    /// The right value
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Returns self by reference
    pub fn as_ref(&self) -> Either<&L, &R> {
        match *self {
            Self::Left(ref inner) => Either::Left(inner),
            Self::Right(ref inner) => Either::Right(inner),
        }
    }
}

use std::{cmp::Ordering, mem};

use gc::GcRef;

use crate::{
    env::Env,
    error::{InterpretError, InterpretResult},
    literal::{nil, FunBody},
    stack::StackVec,
};

pub use crate::{
    env::Table,
    list::List,
    literal::{symbol::Symbol, Constant, Fun},
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
            Err($crate::error::InterpretError { line: $crate::LINE, column: $crate::COLUMN, err: msg })
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
        self.constants = constants.into_iter().collect();
    }

    /// Pop's the last value on the stack
    pub fn pop_last(&self) -> &Constant {
        self.stack.last().unwrap_or(&Constant::Nil)
    }

    /// Executes a given set of bytecode instructions
    pub fn run(&mut self, bytecode: Bytecode) -> InterpretResult<Constant> {
        self.call_stack.push(CallFrame::new(bytecode));

        macro_rules! binop {
            ($op:tt) => {{
                let right = self.pop();
                let left = self.pop();

                self.push((left $op right)?)
            }}
        }

        macro_rules! unaop {
            ($op:tt) => {{
                let right = self.pop();

                self.push(($op right)?)
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
                    if self.constants.len() <= n {
                        panic!("err: can't find consts. Are you in repl?")?;
                    }

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
                            None => return panic!("unknown variable {}", val),
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

                Call(carity) => self.call(carity)?,
                TCall(carity) => self.tcall(carity)?,

                Prep => {
                    let val = self.pop();

                    match self.pop() {
                        Constant::List(xs) => {
                            self.push(Constant::List(GcRef::new(xs.prepend(val))))
                        }
                        other => return panic!("Expected a list, found a `{}`", other),
                    };
                }

                Insert(key) => {
                    let value = self.pop();
                    let len = self.stack.len() - 1;

                    match &mut self.stack[len] {
                        Constant::Table(ts) => {
                            ts.insert(key, value);
                        }
                        other => return panic!("Expected a table, found a `{}`", other),
                    };
                }

                Index => self.index()?,

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

        Ok(Constant::Nil)
    }

    fn call_helper(&mut self, carity: usize) -> InterpretResult<(FunBody, usize, Vec<Constant>)> {
        let mut fargs;

        let (farity, body) = match self.pop() {
            Constant::Fun(f) => {
                fargs = f.args.clone();
                (f.arity, f.body.clone())
            }
            other => return panic!("Can't call {}", other),
        };

        let mut old_fargs_len = 0;
        let len = fargs.len();
        while fargs.len() - len < carity {
            fargs.insert(old_fargs_len, self.pop());
            old_fargs_len += 1;
        }

        Ok((body, farity, fargs))
    }

    fn call(&mut self, carity: usize) -> InterpretResult<()> {
        let (body, farity, fargs) = self.call_helper(carity)?;
        match carity.cmp(&farity) {
            Ordering::Greater => {
                return panic!(
                    "function expected {} arguments, but received {}",
                    farity, carity
                )
            }
            Ordering::Less => self.push(Constant::Fun(GcRef::new(literal::Fun {
                arity: farity - carity,
                body,
                args: fargs,
            }))),
            Ordering::Equal => {
                let curr_env = mem::replace(&mut self.variables, Env::new());
                match body.get() {
                    Either::Left(bytecode) => {
                        fargs.into_iter().for_each(|it| self.push(it));
                        self.run(bytecode.clone())?;
                    }
                    Either::Right(fp) => {
                        let ret = fp(self, fargs.into_iter().rev().collect());
                        self.push(ret)
                    }
                }
                self.variables = curr_env;
            }
        }
        Ok(())
    }

    fn tcall(&mut self, carity: usize) -> InterpretResult<()> {
        let (body, farity, fargs) = self.call_helper(carity)?;
        match carity.cmp(&farity) {
            Ordering::Greater => panic!(
                "function expected {} arguments, but received {}",
                farity, carity
            )?,

            Ordering::Less => panic!("Can't use partial application in a tail call")?,
            Ordering::Equal => {
                fargs.into_iter().for_each(|it| self.push(it));

                if body.get().as_ref() == Either::Left(self.bytecode()) {
                    *self.ip() = 0;
                } else {
                    panic!("Can't use tail calls with different functions")?
                }
            }
        }
        Ok(())
    }

    fn index(&mut self) -> InterpretResult<()> {
        match self.pop() {
            Constant::Num(n) if n.fract() == 0.0 && n >= 0.0 => match &self.pop() {
                Constant::List(xs) => self.push(xs.index(n as usize)),
                other => panic!("Expected a list to index, found a `{}`", other)?,
            },

            Constant::Sym(key) => match &self.pop() {
                Constant::Table(ts) => self.push(ts.get().get(&key).unwrap_or_else(nil)),
                other => panic!("Expected a table to index, found a `{}`", other)?,
            },

            other => return panic!("Expected a integer to use as index, found a `{}`", other),
        };
        Ok(())
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
