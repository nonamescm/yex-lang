use crate::{
    error::InterpretResult, gc::GcRef, stackvec, Bytecode, StackVec, Value, VirtualMachine,
};
pub type NativeFn = fn(*mut VirtualMachine, Vec<Value>) -> InterpretResult<Value>;
pub type FnBody = GcRef<FnKind>;
pub type FnArgs = StackVec<Value, 8>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// The kind of a function.
pub enum FnKind {
    /// A native function.
    Native(NativeFn),
    /// A function defined in the source code.
    Bytecode(Bytecode),
}

#[derive(PartialEq, Clone)]
/// Yex function struct
pub struct Fn {
    /// The number of argument the function receives
    pub arity: usize,
    /// The function body
    pub body: FnBody,
    /// The function Arguments
    pub args: FnArgs,
}

impl Fn {
    /// Create a new function
    pub fn new_bt(arity: usize, body: Bytecode) -> Self {
        Self {
            arity,
            body: GcRef::new(FnKind::Bytecode(body)),
            args: FnArgs::new(),
        }
    }

    /// Create a new native function
    pub fn new_native(arity: usize, native: NativeFn) -> Self {
        Self {
            arity,
            body: GcRef::new(FnKind::Native(native)),
            args: FnArgs::new(),
        }
    }

    /// Converts the Fn to a GcRef<Fn>
    #[must_use]
    pub fn to_gcref(self) -> GcRef<Fn> {
        GcRef::new(self)
    }

    /// Apply the function to the given arguments
    pub fn apply(&self, app: FnArgs) -> Self {
        let mut args = stackvec![];
        for arg in app.iter().rev().chain(self.args.iter()) {
            args.push(arg.clone());
        }

        Fn {
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

impl std::fmt::Debug for Fn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fn {{ arity: {}, body: {:?} }}", self.arity, self.body)
    }
}

impl std::fmt::Display for Fn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun({})>", self.arity)
    }
}
