use crate::{Bytecode, Constant, OpCode, OpCodeMetadata, VirtualMachine};

fn metadata_nil(op: OpCode) -> OpCodeMetadata {
    OpCodeMetadata {
        line: 0,
        column: 0,
        opcode: op,
    }
}

#[test]
fn test_ops() {
    use OpCode::*;

    let mut vm = VirtualMachine::default();

    vm.run(Bytecode {
        instructions: vec![Push(0), Push(0), Add]
            .into_iter()
            .map(metadata_nil)
            .collect(),
        constants: vec![Constant::Num(1.0)],
    });
    assert_eq!(vm.pop_last(), &Constant::Num(2.0));

    vm.reset();

    vm.run(Bytecode {
        instructions: vec![Push(0), Push(0), Sub]
            .into_iter()
            .map(metadata_nil)
            .collect(),
        constants: vec![Constant::Num(1.0)],
    });
    assert_eq!(vm.pop_last(), &Constant::Num(0.0));

    vm.reset();

    vm.run(Bytecode {
        instructions: vec![Push(0), Push(0), Mul]
            .into_iter()
            .map(metadata_nil)
            .collect(),
        constants: vec![Constant::Num(2.0)],
    });
    assert_eq!(vm.pop_last(), &Constant::Num(4.0));

    vm.reset();

    vm.run(Bytecode {
        instructions: vec![Push(0), Push(0), Div]
            .into_iter()
            .map(metadata_nil)
            .collect(),
        constants: vec![Constant::Num(2.0)],
    });
    assert_eq!(vm.pop_last(), &Constant::Num(1.0));

    vm.reset();
}
