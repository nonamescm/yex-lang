use crate::{Bytecode, Constant, OpCode, VirtualMachine};

#[test]
fn test_ops() {
    let mut vm = VirtualMachine::default();

    vm.run(Bytecode {
        instructions: vec![OpCode::Push(0), OpCode::Push(1), OpCode::Add],
        constants: vec![Constant::Num(1.0), Constant::Num(1.0)],
    });

    assert_eq!(vm.pop_last(), &Constant::Num(2.0))
}
