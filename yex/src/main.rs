use rustyline::Editor;
use std::{env::args, process::exit};

fn start(_: Vec<String>) -> i32 {
    let mut repl = Editor::<()>::new();

    loop {
        let line = match repl.readline("yex> ").map(|it| it.trim().to_string()) {
            Ok(str) => {
                repl.add_history_entry(&str);
                str
            }
            Err(_) => return 0,
        };
        if line.is_empty() {
            continue;
        }

        if line.starts_with("def") {
            match front::parse(line) {
                Ok(ast) => {
                    eprintln!("{:#?}", ast);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        } else {
            match front::parse_expr(line) {
                Ok(ast) => {
                    eprintln!("{:#?}", ast);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
    }
}

fn main() {
    let args = args().collect();
    exit(start(args));
}
