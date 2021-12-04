use front::compile;
use rustyline::{error::ReadlineError, Editor};
use std::fs;
use vm::VirtualMachine;

fn start(mut args: Vec<String>) -> i32 {
    let mut vm = VirtualMachine::default();

    if args.len() > 1 {
        let file = fs::read_to_string(std::mem::take(&mut args[1])).unwrap();
        let (bytecode, constants) = compile(file).unwrap_or_else(|e| {
            println!("{}", e);
            (vec![], vec![])
        });
        vm.set_consts(constants);
        vm.run(bytecode);
        return 0;
    }

    let mut repl = Editor::<()>::new();
    repl.load_history("/tmp/.yex_history").ok();

    loop {
        match repl.readline("yex> ") {
            Ok(l) => {
                let mut l = l.trim().to_string();
                repl.add_history_entry(&l);
                if l == "" {
                    continue;
                }
                if !l.starts_with("let") {
                    l = String::from("let __last__ = puts(") + &l;
                    l.push(')');
                }
                match compile(l) {
                    Ok((bytecode, constants)) => {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "instructions: {:#?}\nconstants: {:#?}\n",
                            bytecode.iter().map(|c| c.opcode).collect::<Vec<_>>(),
                            constants
                        );
                        vm.set_consts(constants);
                        vm.run(bytecode);
                        vm.debug_stack();

                        if !vm.pop_last().is_nil() {
                            println!("=> {}", vm.pop_last())
                        }
                    }
                    Err(e) => println!("{}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }

            Err(ReadlineError::Eof) => {
                break 0;
            }

            Err(e) => {
                println!("error reading line");

                #[cfg(debug_assertions)]
                eprintln!("Error: {}", e);

                drop(e);
            }
        }
        vm.reset();
    }
}

fn main() {
    use std::{env::args, process::exit};

    exit(start(args().collect()))
}
