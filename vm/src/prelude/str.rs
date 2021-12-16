use crate::{err_tuple, Constant, GcRef};

pub fn str_split(args: &[Constant]) -> Constant {
    use Constant::*;

    let str = match &args[0] {
        Str(str) => str.get(),
        other => err_tuple!("split() expected str, found {}", other),
    };

    let pat = match &args[1] {
        Str(pat) => pat.get(),
        other => err_tuple!("split() expected str, found {}", other),
    };

    let mut list = crate::List::new();
    for i in str.rsplit(pat) {
        list = list.prepend(Str(GcRef::new(i.to_string())));
    }

    List(GcRef::new(list))
}

pub fn starts_with(args: &[Constant]) -> Constant {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match &args[0] {
        Str(string) => string,
        other => err_tuple!("starts_with()[0] expected str, found {}", other),
    };
    let pattern = match &args[1] {
        Str(pat) => pat.get(),
        other => err_tuple!("starts_with()[1] expected str, found {}", other),
    };

    Bool(str.starts_with(pattern))
}

pub fn ends_with(args: &[Constant]) -> Constant {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match &args[0] {
        Str(string) => string,
        other => err_tuple!("ends_with() expected str, found {}", other),
    };
    let pattern = match &args[1] {
        Str(pat) => pat.get(),
        other => err_tuple!("ends_with() expected str, found {}", other),
    };
    Bool(str.ends_with(pattern))
}

pub fn replace(args: &[Constant]) -> Constant {
    let str = match &args[0] {
        Constant::Str(str) => str.get(),
        other => err_tuple!("replace()[0] expected a str, but found `{}`", other),
    };

    let s_match = match &args[1] {
        Constant::Str(str) => str.get(),
        other => err_tuple!("replace()[1] expected a str, but found `{}`", other),
    };

    let s_match2 = match &args[2] {
        Constant::Str(str) => str.get(),
        other => err_tuple!("replace()[2] expected a str, but found `{}`", other),
    };

    let str = str.replace(s_match, &s_match2);

    Constant::Str(GcRef::new(str))
}
