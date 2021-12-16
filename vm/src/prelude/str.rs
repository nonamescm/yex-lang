use crate::{err_tuple, Constant, GcRef};

pub fn str_split(args: &[Constant]) -> Constant {
    use Constant::*;

    let str = match args[0].get() {
        Str(str) => str,
        other => err_tuple!("split() expected str, found {}", other),
    };

    let pat = match args[1].get() {
        Str(pat) => pat,
        other => err_tuple!("split() expected str, found {}", other),
    };

    let mut list = crate::List::new();
    for i in str.rsplit(pat) {
        list = list.prepend(GcRef::new(Str(i.to_string())));
    }

    GcRef::new(List(list))
}

pub fn starts_with(args: &[Constant]) -> Constant {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match args[0].get() {
        Str(string) => string,
        other => err_tuple!("starts_with()[0] expected str, found {}", other),
    };
    let pattern = match args[1].get() {
        Str(pat) => pat,
        other => err_tuple!("starts_with()[1] expected str, found {}", other),
    };
    GcRef::new(Bool(str.starts_with(pattern)))
}

pub fn ends_with(args: &[Constant]) -> Constant {
    /*
    args:
        number | name    | type
        0      | str     |(do i need to say?)
        1      | pattern | str
     */
    use Constant::*;

    let str = match args[0].get() {
        Str(string) => string,
        other => err_tuple!("ends_with() expected str, found {}", other),
    };
    let pattern = match args[1].get() {
        Str(pat) => pat,
        other => err_tuple!("ends_with() expected str, found {}", other),
    };
    GcRef::new(Bool(str.ends_with(pattern)))
}

pub fn replace(args: &[Constant]) -> Constant {
    let str = match args[0].get() {
        Constant::Str(str) => str,
        other => err_tuple!("replace()[0] expected a str, but found `{}`", other),
    };
    let s_match = match args[1].get() {
        Constant::Str(str) => str,
        other => err_tuple!("replace()[1] expected a str, but found `{}`", other),
    };

    let s_match2 = match args[2].get() {
        Constant::Str(str) => str,
        other => err_tuple!("replace()[2] expected a str, but found `{}`", other),
    };

    GcRef::new(Constant::Str(str.replace(s_match, s_match2)))
}
