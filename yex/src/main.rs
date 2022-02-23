use rustyline::Editor;
use std::{env::args, fs, process::exit};

fn eval_file(file: &str) {
    let file = match fs::read_to_string(file) {
        Ok(file) => file,
        Err(..) => {
            eprintln!("error reading {}", file);
            exit(1);
        }
    };

    let ast = front::parse(file);

    println!("{:?}", ast);
}

fn start(args: Vec<String>) -> i32 {
    let mut repl = Editor::<()>::new();

    if args.len() > 1 {
        for args in args.iter().skip(1) {
            eval_file(args);
        }
    }

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
