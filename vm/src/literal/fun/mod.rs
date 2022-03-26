use crate::{
    error::InterpretResult, gc::GcRef, stackvec, Bytecode, Either, StackVec, Value, VirtualMachine,
};
pub type NativeFun = fn(*mut VirtualMachine, Vec<Value>) -> InterpretResult<Value>;
pub type FunBody = GcRef<Either<Bytecode, NativeFun>>;
pub type FunArgs = StackVec<Value, 8>;

#[derive(PartialEq, Clone)]
/// Yex function struct
pub struct Fun {
    /// The number of argument the function receives
    pub arity: usize,
    /// The function body
    pub body: FunBody,
    /// The function Arguments
    pub args: FunArgs,
}

impl Fun {
    /// Create a new function
    pub fn new_bt(arity: usize, body: Bytecode) -> Self {
        Self {
            arity,
            body: GcRef::new(Either::Left(body)),
            args: FunArgs::new(),
        }
    }

    /// Create a new native function
    pub fn new_native(arity: usize, native: NativeFun) -> Self {
        Self {
            arity,
            body: GcRef::new(Either::Right(native)),
            args: FunArgs::new(),
        }
    }

    /// Apply the function to the given arguments
    pub fn apply(&self, app: FunArgs) -> Self {
        let mut args = stackvec![];
        for arg in app.iter().rev().chain(self.args.iter()) {
            args.push(arg.clone());
        }

        Fun {
            arity: self.arity + self.args.len() - args.len(),
            body: self.body.clone(),
            args,
        }
    }
}

impl std::fmt::Debug for Fun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fun {{ arity: {}, body: {:?} }}", self.arity, self.body)
    }
}

impl std::fmt::Display for Fun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun({})>", self.arity)
    }
}
