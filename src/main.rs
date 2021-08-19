use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::{io, thread};

use glacier_lang::glacier_compiler::Compiler;
use glacier_lang::glacier_parser::parse;
use glacier_lang::glacier_vm::value::ValueType;
use glacier_lang::glacier_vm::vm::{Heap, VariableMap, VM};

fn get_input() -> String {
    let mut input = String::new();
    print!("> ");
    io::stdout().flush().expect("flush failed!");
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    input.trim_end().to_string()
}

fn cli() {
    let argv: Vec<String> = args().collect();

    if argv.len() < 2 {
        let mut heap = Heap::default();
        let mut vars = VariableMap::default();

        println!("Welcome to Glacier repl. Type :quit to quit.",);

        loop {
            let ip = get_input();
            if ip.trim() == ":quit" {
                break;
            }

            let ast = parse(&ip);
            if let Ok(ast) = ast {
                let mut compiler = Compiler::new(&ip);
                compiler.compile(ast);
                let inst = compiler.result.clone();

                // dbg!(&inst);

                let mut vm = VM::default();

                vm.heap = heap.clone();
                vm.variables = vars.clone();

                vm.run(inst);
                if let Some(x) = &vm.error {
                    eprintln!("Runtime Error:\n{}", x.to_string(&vm.heap));
                } else if let Some(l) = &vm.last_popped {
                    if l.value_type() != ValueType::Null {
                        println!("{}", l.to_debug_string(&vm.heap));
                    }
                    heap = vm.heap;
                    vars = vm.variables;
                } else {
                    heap = vm.heap;
                    vars = vm.variables;
                }
            } else if let Err(e) = ast {
                eprintln!("Parsing Error:\n{}", e);
            }
        }
        return;
    }

    let filename = &argv[1];
    let mut file = File::open(filename).expect("Unable to open the file");
    let mut contents = vec![];
    file.read_to_end(&mut contents)
        .expect("Unable to read the file");

    let code = String::from_utf8(contents).unwrap();

    let ast = parse(&*code);
    if let Ok(ast) = ast {
        let mut compiler = Compiler::new(&code);
        compiler.compile(ast);
        let inst = compiler.result;

        let mut vm = VM::default();

        let mut index = 2;
        loop {
            if let Some(option) = argv.get(index) {
                match option.as_str() {
                    "use_ref" => {
                        println!("INFO: Using Reference Mode");
                        eprintln!("WARNING: There will be bugs if you use reference.");
                        vm.use_reference = true;
                    }
                    "no_gc" => {
                        println!("INFO: Using NoGC Mode");
                        vm.use_gc = false;
                    }
                    _ => {
                        eprintln!("WARNING: no such option: {}", option)
                    }
                }
            } else {
                break;
            }
            index += 1;
        }

        vm.run(inst);
        if let Some(x) = &vm.error {
            eprintln!("At Line {}, ", vm.line + 1);
            eprintln!("Runtime Error:\n{}", x.to_string(&vm.heap));
        }
    } else if let Err(e) = ast {
        eprintln!("Parsing Error:\n{}", e);
    }
}

static STACK_SIZE: usize = 1 << 23;

fn main() {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .name(format!("Glacier Programming Language"))
        .spawn(cli)
        .unwrap();

    child.join().unwrap();
}
