use crate::{
    env::EnvTable,
    gc::GcRef,
    literal::{nil, Value, TryGet, fun::FnKind},
    raise, InterpretResult, YexType,
};
use std::io::Write;

fn println(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        Value::Str(s) => println!("{}", &**s),
        other => println!("{}", other),
    };
    Ok(nil())
}

fn print(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        Value::Str(s) => print!("{}", &**s),
        other => print!("{}", other),
    };
    Ok(nil())
}

fn input(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        Value::Str(s) => print!("{}", **s),
        other => print!("{}", other),
    };

    if std::io::stdout().flush().is_err() {
        raise!("Error flushing stdout")?;
    }

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        raise!("Error reading line")?;
    }

    input.pop();
    Ok(Value::Str(GcRef::new(input)))
}

fn str(args: &[Value]) -> InterpretResult<Value> {
    if let Value::Str(s) = &args[0] {
        Ok(Value::Str(s.clone()))
    } else {
        Ok(Value::Str(GcRef::new(format!("{}", &args[0]))))
    }
}

fn r#typeof(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Type(args[0].type_of()))
}

fn inspect(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(format!("{:#?}", &args[0]))))
}

fn num(args: &[Value]) -> InterpretResult<Value> {
    let str = match &args[0] {
        Value::Sym(symbol) => symbol.as_str(),
        Value::Str(str) => &*str,
        n @ Value::Num(..) => return Ok(n.clone()),
        other => raise!("Expected a string or a symbol, found {}", other)?,
    };

    match str.parse::<f64>() {
        Ok(n) => Ok(Value::Num(n)),
        Err(e) => raise!("{:?}", e),
    }
}

fn exit(args: &[Value]) -> InterpretResult<Value> {
    let code: f64 = args[0].get()?;
    if code.fract() != 0.0 {
        raise!("Expected an integer, found {}", code)?;
    }
    std::process::exit(code as i32);
}

fn raise(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        Value::Str(s) => raise!("{}", &**s),
        other => raise!("{}", other),
    }
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
                Value::Fn(GcRef::new(crate::literal::Fn {
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
    insert_fn!("str", str);
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

    insert!("Table", Value::Type(GcRef::new(YexType::table())));
    prelude
}
