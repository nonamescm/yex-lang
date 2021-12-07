use front::{compile, compile_expr};
use rustyline::Editor;
use std::{env::args, fs::read_to_string, mem::take, process::exit};
use vm::VirtualMachine;

fn start(mut args: Vec<String>) -> i32 {
    let mut vm = VirtualMachine::default();
    let mut repl = Editor::<()>::new();
    let is_repl = args.len() == 1;

    loop {
        let file = if is_repl {
            match repl.readline("yex> ").map(|it| it.trim().to_string()) {
                Ok(str) => {
                    repl.add_history_entry(&str);
                    str
                },
                Err(_) => return 0,
            }
        } else {
            let file = read_to_string(take(&mut args[1])).unwrap_or_else(|_| {
                eprintln!("file not found");
                exit(1)
            });
            file.trim().to_string()
        };

        let (bytecode, constants) = {
            if file.trim().starts_with("let") {
                compile(file)
            } else {
                compile_expr(file)
            }
            .unwrap_or_else(|e| {
                eprintln!("{}", e);
                (vec![], vec![])
            })
        };

        #[cfg(debug_assertions)]
        {
            println!("bytecode: {:#?}", &bytecode);
            println!("constants: {:#?}", &constants);
        }
        vm.set_consts(constants);
        vm.run(bytecode);

        if !is_repl {
            break;
        } else {
            println!(">> {}", vm.pop_last());
            vm.reset();
        }
    }
    0
}

fn main() {
    exit(start(args().collect()))
}
