#[test]
fn test_add() {
    use crate::{Instruction, Literal, VirtualMachine};

    let mut vm = VirtualMachine {
        reg: [Literal::Num(0.0); 256],
    };

    let test = vm.run(vec![
        Instruction::Load(Literal::Num(112.0), 40),
        Instruction::Load(Literal::Num(137.0), 255),
        Instruction::Add(40, 255, 30),
        Instruction::Ret(30),
    ]);

    assert_eq!(test, Literal::Num(112.0 + 137.0))
}

#[test]
fn test_sub() {
    use crate::{Instruction, Literal, VirtualMachine};

    let mut vm = VirtualMachine {
        reg: [Literal::Num(0.0); 256],
    };

    let test = vm.run(vec![
        Instruction::Load(Literal::Num(112.0), 40),
        Instruction::Load(Literal::Num(137.0), 255),
        Instruction::Sub(40, 255, 30),
        Instruction::Ret(30),
    ]);

    assert_eq!(test, Literal::Num(112.0 - 137.0))
}

#[test]
fn test_mul() {
    use crate::{Instruction, Literal, VirtualMachine};

    let mut vm = VirtualMachine {
        reg: [Literal::Num(0.0); 256],
    };

    let test = vm.run(vec![
        Instruction::Load(Literal::Num(112.0), 40),
        Instruction::Load(Literal::Num(137.0), 255),
        Instruction::Mul(40, 255, 30),
        Instruction::Ret(30),
    ]);

    assert_eq!(test, Literal::Num(112.0 * 137.0))
}

#[test]
fn test_div() {
    use crate::{Instruction, Literal, VirtualMachine};

    let mut vm = VirtualMachine {
        reg: [Literal::Num(0.0); 256],
    };

    let test = vm.run(vec![
        Instruction::Load(Literal::Num(112.0), 40),
        Instruction::Load(Literal::Num(137.0), 255),
        Instruction::Div(40, 255, 30),
        Instruction::Ret(30),
    ]);

    assert_eq!(test, Literal::Num(112.0 / 137.0))
}
