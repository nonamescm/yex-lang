use crate::{
    error::InterpretResult,
    gc::GcRef,
    literal::{nil, TryGet},
    raise, List, Symbol, Value, VirtualMachine,
};

pub fn get(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[0].get()?;
    let index: f64 = args[1].get()?;

    if index < 0.0 || index.fract() != 0.0 {
        raise!(ValueError)?;
    }

    let char = string
        .chars()
        .nth(index as usize)
        .map(|c| c.to_string().into())
        .unwrap_or_else(nil);
    Ok(char)
}

pub fn split(_: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let string: String = args[0].get()?;
    let separator: String = args[1].get()?;

    let list: List = string
        .split(&separator)
        .map(|str| str.to_owned().into())
        .collect();

    Ok(list.rev().into())
}

fn format_value(vm: &mut VirtualMachine, value: Value) -> InterpretResult<String> {
    let show: Symbol = "show".into();

    let str = match value {
        Value::Str(s) => s.to_string(),
        ref val @ Value::Instance(ref i) if i.ty.fields.get(&show).is_some() => {
            vm.push(val.clone());
            vm.push(i.ty.fields.get(&show).unwrap());

            vm.call(1)?;

            let res = vm.pop();

            format_value(vm, res)?
        }
        other => format!("{}", other),
    };

    Ok(str)
}

pub fn fmt(vm: *mut VirtualMachine, args: Vec<Value>) -> InterpretResult<Value> {
    let vm = unsafe { &mut *vm };

    let format: String = args[0].get()?;
    let args: List = args[1].get()?;
    let mut idx = 0;

    let res = format
        .chars()
        .map(|it| {
            if it == '&' {
                format_value(vm, args.index(idx)).map(|str| {
                    idx += 1;
                    str
                })
            } else {
                Ok(it.to_string())
            }
        })
        .collect::<Result<String, _>>()?;

    Ok(res.into())
}

pub fn init(_: *mut VirtualMachine, _: Vec<Value>) -> InterpretResult<Value> {
    Ok(Value::Str(GcRef::new(String::from(""))))
}
