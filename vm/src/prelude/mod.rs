use crate::{
    env::EnvTable,
    error::InterpretError,
    gc::GcRef,
    literal::{fun::FnKind, nil, TryGet, Value},
    raise_err, InterpretResult, Symbol, YexModule,
};
use std::io::{self, Write};

fn println(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        Value::Str(s) => println!("{}", **s),
        other => println!("{}", other),
    };

    Ok(nil())
}

fn print(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        Value::Str(s) => print!("{}", **s),
        other => print!("{}", other),
    };

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

    insert_fn!("println", println);
    insert_fn!("print", print);
    insert_fn!("input", input);
    insert_fn!("type", r#type);
    insert_fn!("inspect", inspect);
    insert_fn!("num", num);
    insert_fn!("exit", exit);
    insert_fn!("raise", raise, 2);

    insert!("Nil", Value::Module(GcRef::new(YexModule::nil())));
    insert!("Bool", Value::Module(GcRef::new(YexModule::bool())));
    insert!("Num", Value::Module(GcRef::new(YexModule::num())));
    insert!("Str", Value::Module(GcRef::new(YexModule::str())));
    insert!("List", Value::Module(GcRef::new(YexModule::list())));
    insert!("Sym", Value::Module(GcRef::new(YexModule::sym())));
    insert!("Fn", Value::Module(GcRef::new(YexModule::fun())));
    insert!("Tuple", Value::Module(GcRef::new(YexModule::tuple())));
    insert!("Struct", Value::Module(GcRef::new(YexModule::struct_())));

    prelude
}
