mod literal;
#[cfg(test)]
mod tests;

pub use literal::Literal;

type Address = u8;

pub enum Instruction {
    Add(Address, Address, Address),
    Load(Literal, Address),
    Ret(Address),
}

pub struct VirtualMachine {
    reg: [Literal; 256],
}

impl VirtualMachine {
    pub fn run(&mut self, code: Vec<Instruction>) -> Literal {
        for intr in code {
            if let Some(v) = self.run_instruction(intr) {
                return v
            }
        }
        Literal::Nil
    }

    fn run_instruction(&mut self, intr: Instruction) -> Option<Literal> {
        match intr {
            Instruction::Add(addr1, addr2, new_addr) => {
                self.reg[new_addr as usize] = self.reg[addr1 as usize] + self.reg[addr2 as usize];
            }
            Instruction::Load(lit, reg) => {
                self.reg[reg as usize] = lit;
            }
            Instruction::Ret(addr) => {
                return Some(self.reg[addr as usize])
            }
        };
        None
    }
}
