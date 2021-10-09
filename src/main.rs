use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::{io, thread};

use glacier_lang::backends::js::codegen::JSCodeGen;
use glacier_lang::backends::CodeGen;
use glacier_lang::glacier_compiler::Compiler;
use glacier_lang::glacier_parser::parse;
use glacier_lang::glacier_vm::value::ValueType;
use glacier_lang::glacier_vm::vm::{Heap, VariableMap, VM};

use log::{error, info, warn};
use simple_logger::SimpleLogger;

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
    SimpleLogger::new().init().unwrap();

    let argv: Vec<String> = args().collect();

    if argv.len() < 2 {
        let mut heap = Heap::default();
        let mut vars = VariableMap::default();
        let mut insts = vec![];

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
                let mut insts_copy = insts.clone();
                let inst = compiler.result.clone();
                insts_copy.extend(inst.clone());

                let mut vm = VM::default();

                vm.heap = heap.clone();
                vm.variables = vars.clone();

                let l = insts.len();
                vm.run_with_start(insts_copy, l, l as isize);
                if let Some(x) = &vm.error {
                    error!("Runtime Error:\n{}", x.to_string(&vm.heap));
                } else if let Some(l) = &vm.last_popped {
                    if l.value_type() != ValueType::Null {
                        println!("{}", l.to_debug_string(&vm.heap));
                    }
                    heap = vm.heap;
                    vars = vm.variables;
                    insts.extend(inst);
                } else {
                    heap = vm.heap;
                    vars = vm.variables;
                    insts.extend(inst);
                }
            } else if let Err(e) = ast {
                error!("Parsing Error:\n{}", e);
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
        if let Some(mode) = argv.get(2) {
            if mode.starts_with("mode=") {
                let opt = mode.split("mode=").collect::<Vec<&str>>()[1].trim();
                match opt {
                    "vm" => {}
                    "js" => {
                        let mut jsgen = JSCodeGen::default();
                        let res = jsgen.generate(&ast, ());
                        println!("{}", res);
                        return;
                    }
                    _ => {
                        warn!("WARNING: invalid mode: {}", opt);
                    }
                }
            }
        }

        let mut compiler = Compiler::new(&code);
        compiler.compile(ast);
        let inst = compiler.result;

        let mut vm = VM::default();

        let mut index = 2;
        loop {
            if let Some(option) = argv.get(index) {
                match option.as_str() {
                    "no_gc" => {
                        info!("INFO: Using NoGC Mode");
                        vm.use_gc = false;
                    }
                    _ => {
                        warn!("WARNING: no such option: {}", option)
                    }
                }
            } else {
                break;
            }
            index += 1;
        }

        vm.run(inst);
        if let Some(x) = &vm.error {
            error!("At Line {}, ", vm.line + 1);
            error!("Runtime Error:\n{}", x.to_string(&vm.heap));
        }
    } else if let Err(e) = ast {
        error!("Parsing Error:\n{}", e);
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
