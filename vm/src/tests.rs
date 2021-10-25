use crate::{Bytecode, Constant, OpCode, VirtualMachine};

#[test]
fn test_ops() {
    let mut vm = VirtualMachine::default();

    let res = vm.run(Bytecode {
        instructions: vec![OpCode::Push, OpCode::Push, OpCode::Add, OpCode::Ret]
            .into_iter()
            .map(|c| c as u8)
            .collect(),
        constants: vec![Constant::Num(1.0), Constant::Num(1.0)],
    });

    assert_eq!(res, Constant::Num(2.0))
}
