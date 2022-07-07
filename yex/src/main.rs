use rustyline::Editor;
use std::{env::args, fs::{self, File}, process::exit};
use vm::{OpCode, OpCodeMetadata, VirtualMachine};

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

fn patch_bytecode(ops: &mut Vec<OpCodeMetadata>, old_len: usize) {
    for op in ops.iter_mut() {
        if let OpCode::Push(idx) = &mut op.opcode {
            *idx += old_len;
        }
    }
}

fn start(args: Vec<String>) -> i32 {
    let mut repl = Editor::<()>::new();

    let path = format!("{}/.yex_history", std::env::var("HOME").unwrap());
    if repl.load_history(&path).is_err() {
        File::create(&path).ok();
        repl.load_history(&path).ok();
    }

    if args.len() > 1 {
        for args in args.iter().skip(1) {
            eval_file(args);
        }
        return 0;
    }

    let mut vm = VirtualMachine::default();

    loop {
        let line = match repl.readline("yex> ") {
            Ok(str) => str.trim().to_string(),
            Err(_) => {
                repl.save_history(&path).ok();
                return 0;
            }
        };

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        repl.add_history_entry(&line);

        if line.starts_with("def") || line.starts_with("let") || line.starts_with("type") {
            match front::parse(line) {
                Ok((mut bt, ct)) => {
                    patch_bytecode(&mut bt, vm.constants.len());
                    vm.constants.extend(ct);
                    vm.run(&bt).unwrap_or_else(|e| println!("{}", e));
                    println!("{}", vm.pop_last());
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        } else {
            match front::parse_expr(line) {
                Ok((mut bt, ct)) => {
                    patch_bytecode(&mut bt, vm.constants.len());
                    vm.constants.extend(ct);
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
