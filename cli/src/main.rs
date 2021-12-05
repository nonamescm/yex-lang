use front::compile;
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
        #[cfg(debug_assertions)]
        {
            println!("{:#?}", &bytecode);
            println!("{:#?}", &constants);
        }
        vm.set_consts(constants);
        vm.run(bytecode);
        0
    } else {
        println!("Yex language interpreter");
        1
    }
}

fn main() {
    use std::{env::args, process::exit};

    exit(start(args().collect()))
}
