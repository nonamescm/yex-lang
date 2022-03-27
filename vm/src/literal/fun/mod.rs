use crate::{
    error::InterpretResult, gc::GcRef, stackvec, Bytecode, StackVec, Value, VirtualMachine,
};
pub type NativeFun = fn(*mut VirtualMachine, Vec<Value>) -> InterpretResult<Value>;
pub type FunBody = GcRef<FnKind>;
pub type FunArgs = StackVec<Value, 8>;

#[derive(Debug, Clone, PartialEq)]
/// The kind of a function.
pub enum FnKind {
    /// A native function.
    Native(NativeFun),
    /// A function defined in the source code.
    Bytecode(Bytecode),
}

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
            body: GcRef::new(FnKind::Bytecode(body)),
            args: FunArgs::new(),
        }
    }

    /// Create a new native function
    pub fn new_native(arity: usize, native: NativeFun) -> Self {
        Self {
            arity,
            body: GcRef::new(FnKind::Native(native)),
            args: FunArgs::new(),
        }
    }

    /// Converts the Fun to a GcRef<Fun>
    #[must_use]
    pub fn to_gcref(self) -> GcRef<Fun> {
        GcRef::new(self)
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

    /// Checks if the function is a native function
    pub fn is_native(&self) -> bool {
        matches!(*self.body, FnKind::Native(_))
    }

    /// Checks if the function is a bytecode function
    pub fn is_bytecode(&self) -> bool {
        matches!(*self.body, FnKind::Bytecode(_))
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
