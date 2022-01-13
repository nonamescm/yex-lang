use crate::{panic, Constant, GcRef, InterpretResult};

pub fn str_split(args: &[Constant]) -> InterpretResult<Constant> {
    use Constant::*;

    let str = match &args[0] {
        Str(str) => str.get(),
        other => return panic!("split() expected str, found {}", other),
    };

    let pat = match &args[1] {
        Str(pat) => pat.get(),
        other => return panic!("split() expected str, found {}", other),
    };

    let mut list = crate::List::new();
    for i in str.rsplit(pat) {
        list = list.prepend(Str(GcRef::new(i.to_string())));
    }

    Ok(List(GcRef::new(list)))
}

pub fn starts_with(args: &[Constant]) -> InterpretResult<Constant> {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match &args[0] {
        Str(string) => string,
        other => return panic!("starts_with()[0] expected str, found {}", other),
    };
    let pattern = match &args[1] {
        Str(pat) => pat.get(),
        other => return panic!("starts_with()[1] expected str, found {}", other),
    };

    Ok(Bool(str.starts_with(pattern)))
}

pub fn ends_with(args: &[Constant]) -> InterpretResult<Constant> {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match &args[0] {
        Str(string) => string,
        other => return panic!("ends_with() expected str, found {}", other),
    };
    let pattern = match &args[1] {
        Str(pat) => pat.get(),
        other => return panic!("ends_with() expected str, found {}", other),
    };
    Ok(Bool(str.ends_with(pattern)))
}

pub fn replace(args: &[Constant]) -> InterpretResult<Constant> {
    let str = match &args[0] {
        Constant::Str(str) => str.get(),
        other => return panic!("replace()[0] expected a str, but found `{}`", other),
    };

    let s_match = match &args[1] {
        Constant::Str(str) => str.get(),
        other => return panic!("replace()[1] expected a str, but found `{}`", other),
    };

    let s_match2 = match &args[2] {
        Constant::Str(str) => str.get(),
        other => return panic!("replace()[2] expected a str, but found `{}`", other),
    };

    let str = str.replace(s_match, s_match2);

    Ok(Constant::Str(GcRef::new(str)))
}
