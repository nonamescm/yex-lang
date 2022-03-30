#![deny(missing_docs)]
#![allow(unused_unsafe)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg)]
//! Virtual Machine implementation for the yex programming language
mod env;
mod error;
#[doc(hidden)]
pub mod gc;
mod literal;
mod opcode;
mod prelude;
mod stack;

use gc::GcRef;
use literal::{
    fun::{FnArgs, NativeFn},
    yextype::instantiate,
};

use crate::error::InterpretResult;

pub use crate::{
    env::EnvTable,
    literal::{
        fun::{Fn, FnKind},
        list::List,
        symbol::Symbol,
        yextype::YexType,
        Value,
    },
    opcode::{OpCode, OpCodeMetadata},
    stack::StackVec,
};

const STACK_SIZE: usize = 512;
const NIL: Value = Value::Nil;

static mut LINE: usize = 1;
static mut COLUMN: usize = 1;

#[macro_export]
#[doc(hidden)]
macro_rules! raise {
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

type BytecodeRef<'a> = &'a Bytecode;
use std::{mem::swap, ops};
/// Implements the Yex virtual machine, which runs the [`crate::OpCode`] instructions in a stack
/// model
pub struct VirtualMachine {
    stack: Stack,
    locals: [Value; 1024],
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
    pub fn run(&mut self, bytecode: BytecodeRef) -> InterpretResult<()> {
        let mut ip = 0;
        let mut frame_locals = 0;

        while ip < bytecode.len() {
            let bytecode = &*bytecode;
            let op = unsafe {
                let op = bytecode[ip];
                LINE = op.line;
                COLUMN = op.column;
                op.opcode
            };

            self.debug_stack(&op);

            match op {
                OpCode::Halt => return Ok(()),

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

                    self.used_locals += 1;
                    frame_locals += 1;
                    self.locals[offset + (self.used_locals - frame_locals)] = value;
                }
                OpCode::Drop(_) => {
                    frame_locals -= 1;
                    self.used_locals -= 1;
                }

                // globals manipulation
                OpCode::Loag(name) => {
                    let value = match self.get_global(name) {
                        Some(value) => value,
                        None => raise!("Undefined global variable: {}", name)?,
                    };
                    self.push(value);
                }
                OpCode::Savg(name) => {
                    let value = self.pop();
                    self.set_global(name, value);
                }

                // list manipulation
                OpCode::Prep => {
                    let value = self.pop();
                    let list = match self.pop() {
                        Value::List(list) => list.prepend(value),
                        value => raise!("Expected list, got {}", value)?,
                    };

                    self.push(Value::List(list));
                }

                OpCode::New(arity) => {
                    let ty = match self.pop() {
                        Value::Type(ty) => ty,
                        value => raise!("Expected type, got `{}`", value)?,
                    };

                    let mut args = vec![];
                    for _ in 0..arity {
                        args.push(self.pop());
                    }

                    instantiate(self, ty, args)?;
                }
                OpCode::Get(field) => {
                    let obj = match self.pop() {
                        Value::Instance(obj) => obj,
                        value => raise!("Expected instance, got `{}`", value)?,
                    };

                    let value = match obj.fields.get(&field) {
                        Some(value) => value.clone(),
                        None => raise!("Undefined field: {}", field)?,
                    };

                    self.push(value);
                }
                OpCode::Invk(name, arity) => self.invoke(name, arity)?,
            }

            ip += 1;
        }

        self.used_locals -= frame_locals;

        Ok(())
    }

    fn invoke(&mut self, name: Symbol, arity: usize) -> InterpretResult<()> {
        let value = self.pop();
        let ty = value.type_of();

        let mut args = stackvec![];
        let mut i = 1;
        for _ in 0..arity {
            unsafe { args.insert_at(arity - i, self.pop()) };
            i += 1;
        }
        unsafe { args.set_len(arity) };

        args.push(value);

        let method = match ty.fields.get(&name) {
            Some(value) => match value {
                Value::Fn(f) => f,
                _ => unreachable!(),
            },
            None => raise!("Undefined method: {}", name)?,
        };

        // arity + 1 because we push the receiver
        if method.arity != arity + 1 {
            raise!("Expected {} arguments, found {}", method.arity - 1, arity)?;
        }

        match &*method.body {
            FnKind::Bytecode(bt) => self.call_bytecode(bt, args),
            FnKind::Native(f) => self.call_native(*f, args),
        }
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
    fn call_args(&mut self, arity: usize, fun: &Fn) -> FnArgs {
        if fun.arity == arity && fun.is_bytecode() && fun.args.is_empty() {
            return stackvec![];
        }

        let mut args = stackvec![];

        let mut i = 1;
        for _ in 0..arity {
            unsafe { args.insert_at(arity - i, self.pop()) };
            i += 1;
        }

        unsafe { args.set_len(arity) };

        for arg in fun.args.iter() {
            args.push(arg.clone());
        }

        args
    }

    pub(crate) fn call(&mut self, arity: usize) -> InterpretResult<()> {
        let fun = match self.pop() {
            Value::Fn(f) => f,
            value => raise!("Expected a function to call, found {value}")?,
        };

        if arity < fun.arity {
            let mut args = stackvec![];
            for _ in 0..arity {
                args.push(self.pop());
            }
            self.push(Value::Fn(GcRef::new(fun.apply(args))));
            return Ok(());
        }

        let args = self.call_args(arity, &fun);

        if arity > fun.arity {
            raise!("Too many arguments for function {}", *fun)?;
        }

        if arity < fun.arity {
            self.push(Value::Fn(GcRef::new(fun.apply(args))));
            return Ok(());
        }

        match &*fun.body {
            FnKind::Bytecode(bytecode) => self.call_bytecode(bytecode, args),
            FnKind::Native(ptr) => self.call_native(*ptr, args),
        }
    }

    #[inline]
    fn call_bytecode(&mut self, bytecode: BytecodeRef, args: FnArgs) -> InterpretResult<()> {
        self.used_locals += 1;
        for arg in args {
            self.push(arg);
        }

        self.run(bytecode)?;
        self.used_locals -= 1;
        Ok(())
    }

    #[inline]
    fn call_native(&mut self, fp: NativeFn, args: FnArgs) -> InterpretResult<()> {
        let args = args.reverse().into();
        let result = fp(self, args);
        self.try_push(result)
    }

    #[inline]
    fn valid_tail_call(&mut self, arity: usize, frame: BytecodeRef) -> InterpretResult<()> {
        let fun = match self.pop() {
            Value::Fn(fun) => fun,
            value => raise!("Expected a function, found {value}")?,
        };
        match &*fun.body {
            FnKind::Bytecode(_) if fun.arity != arity => {
                raise!(
                    "Expected function with arity {}, found {}",
                    arity,
                    fun.arity
                )
            }
            FnKind::Bytecode(bytecode) if bytecode != frame => {
                raise!("Tried to tail call a function with a different bytecode")
            }
            FnKind::Native(_) => {
                raise!("Tried to use a tail call on a non-tail callable function")
            }
            FnKind::Bytecode(_) => Ok(()),
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
        F: ops::Fn(Value, Value) -> InterpretResult<T>,
    {
        let a = self.pop();
        let b = self.pop();
        Ok(self.push(f(b, a)?.into()))
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

        let prelude = prelude::prelude();
        Self {
            stack: STACK,
            locals: [NIL; 1024],
            used_locals: 0,
            constants: Vec::new(),
            globals: prelude,
        }
    }
}
