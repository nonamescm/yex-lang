use crate::{
    env::EnvTable,
    error::InterpretError,
    gc::GcRef,
    literal::{fun::FnKind, nil, str::methods::format_value, TryGet, Value},
    raise_err, InterpretResult, Symbol, VirtualMachine, YexType,
};
use std::io::{Write, self};

fn println(vm: &mut VirtualMachine, args: &[Value]) -> InterpretResult<Value> {
    let message = format_value(vm, args[0].clone())?;
    println!("{}", message);

    Ok(nil())
}

fn print(vm: &mut VirtualMachine, args: &[Value]) -> InterpretResult<Value> {
    let message = format_value(vm, args[0].clone())?;
    print!("{}", message);

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

fn r#typeof(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Type(args[0].type_of()))
}

fn inspect(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(format!("{:#?}", &args[0]))))
}

fn num(args: &[Value]) -> InterpretResult<Value> {
    let str: String = args[0].get()?;

    str.parse::<f64>()
        .map(Value::Num)
        .map_err(|_| raise_err!(TypeError))
}

fn exit(args: &[Value]) -> InterpretResult<Value> {
    let code: isize = args[0].get()?;

    std::process::exit(code as i32);
}

fn raise(args: &[Value]) -> InterpretResult<Value> {
    let msg: Symbol = args[0].get()?;

    Err(InterpretError {
        err: msg,
        line: unsafe { crate::LINE },
        column: unsafe { crate::COLUMN },
    })
}

pub fn prelude() -> EnvTable {
    let mut prelude = EnvTable::with_capacity(64);
    macro_rules! insert_fn {
        ($name: expr, $fn: expr) => {
            insert_fn!($name, $fn, 1)
        };
        ($name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Value::Fn(GcRef::new(crate::literal::fun::Fn {
                    arity: $arity,
                    body: GcRef::new(FnKind::Native(|_, it| $fn(&*it))),
                    args: $crate::StackVec::new(),
                })),
            )
        };

        (@vm $name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Value::Fn(GcRef::new(crate::literal::fun::Fn {
                    arity: $arity,
                    body: GcRef::new(FnKind::Native(|vm, it| {
                        $fn(unsafe { vm.as_mut().unwrap() }, &*it)
                    })),
                    args: $crate::StackVec::new(),
                })),
            )
        };
    }

    macro_rules! insert {
        ($name: expr, $value: expr) => {
            prelude.insert($crate::Symbol::new($name), $value)
        };
    }

    insert_fn!(@vm "println", println, 1);
    insert_fn!(@vm "print", print, 1);
    insert_fn!("input", input);
    insert_fn!("typeof", r#typeof);
    insert_fn!("inspect", inspect);
    insert_fn!("num", num);
    insert_fn!("exit", exit);
    insert_fn!("raise", raise);

    insert!("Nil", Value::Type(GcRef::new(YexType::nil())));
    insert!("Bool", Value::Type(GcRef::new(YexType::bool())));
    insert!("Num", Value::Type(GcRef::new(YexType::num())));
    insert!("Str", Value::Type(GcRef::new(YexType::str())));
    insert!("List", Value::Type(GcRef::new(YexType::list())));
    insert!("Sym", Value::Type(GcRef::new(YexType::sym())));
    insert!("Fn", Value::Type(GcRef::new(YexType::fun())));
    insert!("File", Value::Type(GcRef::new(YexType::file())));

    insert!("Table", Value::Type(GcRef::new(YexType::table())));
    prelude
}
