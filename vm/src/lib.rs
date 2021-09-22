mod literal;
#[cfg(test)]
mod tests;
pub use literal::Literal;

pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Push(Literal),
    Ret,
    Halt,
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
            Err(e) => panic!("[{}:{}] {}", self.line, self.column, e),
        }
    }

    fn run_instruction(&mut self, intr: Instruction) -> Option<Literal> {
        match intr {
            Instruction::Add => {
                let second = self.pop();
                let first = self.pop();

                self.push(self.try_do(first + second));
            }
            Instruction::Sub => {
                let second = self.pop();
                let first = self.pop();

                self.push(self.try_do(first - second));
            }
            Instruction::Mul => {
                let second = self.pop();
                let first = self.pop();

                self.push(self.try_do(first * second));
            }
            Instruction::Div => {
                let second = self.pop();
                let first = self.pop();

                self.push(self.try_do(first / second));
            }
            Instruction::Neg => {
                let first = self.pop();
                self.push(self.try_do(-first));
            }
            Instruction::Push(lit) => {
                self.push(lit);
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
