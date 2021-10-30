use front::compile;
use rustyline::{error::ReadlineError, Editor};
use vm::VirtualMachine;

fn start(_args: Vec<String>) -> i32 {
    let mut vm = VirtualMachine::default();
    let mut repl = Editor::<()>::new();
    repl.load_history("/tmp/.yex_history").ok();

    loop {
        match repl.readline("yex> ") {
            Ok(l) => {
                repl.add_history_entry(l.as_str());
                if l.trim() == "" {
                    continue;
                }
                match compile(l) {
                    Ok(bytecode) => {
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "instructions: {:?}\nconstants: {:?}",
                            bytecode.instructions, bytecode.constants
                        );
                        vm.run(bytecode);

                        println!("{}", vm.pop_last())
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
            }
        }
        vm.reset();
    }
}

fn main() {
    use std::{env::args, process::exit};

    exit(start(args().collect()))
}
