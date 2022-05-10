#![deny(missing_docs)]
#![allow(unused_unsafe)]
#![deny(clippy::all)]
#![allow(clippy::unit_arg, clippy::option_map_unit_fn)]
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
    tuple::Tuple,
    TryGet,
};

use crate::error::InterpretResult;

pub use crate::{
    env::EnvTable,
    literal::{
        fun::{Fn, FnKind},
        list::List,
        symbol::Symbol,
        table::YexStruct,
        yexmodule::YexModule,
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
    ($err: ident, $($fmtargs:expr),*) => {{
        Err($crate::raise_err!($err, $($fmtargs),*))
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! raise_err {
    ($error: ident, $($fmtargs:expr),*) => {
        unsafe {
            let msg = $crate::Symbol::new(stringify!($error));
            $crate::error::InterpretError {
                line: $crate::LINE,
                column: $crate::COLUMN,
                err: msg,
                msg: format!($($fmtargs),*),
            }
        }
    };
}

type Stack = StackVec<Value, STACK_SIZE>;

/// Bytecode for the virtual machine, contains the instructions to be executed and the constants to
/// be loaded
pub type Bytecode = Vec<OpCodeMetadata>;

type BytecodeRef<'a> = &'a Bytecode;
use std::{mem::swap, ops, ptr};
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
        let bytecode = &*bytecode;
        let mut try_stack = vec![];

        let mut ip = 0;
        let mut frame_locals = 0;

        while ip < bytecode.len() {
            let op = unsafe {
                let op = bytecode[ip];
                LINE = op.line;
                COLUMN = op.column;
                op.opcode
            };

            self.debug_stack(&op);

            let res = match op {
                OpCode::Try(offset) => {
                    try_stack.push(offset);
                    Ok(())
                }

                OpCode::EndTry => {
                    try_stack.pop();
                    Ok(())
                }

                OpCode::Jmp(offset) => {
                    ip = offset;
                    continue;
                }

                OpCode::Jmf(offset) => {
                    if !self.pop().to_bool() {
                        ip = offset;
                        continue;
                    }
                    Ok(())
                }

                OpCode::TCall(arity) => {
                    self.valid_tail_call(arity, bytecode)?;
                    ip = 0;
                    continue;
                }

                _ => self.run_op(op, &mut frame_locals),
            };

            if let Err(e) = res {
                if try_stack.is_empty() {
                    return Err(e);
                }

                let try_ip = try_stack.pop().unwrap();
                self.push(e.err.into());
                ip = try_ip;
            }

            ip += 1;
        }

        self.used_locals -= frame_locals;

        Ok(())
    }

    #[inline(always)]
    fn run_op(&mut self, op: OpCode, frame_locals: &mut usize) -> InterpretResult<()> {
        match op {
            OpCode::Nop => (),

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

            OpCode::Swap(a, b) => unsafe {
                let a = self.stack.get_uninit_mut(a) as *mut _;
                let b = self.stack.get_uninit_mut(b) as *mut _;

                ptr::swap(a, b);
            },

            OpCode::Rev => {
                let (a, b) = self.pop_two();
                self.push(b);
                self.push(a);
            }

            // function calls
            OpCode::Call(arity) => self.call(arity)?,

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
                let value = self.locals[offset + self.used_locals - *frame_locals].clone();
                self.push(value);
            }
            OpCode::Save(offset) => {
                let value = self.pop();

                self.used_locals += 1;
                *frame_locals += 1;
                self.locals[offset + (self.used_locals - *frame_locals)] = value;
            }
            OpCode::Drop(_) => {
                *frame_locals -= 1;
                self.used_locals -= 1;
            }

            // globals manipulation
            OpCode::Loag(name) => {
                let value = match self.get_global(name) {
                    Some(value) => value,
                    None => raise!(NameError, "Undefined variable '{}'", name)?,
                };
                self.push(value);
            }
            OpCode::Savg(name) => {
                let value = self.pop();
                if self.globals.get(&name).is_some() {
                    raise!(NameError, "Tried to reassign global variable '{}'", name)?;
                }
                self.set_global(name, value);
            }

            // list manipulation
            OpCode::Prep => {
                let list: List = self.pop().get()?;
                let value = self.pop();

                self.push(list.prepend(value).into());
            }

            OpCode::New => {
                todo!()
            }

            OpCode::Get(field) => {
                let obj: YexStruct = self.pop().get()?;

                let value = obj.get(field);
                self.push(value);
            }

            OpCode::Set(field) => unsafe {
                let value = self.pop();
                let obj = self
                    .stack
                    .get_uninit_mut(self.stack.len() - 1)
                    .assume_init_mut();

                let obj = match obj {
                    Value::Struct(obj) => obj,
                    _ => raise!(TypeError, "Expected a struct")?,
                };

                let struct_fields = &obj.module.struct_fields;

                if !struct_fields.is_empty() && !struct_fields.contains(&field) {
                    raise!(
                        NameError,
                        "Undefined field '{}' for struct '{}'",
                        field,
                        obj.module.name.as_str()
                    )?;
                }

                obj.items = obj.items.prepend(vec![field.into(), value].into());
            },

            OpCode::Type => {
                let value = self.pop();
                self.push(Value::Module(value.type_of()));
            }

            OpCode::Ref(method) => {
                let ty: GcRef<YexModule> = self.pop().get()?;

                let method = ty.fields.get(&method).ok_or(raise_err!(
                    FieldError,
                    "Undefined method '{}' for type '{}'",
                    method,
                    ty.name
                ))?;

                self.push(method);
            }

            OpCode::Tup(len) => {
                let mut tup = vec![];
                for _ in 0..len {
                    tup.push(self.pop());
                }
                self.push(tup.into());
            }

            OpCode::TupGet(index) => {
                let tup: Tuple = self.pop().get()?;
                let elem = tup.0.get(index).unwrap(); // this SHOULD be unreachable
                self.push(elem.clone());
            }

            OpCode::Struct(name) => {
                let ty: GcRef<YexModule> = if let Some(name) = name {
                    self.get_global(name)
                        .ok_or(raise_err!(NameError, "Undefined module '{}'", name))
                        .and_then(|ty| ty.get())
                        .and_then(|ty: GcRef<YexModule>| {
                            if ty.struct_ {
                                Ok(ty)
                            } else {
                                raise!(TypeError, "Expected a struct")
                            }
                        })?
                } else {
                    GcRef::new(YexModule::struct_())
                };

                self.push(YexStruct::new(ty).into());
            }

            // these opcodes are handled by the run function, since they can manipulate the ip
            OpCode::Try(..)
            | OpCode::EndTry
            | OpCode::Jmp(..)
            | OpCode::Jmf(..)
            | OpCode::TCall(..) => unreachable!(),
        };

        Ok(())
    }

    #[cfg(debug_assertions)]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self, instruction: &OpCode) {
        eprintln!("Stack: {:?} ({instruction:?})", self.stack);
    }

    #[cfg(not(debug_assertions))]
    /// Debug the values on the stack and in the bytecode
    pub fn debug_stack(&self, _: &OpCode) {}

    #[inline(always)]
    fn call_args(&mut self, arity: usize, fun: &Fn) -> Option<FnArgs> {
        if fun.is_bytecode() && fun.args.is_empty() {
            return None;
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

        Some(args)
    }

    #[inline(always)]
    pub(crate) fn call(&mut self, arity: usize) -> InterpretResult<()> {
        let fun: GcRef<Fn> = self.pop().get()?;

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
            raise!(
                CallError,
                "Too many arguments passed for function {:?}",
                fun
            )?;
        }

        match &*fun.body {
            FnKind::Bytecode(bytecode) => self.call_bytecode(bytecode, args),
            FnKind::Native(ptr) => self.call_native(*ptr, args),
        }
    }

    #[inline(always)]
    fn call_bytecode(
        &mut self,
        bytecode: BytecodeRef,
        args: Option<FnArgs>,
    ) -> InterpretResult<()> {
        self.used_locals += 1;

        args.map(|stack| {
            for arg in stack {
                self.push(arg)
            }
        });

        self.run(bytecode)?;
        self.used_locals -= 1;
        Ok(())
    }

    #[inline(always)]
    fn call_native(&mut self, fp: NativeFn, args: Option<FnArgs>) -> InterpretResult<()> {
        let args = args.unwrap_or_else(FnArgs::new).reverse().into();
        let result = fp(self, args);
        self.try_push(result)
    }

    #[inline]
    fn valid_tail_call(&mut self, arity: usize, frame: BytecodeRef) -> InterpretResult<()> {
        let fun: GcRef<Fn> = self.pop().get()?;

        match &*fun.body {
            FnKind::Bytecode(_) if fun.arity != arity => {
                raise!(TailCallError, "")
            }
            FnKind::Bytecode(bytecode) if bytecode != frame => {
                raise!(TailCallError, "")
            }
            FnKind::Native(_) => {
                raise!(TailCallError, "")
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
