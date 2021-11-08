use crate::{Constant, OpCode, OpCodeMetadata, VirtualMachine};

macro_rules! vecm {
    ($($tt:tt)*) => {
        vec![$($tt)*]
            .into_iter()
            .map(metadata_nil)
            .collect()
    }
}

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

    vm.set_consts(vec![Constant::Num(1.0)]);

    vm.run(vecm![Push(0), Push(0), Add]);
    assert_eq!(vm.pop_last(), &Constant::Num(2.0));
    vm.reset();

    vm.run(vecm![Push(0), Push(0), Sub]);
    assert_eq!(vm.pop_last(), &Constant::Num(0.0));
    vm.reset();

    vm.set_consts(vec![Constant::Num(2.0)]);

    vm.run(vecm![Push(0), Push(0), Mul]);
    assert_eq!(vm.pop_last(), &Constant::Num(4.0));
    vm.reset();

    vm.run(vecm![Push(0), Push(0), Div]);
    assert_eq!(vm.pop_last(), &Constant::Num(1.0));
    vm.reset();
}
