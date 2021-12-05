use crate::{Constant, Symbol};

/// OpCodes for the virtualMachine
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum OpCode {
    /// Stops the virtual machine
    Halt,

    /// Push a value by it's index on the constant table on-to the stack
    Push(usize), // pointer to constant table

    /// Pop a value from the stack
    Pop,

    /// Read a value from a variable, receives the index of the variable name in the constant table as
    /// argument
    Load(Symbol),

    /// Save a value to a variable
    Save(Symbol),

    /// Drops a variable, receives the index of the variable name in the constant table as argument
    Drop(Symbol),

    /// Jump if the value on the stack top is false
    Jmf(usize),

    /// Unconditional jump
    Jmp(usize),

    /// Creates a new scope
    Nsc,

    /// Ends a scope
    Esc,

    /// Calls the value on the top of the stack
    Call(usize /* number of arguments */),

    /// same as call but with tail optimization
    TCall(usize),

    /// Calls a native rust function
    Cnll(fn(Constant) -> Constant),

    /// Prepends a value to a list
    Prep,

    /// Add the two values on the stack top
    Add,

    /// Subtract the two values on the stack top
    Sub,

    /// Multiplicate the two values on the stack top
    Mul,

    /// Divide the two values on the stack top
    Div,

    /// Negates the value on the stack top
    Neg,

    /// Apply a unary not to the stack top
    Not,

    /// Apply a xor operation on the two values on the stack top
    Xor,

    /// Apply shift-right operation on the two values on the stack top
    Shr,

    /// Apply shift-left operation on the two values on the stack top
    Shl,

    /// Apply bit-and operation on the two values on the stack top
    BitAnd,

    /// Apply bit-or operation on the two values on the stack top
    BitOr,

    /// Check if the two values on the stack tops are equal
    Eq,
}

/// Stocks the [`crate::OpCode`] with the line and the column of it on the original source code,
/// make it possible to be used for error handling
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OpCodeMetadata {
    /// Source's code line
    pub line: usize,
    /// Source's code column
    pub column: usize,
    /// Actual opcode
    pub opcode: OpCode,
}

impl std::fmt::Debug for OpCodeMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.opcode)
    }
}
