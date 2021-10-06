use front::compile;
use rustyline::{error::ReadlineError, Editor};
use std::env;
use vm::VirtualMachine;

fn tempfile() -> Option<String> {
    let mut dir = env::temp_dir();
    dir.push(".yex_history");
    Some(dir.to_str()?.to_string())
}

fn _main(_args: Vec<String>) -> i32 {
    let mut vm = VirtualMachine::default();
    let mut repl = Editor::<()>::new();
    let history = tempfile();
    if let Some(f) = history {
        repl.load_history(&f).ok();
    }

    loop {
        match repl.readline("yex> ") {
            Ok(l) => {
                repl.add_history_entry(l.as_str());
                match compile(l) {
                    Ok(mut vec) => {
                        vec.push(vm::Instruction::Ret);
                        println!("{}", vm.run(vec))
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
                println!("Error: {}", e);
            }
        }
    }
}

fn main() {
    use std::{env::args, process::exit};

    exit(_main(args().collect()))
}
