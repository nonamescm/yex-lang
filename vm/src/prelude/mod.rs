use crate::{
    env::EnvTable,
    error::InterpretError,
    gc::GcRef,
    literal::{nil, show, TryGet, Value},
    raise_err, InterpretResult, Symbol, VirtualMachine, YexModule,
};
use std::io::{self, Write};

#[macro_export]
/// Insert a function into a `EnvTable`
macro_rules! insert_fn {
    ($table:ident, $name: expr, $fn: expr) => {
        insert_fn!($table, $name, $fn, 1)
    };

    ($table:ident, $name: expr, $fn: expr, $arity:expr) => {
        $table.insert(
            $crate::Symbol::new($name),
            $crate::literal::Value::Fn($crate::gc::GcRef::new($crate::literal::fun::Fn {
                arity: $arity,
                body: $crate::gc::GcRef::new($crate::literal::fun::FnKind::Native(|_, it| {
                    $fn(&*it)
                })),
                args: $crate::StackVec::new(),
            })),
        )
    };

    (:vm $table:ident, $name: expr, $fn: expr, $arity:expr) => {
        $table.insert(
            $crate::Symbol::new($name),
            $crate::literal::Value::Fn($crate::gc::GcRef::new($crate::literal::fun::Fn {
                arity: $arity,
                body: $crate::gc::GcRef::new($crate::literal::fun::FnKind::Native(|vm, it| {
                    $fn(unsafe { vm.as_mut().unwrap() }, &*it)
                })),
                args: $crate::StackVec::new(),
            })),
        )
    };
}
#[macro_export]
/// Insert a thing to `EnvTable`
macro_rules! insert {
    ($table:ident, $name: expr, $value: expr) => {
        $table.insert($crate::Symbol::new($name), $value)
    };
}

fn println(vm: &mut VirtualMachine, args: &[Value]) -> InterpretResult<Value> {
    println!("{}", show(vm, args.into())?);
    Ok(nil())
}

fn print(vm: &mut VirtualMachine, args: &[Value]) -> InterpretResult<Value> {
    print!("{}", show(vm, args.into())?);
    Ok(nil())
}

fn debug_stack(vm: &mut VirtualMachine, _args: &[Value]) -> InterpretResult<Value> {
    println!("{:#?}", vm.stack);
    Ok(nil())
}

fn input(args: &[Value]) -> InterpretResult<Value> {
    let prompt: String = args[0].get()?;
    print!("{}", prompt);

    io::stdout().flush()?;

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    input.pop();

    Ok(Value::Str(GcRef::new(input)))
}

fn r#type(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Module(args[0].type_of()))
}

fn inspect(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(format!("{:#?}", &args[0]))))
}

fn num(args: &[Value]) -> InterpretResult<Value> {
    let str: String = args[0].get()?;

    str.parse::<f64>()
        .map(Value::Num)
        .map_err(|_| raise_err!(TypeError, "Cannot convert '{}' to number", str))
}

fn exit(args: &[Value]) -> InterpretResult<Value> {
    let code: isize = args[0].get()?;

    std::process::exit(code as i32);
}

fn raise(args: &[Value]) -> InterpretResult<Value> {
    let err: Symbol = args[0].get()?;
    let msg: String = args[1].get()?;

    Err(InterpretError {
        err,
        msg,
        line: unsafe { crate::LINE },
        column: unsafe { crate::COLUMN },
    })
}

pub fn prelude() -> EnvTable {
    let mut prelude = EnvTable::with_capacity(64);
    insert_fn!(:vm prelude, "println", println, 1);
    insert_fn!(:vm prelude, "print", print, 1);
    insert_fn!(:vm prelude, "print_stack!", debug_stack, 1);
    insert_fn!(prelude, "input", input);
    insert_fn!(prelude, "type", r#type);
    insert_fn!(prelude, "inspect", inspect);
    insert_fn!(prelude, "num", num);
    insert_fn!(prelude, "exit", exit);
    insert_fn!(prelude, "raise", raise, 2);

    insert!(prelude, "Nil", Value::Module(GcRef::new(YexModule::nil())));
    insert!(
        prelude,
        "Bool",
        Value::Module(GcRef::new(YexModule::bool()))
    );
    insert!(prelude, "Num", Value::Module(GcRef::new(YexModule::num())));
    insert!(prelude, "Str", Value::Module(GcRef::new(YexModule::str())));
    insert!(
        prelude,
        "List",
        Value::Module(GcRef::new(YexModule::list()))
    );
    insert!(prelude, "Sym", Value::Module(GcRef::new(YexModule::sym())));
    insert!(prelude, "Fn", Value::Module(GcRef::new(YexModule::fun())));
    insert!(
        prelude,
        "Tuple",
        Value::Module(GcRef::new(YexModule::tuple()))
    );
    insert!(
        prelude,
        "Result",
        Value::Module(GcRef::new(YexModule::result()))
    );
    insert!(prelude, "FFI", Value::Module(GcRef::new(YexModule::ffi())));

    prelude
}
