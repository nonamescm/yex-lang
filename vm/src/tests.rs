#[test]
fn test_add() {
    use crate::{Instruction, Literal, VirtualMachine};

    let mut vm = VirtualMachine {
        reg: [Literal::Num(0); 256],
    };

    let test = vm.run(vec![
        Instruction::Load(Literal::Num(137), 40),
        Instruction::Load(Literal::Num(112), 255),
        Instruction::Add(255, 40, 30),
        Instruction::Ret(30),
    ]);

    assert_eq!(test, Literal::Num(137 + 112))
}
