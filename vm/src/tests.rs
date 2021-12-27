use crate::{
    gc::GcRef, list::List, Constant, OpCode, OpCodeMetadata, Symbol, Table, VirtualMachine,
};

macro_rules! vecm {
    ($($tt:tt)*) => {
        &vec![$($tt)*]
            .into_iter()
            .map(metadata_nil)
            .collect::<Vec<_>>()
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

    vm.run(vecm![Push(0), Push(0), Add]).unwrap();
    assert_eq!(vm.pop_last(), &Constant::Num(2.0));
    vm.reset();

    vm.run(vecm![Push(0), Push(0), Sub]).unwrap();
    assert_eq!(vm.pop_last(), &Constant::Num(0.0));
    vm.reset();

    vm.set_consts(vec![Constant::Num(2.0)]);

    vm.run(vecm![Push(0), Push(0), Mul]).unwrap();
    assert_eq!(vm.pop_last(), &Constant::Num(4.0));
    vm.reset();

    vm.run(vecm![Push(0), Push(0), Div]).unwrap();
    assert_eq!(vm.pop_last(), &Constant::Num(1.0));
    vm.reset();
}

#[test]
fn test_list() {
    use Constant::Num;
    let list = List::new();
    assert_eq!(list.head(), None);

    let list = list.prepend(Num(1.0)).prepend(Num(2.0)).prepend(Num(3.0));

    assert_eq!(list.head(), Some(Num(3.0)));

    let list = list.tail();
    assert_eq!(list.head(), Some(Num(2.0)));

    let list = list.tail();
    assert_eq!(list.head(), Some(Num(1.0)));

    let list = list.tail();
    assert_eq!(list.head(), None);

    // Make sure empty tail works
    let list = list.tail();
    assert_eq!(list.head(), None);
}
#[test]
fn table_test() {
    let mut table = Table::new();
    table = table.insert(
        Symbol::new("test"),
        Constant::Table(GcRef::new(Table::new())),
    );
    assert_eq!(
        table.get(&Symbol::new("test")),
        Some(Constant::Table(GcRef::new(Table::new())))
    );
}
#[test]
fn gc_alloc_eq_for_same_values_test() {
    let val1 = GcRef::new(Table::new());
    let val2 = GcRef::new(Table::new());
    assert_eq!(val1, val2);
}
