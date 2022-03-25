use rustyline::Editor;
use std::{env::args, fs, process::exit};
use vm::VirtualMachine;

fn eval_file(file: &str) {
    let file = match fs::read_to_string(file) {
        Ok(file) => file,
        Err(..) => {
            eprintln!("error reading {}", file);
            exit(1);
        }
    };

    let (bt, ct) = match front::parse(file) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    let mut vm = VirtualMachine::default();


    vm.set_consts(ct);
    if let Err(e) = vm.run(&bt) {
        eprintln!("{}", e);
        exit(1);
    }

}

fn start(args: Vec<String>) -> i32 {
    let mut repl = Editor::<()>::new();

    if args.len() > 1 {
        for args in args.iter().skip(1) {
            eval_file(args);
        }
        return 0;
    }

    let mut vm = VirtualMachine::default();

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
                Ok((bt, ct)) => {
                    vm.set_consts(ct);
                    vm.run(&bt).unwrap_or_else(|e| println!("{}", e));
                    println!("{}", vm.pop_last());
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        } else {
            match front::parse_expr(line) {
                Ok((bt, ct)) => {
                    vm.set_consts(ct);
                    vm.run(&bt).unwrap_or_else(|e| println!("{}", e));
                    println!("{}", vm.pop_last());
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        vm.reset();
    }
}

fn main() {
    let args = args().collect();
    exit(start(args));
}
