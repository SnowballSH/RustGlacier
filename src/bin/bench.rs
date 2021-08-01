use std::time::Instant;

use glacier_lang::glacier_compiler::Compiler;
use glacier_lang::glacier_parser::parse;
use glacier_lang::glacier_vm::value::Value;
use glacier_lang::glacier_vm::vm::VM;

fn main() {
    let benchmark_code = [
        (
            r#"
a = 4
a + 5
"#,
            Value::Int(9),
        ),
        (
            r#"
a = -5
b = 30
(a + b) / 5
"#,
            Value::Int(5),
        ),
        (
            r#"
condition = !true
if !condition
    9 + condition
else
    0
"#,
            Value::Int(9),
        ),
        (
            r#"
if false
    8
end
"#,
            Value::Null,
        ),
        (
            r#"
"Hello, " + "world!"
"#,
            Value::String(format!("Hello, world!")),
        ),
        (
            r#"
i = 5
res = 1
while i
    res = res * i
    i = i - 1
end
res
"#,
            Value::Int(120),
        ),
        (
            r#"
i = 100
res = 1
while i
    res = res + res % i
    i = i - 1
end
res
"#,
            Value::Int(2304),
        ),
    ];

    let mut i = 0;
    let start = Instant::now();
    for (code, result) in benchmark_code {
        let start1 = Instant::now();
        let mut ok = true;
        println!("BENCH #{}", i);
        let ast = parse(code);
        if let Ok(ast) = ast {
            let mut compiler = Compiler::new(code);
            compiler.compile(ast);
            let inst = compiler.result.clone();

            let mut vm = VM::default();

            vm.run(inst);
            if let Some(x) = &vm.error {
                eprintln!("Runtime Error: {}", x.to_string());
                ok = false;
            } else if vm.last_popped != Some(result.clone()) {
                eprintln!(
                    "Assert Error: Expected {}, got {:?}",
                    result.to_debug_string(),
                    vm.last_popped.and_then(|x| Some(x.to_debug_string()))
                );
                ok = false;
            }
        } else if let Err(e) = ast {
            eprintln!("Parsing Error: {}", e);
            ok = false;
        }
        println!("{:?} {}", start1.elapsed(), if ok { "OK" } else { "NOTOK" });
        i += 1;
    }
    println!("{:?}", start.elapsed());
}
