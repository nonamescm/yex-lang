use crate::{
    env::EnvTable,
    gc::GcRef,
    literal::{nil, Value},
    panic, InterpretResult, List, Symbol, YexType,
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
        panic!("Error flushing stdout")?;
    }

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        panic!("Error reading line")?;
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
    let type_name = match &args[0] {
        Value::List(_) => "list",
        Value::Str(_) => "str",
        Value::Num(_) => "num",
        Value::Bool(_) => "bool",
        Value::Sym(_) => "symbol",
        Value::Nil => "nil",
        Value::Fun { .. } => "fn",
        Value::Type(ty) => ty.name.to_str(),
        Value::Instance(inst) => inst.ty.name.to_str(),
    };

    Ok(Value::Sym(Symbol::new(type_name)))
}

fn inspect(args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(format!("{:#?}", &args[0]))))
}

fn get_os(_args: &[Value]) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(std::env::consts::OS.to_string())))
}

fn num(args: &[Value]) -> InterpretResult<Value> {
    let str = match &args[0] {
        Value::Sym(symbol) => symbol.to_str(),
        Value::Str(str) => &*str,
        n @ Value::Num(..) => return Ok(n.clone()),
        other => panic!("Expected a string or a symbol, found {}", other)?,
    };

    match str.parse::<f64>() {
        Ok(n) => Ok(Value::Num(n)),
        Err(e) => panic!("{:?}", e),
    }
}

fn list(args: &[Value]) -> InterpretResult<Value> {
    match &args[0] {
        xs @ Value::List(_) => Ok(xs.clone()),
        Value::Str(s) => {
            let mut xs = List::new();
            for c in s.chars() {
                xs = xs.prepend(Value::Str(GcRef::new(c.to_string())));
            }
            Ok(Value::List(xs.rev()))
        }
        _ => panic!("Expected a string or a list, found {}", &args[0]),
    }
}

fn exit(args: &[Value]) -> InterpretResult<Value> {
    let code = match &args[0] {
        Value::Num(n) if n.fract() == 0.0 => *n as i32,
        other => panic!("Expected a valid int number, found {}", other)?,
    };

    std::process::exit(code);
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
                Value::Fun(GcRef::new(crate::literal::fun::Fun {
                    arity: $arity,
                    body: GcRef::new($crate::Either::Right(|_, it| $fn(&*it))),
                    args: $crate::StackVec::new(),
                })),
            )
        };

        (@vm $name: expr, $fn: expr, $arity:expr) => {
            prelude.insert(
                $crate::Symbol::new($name),
                Value::Fun(GcRef::new(crate::literal::Fun {
                    arity: $arity,
                    body: GcRef::new($crate::Either::Right(|vm, it| {
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
    insert_fn!("list", list);
    insert_fn!("typeof", r#typeof);
    insert_fn!("inspect", inspect);
    insert_fn!("num", num);
    insert_fn!("exit", exit);
    insert_fn!("getos", get_os, 0);

    insert!("Nil", Value::Type(GcRef::new(YexType::nil())));
    insert!("Bool", Value::Type(GcRef::new(YexType::bool())));
    insert!("Num", Value::Type(GcRef::new(YexType::num())));
    insert!("Str", Value::Type(GcRef::new(YexType::str())));
    insert!("List", Value::Type(GcRef::new(YexType::list())));
    insert!("Sym", Value::Type(GcRef::new(YexType::sym())));
    insert!("Fun", Value::Type(GcRef::new(YexType::fun())));

    prelude
}
