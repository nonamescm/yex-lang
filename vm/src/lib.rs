#![deny(missing_docs)]
#![allow(unused_unsafe)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]
//! Virtual Machine implementation for the yex programming language
mod either;
mod env;
mod error;
#[doc(hidden)]
pub mod gc;
mod list;
mod literal;
mod opcode;
mod prelude;
mod stack;
mod table;

use env::EnvTable;
use gc::GcRef;
use literal::{FunArgs, NativeFun};

use crate::error::InterpretResult;

pub use crate::{
    either::Either,
    list::List,
    literal::{symbol::Symbol, Fun, Value},
    opcode::{OpCode, OpCodeMetadata},
    stack::StackVec,
    table::Table,
};

const STACK_SIZE: usize = 512;

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

type Stack = StackVec<Value, STACK_SIZE>;

/// Bytecode for the virtual machine, contains the instructions to be executed and the constants to
/// be loaded
pub type Bytecode = Vec<OpCodeMetadata>;
type BytecodeRef<'a> = &'a [OpCodeMetadata];
use dlopen::raw::Library;
use std::{collections::HashMap, mem::swap};
/// Implements the Yex virtual machine, which runs the [`crate::OpCode`] instructions in a stack
/// model
pub struct VirtualMachine {
    dlopen_libs: HashMap<String, GcRef<Library>>,
    stack: Stack,
    locals: [Value; 512],
    used_locals: usize,
    constants: Vec<Value>,
    globals: EnvTable,
}

impl VirtualMachine {
    /// Reset the instruction pointer and the stack
    pub fn reset(&mut self) {
        self.stack = stackvec![];
    }

    /// sets the constants for execution
    pub fn set_consts(&mut self, constants: Vec<Value>) {
        self.constants = constants;
    }

    /// Pop's the last value on the stack
    pub fn pop_last(&self) -> &Value {
        self.stack.last().unwrap_or(&Value::Nil)
    }

    /// Get the value of a global variable
    pub fn get_global<T: Into<Symbol>>(&self, name: T) -> Option<Value> {
        self.globals.get(&name.into())
    }

    /// Set the value of a global variable
    pub fn set_global<T: Into<Symbol>>(&mut self, name: T, value: Value) {
        self.globals.insert(name.into(), value);
    }

    /// Executes a given set of bytecode instructions
    pub fn run(&mut self, bytecode: BytecodeRef) -> InterpretResult<Value> {
        const NIL: Value = Value::Nil;
        let mut ip = 0;
        let mut frame_locals = 0;

        while ip < bytecode.len() {
            unsafe {
                LINE = bytecode[ip].line;
                COLUMN = bytecode[ip].column;
            }

            let op = bytecode[ip].opcode;

            self.debug_stack(&op);

            match op {
                OpCode::Halt => return Ok(Value::Nil),

                // Stack manipulation
                OpCode::Push(value) => {
                    let value = self.constants[value].clone();
                    self.push(value);
                }
                OpCode::Pop => {
                    self.pop();
                }

                OpCode::Dup => {
                    let value = self.pop();
                    self.push(value.clone());
                    self.push(value);
                }

                OpCode::Rev => {
                    let (a, b) = self.pop_two();
                    self.push(b);
                    self.push(a);
                }

                // jump instructions
                OpCode::Jmp(offset) => {
                    ip = offset;
                    continue;
                }
                OpCode::Jmf(offset) => {
                    if !self.pop().to_bool() {
                        ip = offset;
                        continue;
                    }
                }

                // function calls
                OpCode::Call(arity) => self.call(arity)?,
                OpCode::TCall(arity) => {
                    self.valid_tail_call(arity, bytecode)?;
                    ip = 0;
                    continue;
                }

                // mathematical operators
                OpCode::Add => self.binop(|a, b| a + b)?,
                OpCode::Sub => self.binop(|a, b| a - b)?,
                OpCode::Mul => self.binop(|a, b| a * b)?,
                OpCode::Div => self.binop(|a, b| a / b)?,
                OpCode::Rem => self.binop(|a, b| a % b)?,

                // bitwise operators
                OpCode::BitAnd => self.binop(|a, b| a & b)?,
                OpCode::BitOr => self.binop(|a, b| a | b)?,
                OpCode::Xor => self.binop(|a, b| a ^ b)?,
                OpCode::Shl => self.binop(|a, b| a << b)?,
                OpCode::Shr => self.binop(|a, b| a >> b)?,

                // comparison operators
                OpCode::Eq => self.binop(|a, b| Ok(a == b))?,
                OpCode::Less => {
                    let (a, b) = self.pop_two();
                    self.push(a.ord_cmp(&b)?.is_lt().into());
                }
                OpCode::LessEq => {
                    let (a, b) = self.pop_two();
                    self.push(a.ord_cmp(&b)?.is_le().into());
                }

                // unary operators
                OpCode::Not => {
                    let value = self.pop();
                    self.push(!value);
                }
                OpCode::Len => {
                    let value = self.pop();
                    self.push(Value::Num(value.len() as f64));
                }
                OpCode::Neg => {
                    let value = self.pop();
                    self.try_push(-value)?;
                }

                // locals manipulation
                OpCode::Load(offset) => {
                    let value = self.locals[offset + self.used_locals - frame_locals].clone();
                    self.push(value);
                }
                OpCode::Save(offset) => {
                    let value = self.pop();
                    if self.locals[offset] != NIL {
                        self.used_locals += 1;
                        frame_locals += 1;
                    }
                    self.locals[offset + self.used_locals - frame_locals] = value;
                }
                OpCode::Drop(offset) => {
                    self.locals[offset + self.used_locals - frame_locals] = Value::Nil;
                    frame_locals -= 1;
                    self.used_locals -= 1;
                }

                // globals manipulation
                OpCode::Loag(name) => {
                    let value = match self.get_global(name) {
                        Some(value) => value,
                        None => panic!("Undefined global variable: {}", name)?,
                    };
                    self.push(value);
                }
                OpCode::Savg(name) => {
                    let value = self.pop();
                    self.set_global(name, value);
                }

                // table manipulation
                OpCode::Insert(key) => {
                    let (table, value) = self.pop_two();
                    let table = match table {
                        Value::Table(table) => table.insert(key, value),
                        value => panic!("Expected table, got {}", value)?,
                    };
                    let table = Value::Table(GcRef::new(table.clone()));
                    self.push(table);
                }

                // list manipulation
                OpCode::Prep => {
                    let value = self.pop();
                    let list = match self.pop() {
                        Value::List(list) => list.prepend(value),
                        value => panic!("Expected list, got {}", value)?,
                    };

                    self.push(Value::List(list));
                }
            }

            ip += 1;
        }

        self.used_locals -= frame_locals;

        Ok(self.pop_last().clone())
    }

    #[cfg(debug_assertions)]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self, instruction: &OpCode) {
        eprintln!("Stack: {:?} ({instruction:?})", self.stack);
    }

    #[cfg(not(debug_assertions))]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self, _: &OpCode) {}

    #[inline]
    fn call_args(&mut self, arity: usize, applied: &FunArgs) -> FunArgs {
        let mut args = FunArgs::new();
        for _ in 0..arity {
            args.push(self.pop());
        }
        for arg in applied.iter() {
            args.push(arg.clone());
        }
        args
    }

    pub(crate) fn call(&mut self, arity: usize) -> InterpretResult<()> {
        let fun = match self.pop() {
            Value::Fun(f) => f,
            value => panic!("Expected a function to call, found {value}")?,
        };

        let args = self.call_args(arity, &fun.args);

        if arity < fun.arity {
            let ap = (*fun).clone().apply(args);
            self.push(Value::Fun(GcRef::new(ap)));
            return Ok(());
        }

        match &*fun.body {
            Either::Left(bytecode) => self.call_bytecode(bytecode, args),
            Either::Right(ptr) => self.call_native(*ptr, args),
        }
    }

    #[inline]
    fn call_bytecode(&mut self, bytecode: BytecodeRef, args: FunArgs) -> InterpretResult<()> {
        for arg in args.iter() {
            self.push(arg.clone());
        }

        self.run(bytecode)?;
        Ok(())
    }

    #[inline]
    fn call_native(&mut self, fp: NativeFun, args: FunArgs) -> InterpretResult<()> {
        let args = args.iter().rev().cloned().collect();
        let result = fp(self, args);
        self.try_push(result)
    }

    #[inline]
    fn valid_tail_call(&mut self, arity: usize, frame: BytecodeRef) -> InterpretResult<()> {
        let fun = match self.pop() {
            Value::Fun(fun) => fun,
            value => panic!("Expected a function, found {value}")?,
        };
        match &*fun.body {
            Either::Left(_) if fun.arity != arity => {
                panic!(
                    "Expected function with arity {}, found {}",
                    arity, fun.arity
                )
            }
            Either::Left(bytecode) if bytecode != frame => {
                panic!("Tried to tail call a function with a different bytecode")
            }
            Either::Right(_) => {
                panic!("Tried to use a tail call on a non-tail callable function")
            }
            Either::Left(_) => Ok(()),
        }
    }

    #[track_caller]
    pub(crate) fn push(&mut self, constant: Value) {
        self.stack.push(constant)
    }

    #[track_caller]
    pub(crate) fn pop(&mut self) -> Value {
        self.stack.pop()
    }

    fn binop<T, F>(&mut self, f: F) -> InterpretResult<()>
    where
        T: Into<Value>,
        F: Fn(Value, Value) -> InterpretResult<T>,
    {
        let (a, b) = self.pop_two();
        Ok(self.push(f(a, b)?.into()))
    }

    fn pop_two(&mut self) -> (Value, Value) {
        let mut ret = (self.pop(), self.pop());
        swap(&mut ret.0, &mut ret.1);
        ret
    }

    fn try_push(&mut self, constant: InterpretResult<Value>) -> InterpretResult<()> {
        Ok(self.push(constant?))
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        const STACK: Stack = StackVec::new();
        const NIL: Value = Value::Nil;

        let prelude = prelude::prelude();
        Self {
            stack: STACK,
            locals: [NIL; 512],
            used_locals: 0,
            dlopen_libs: HashMap::new(),
            constants: Vec::new(),
            globals: prelude,
        }
    }
}
