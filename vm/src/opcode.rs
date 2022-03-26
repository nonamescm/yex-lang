use crate::Symbol;

/// OpCodes for the virtualMachine
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum OpCode {
    /// Stops the virtual machine
    Halt,

    /// Push a value by it's index on the constant table on-to the stack
    /// The stack layout before running this opcode: []
    /// The stack layout after running it: [constants[index]]
    Push(usize), // pointer to constant table

    /// Pop a value from the stack
    /// The stack layout before running this opcode: [c]
    /// The stack layout after running it: []
    Pop,

    /// Duplicates the top value of the stack
    /// The stack layout before running this opcode: [c]
    /// The stack layout after running it: [c, c]
    Dup,

    /// Read a value from a variable, receives the index of the variable name in the constant table as
    /// argument
    /// The stack layout before running this opcode: []
    /// The stack layout after running it: [variable-value]
    Load(usize),

    /// Save a value to a variable
    /// The stack layout before running this opcode: [value-to-save]
    /// The stack layout after running it: []
    Save(usize),

    /// Read a value from a variable
    /// The stack layout before running this opcode: []
    /// The stack layout after running it: [variable-value]
    Loag(Symbol),

    /// Save a value to a global variable
    /// The stack layout before running this opcode: [value-to-save]
    /// The stack layout after running it: []
    Savg(Symbol),

    /// Drops a variable, receives the index of the variable name in the constant table as argument
    /// The stack layout before running this opcode: []
    /// The stack layout after running it: []
    Drop(usize),

    /// Jump if the value on the stack top is false, receives the jump address as argument
    /// The stack layout before running this opcode: [cond]
    /// The stack layout after running it: []
    Jmf(usize),

    /// Unconditional jump, receives the jump address as argument
    /// The stack layout before running this opcode: []
    /// The stack layout after running it: []
    Jmp(usize),

    /// Calls the value on the top of the stack, pushing the return value
    /// The stack layout before running this opcode: [fun, ...args]
    /// The stack layout after running it: [return-value]
    Call(usize),

    /// same as call but with tail optimization
    /// The stack layout before running this opcode: [fun, ...args]
    /// The stack layout after running it: [return-value]
    TCall(usize),

    /// Prepends a value to a list, pushing a new list
    /// The stack layout before running this opcode: [list, value]
    /// The stack layout after running it: [new-list]
    Prep,

    /// Swap the two first elements in the stack
    /// The stack layout before running this opcode: [a, b]
    /// The stack layout after running it: [b, a]
    Rev,

    /// Add the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Add,

    /// Gets the remainder of the division of the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Rem,

    /// Subtract the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Sub,

    /// Multiplicate the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Mul,

    /// Divide the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Div,

    /// Negates the value on the stack top
    /// The stack layout before running this opcode: [const]
    /// The stack layout after running it: [result]
    Neg,

    /// Returns the len of the value on the stack top
    /// The stack layout before running this opcode: [const]
    /// The stack layout after running it: [result]
    Len,

    /// Apply a unary not to the stack top
    /// The stack layout before running this opcode: [const]
    /// The stack layout after running it: [result]
    Not,

    /// Apply a xor operation on the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Xor,

    /// Apply shift-right operation on the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Shr,

    /// Apply shift-left operation on the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Shl,

    /// Apply bit-and operation on the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    BitAnd,

    /// Apply bit-or operation on the two values on the stack top
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    BitOr,

    /// Check if the two values on the stack tops are equal
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Eq,

    /// Check if the first value on the top of the stack is less than the second
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    Less,

    /// Check if the first value on the top of the stack is less or equal than the second
    /// The stack layout before running this opcode: [const1, const2]
    /// The stack layout after running it: [result]
    LessEq,

    /// Instantiates a new object
    /// The stack layout before running this opcode: [type, ...args]
    /// The stack layout after running it: [object]
    New,

    /// Access a field of a type
    /// The stack layout before running this opcode: [instance]
    /// The stack layout after running it: [field-value]
    Get(Symbol),

    /// Calls a method of a type
    /// The stack layout before running this opcode: [instance, ...args]
    /// The stack layout after running it: [return-value]
    Invk(Symbol, usize),
}

/// Stocks the [`crate::OpCode`] with the line and the column of it on the original source code,
/// make it possible to be used for error handling
#[derive(Clone, Copy, Eq)]
pub struct OpCodeMetadata {
    /// Source's code line
    pub line: usize,
    /// Source's code column
    pub column: usize,
    /// Actual opcode
    pub opcode: OpCode,
}

impl OpCodeMetadata {
    /// Creates a new [`OpCodeMetadata`]
    pub fn new(line: usize, column: usize, opcode: OpCode) -> Self {
        Self { line, column, opcode }
    }
}

impl std::fmt::Debug for OpCodeMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.opcode)
    }
}

impl std::cmp::PartialEq for OpCodeMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.opcode == other.opcode
    }
}
