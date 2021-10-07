mod literal;
#[cfg(test)]
mod tests;
pub use literal::Literal;

#[derive(PartialEq, Debug)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Xor,
    Shr,
    Shl,
    BitAnd,
    BitOr,
    Push(Literal),
    Ret,
    Halt,
}

macro_rules! panic_vm {
    ($($tt:tt)*) => {{
        std::panic::set_hook(Box::new(|_| {}));
        panic!($($tt)*)
    }}
}


pub struct VirtualMachine {
    stack: Vec<Literal>,
    line: usize,
    column: usize,
}

impl VirtualMachine {
    pub fn run(&mut self, code: Vec<Instruction>) -> Literal {
        for intr in code {
            if let Some(v) = self.run_instruction(intr) {
                return v;
            }
        }
        Literal::Nil
    }

    fn pop(&mut self) -> Literal {
        self.stack.pop().unwrap_or(Literal::Nil)
    }

    fn push(&mut self, literal: Literal) {
        self.stack.push(literal)
    }

    fn try_do<E: std::fmt::Display>(&self, result: Result<Literal, E>) -> Literal {
        match result {
            Ok(l) => l,
            Err(e) => panic_vm!("[{}:{}] {}", self.line, self.column, e),
        }
    }

    fn run_instruction(&mut self, intr: Instruction) -> Option<Literal> {
        match intr {
            Instruction::Add => {
                let right = self.pop();
                let left = self.pop();

                self.push(self.try_do(left + right));
            }
            Instruction::Sub => {
                let right = self.pop();
                let left = self.pop();

                self.push(self.try_do(left - right));
            }
            Instruction::Mul => {
                let right = self.pop();
                let left = self.pop();

                self.push(self.try_do(left * right));
            }
            Instruction::Div => {
                let right = self.pop();
                let left = self.pop();

                self.push(self.try_do(left / right));
            }
            Instruction::Neg => {
                let left = self.pop();
                self.push(self.try_do(-left));
            }
            Instruction::Push(lit) => {
                self.push(lit);
            }

            Instruction::Xor => {
                let right = self.pop();
                let left = self.pop();
                self.push(self.try_do(left ^ right))
            }
            Instruction::BitAnd => {
                let right = self.pop();
                let left = self.pop();
                self.push(self.try_do(left & right))
            }
            Instruction::BitOr => {
                let right = self.pop();
                let left = self.pop();
                self.push(self.try_do(left | right))
            }
            Instruction::Shr => {
                let right = self.pop();
                let left = self.pop();
                self.push(self.try_do(left >> right))
            }
            Instruction::Shl => {
                let right = self.pop();
                let left = self.pop();
                self.push(self.try_do(left << right))
            }

            Instruction::Ret => return Some(self.pop()),
            Instruction::Halt => {
                self.stack = vec![];
            }
        };
        None
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self {
            stack: vec![],
            line: 1,
            column: 1,
        }
    }
}
