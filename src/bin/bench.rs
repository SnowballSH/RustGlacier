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
else:
    0
"#,
            Value::Int(9),
        ),
        (
            r#"
if false:
    8
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
        (
            r#"
# 9742460
s = "974" + 246.s + false.i.s
s.i
"#,
            Value::Int(9742460),
        ),
        (
            r#"
fn add_3(a, b, c)
    a + b + c
end

add_3(65, 15, 10)
"#,
            Value::Int(90),
        ),
        (
            r#"
fn F(i)
    if i <= 1
        i
    else
        F(i - 1) + F(i - 2)
    end
end
F(25)
"#,
            Value::Int(75025),
        ),
        (
            r#"
fn A(i)
    if i > 0 A(i - 1)
    else i
    end
end
A(1000)
"#,
            Value::Int(0),
        ),
        (
            r#"
fn K(i)
    i = i + 5
    if i > 10:
        ret i
    i = i + 5
    if i > 10:
        ret i
end
K(2)
"#,
            Value::Int(12),
        ),
        (
            r#"
fn K(i)
    i = i + 5
    if i > 10:
        ret i
    i = i + 5
    if i > 10:
        ret i
end
K(-20)
"#,
            Value::Null,
        ),
    ];

    let mut i = 0;
    let start = Instant::now();
    for (code, result) in benchmark_code {
        let mut vm = VM::default();

        let start1 = Instant::now();
        let mut ok = true;
        println!("BENCH #{}", i);
        let ast = parse(code);
        let parse_time = start1.elapsed();
        println!("PARSING TIME: {:?}", parse_time);
        let parse_time = start1.elapsed();

        if let Ok(ast) = ast {
            let mut compiler = Compiler::new(code);
            compiler.compile(ast);
            let compile_time = start1.elapsed() - parse_time;
            println!("COMPILATION TIME: {:?}", compile_time);
            let compile_time = start1.elapsed() - parse_time;

            vm.run(compiler.result);

            let vm_time = start1.elapsed() - parse_time - compile_time;
            println!("VM TIME: {:?}", vm_time);

            if let Some(x) = &vm.error {
                eprintln!("Runtime Error: {}", x.to_string(&vm.heap));
                eprintln!("LINE {}", vm.line);
                ok = false;
            } else if vm.last_popped != Some(result.clone()) {
                eprintln!(
                    "Assert Error: Expected {}, got {:?}",
                    result.to_debug_string(&vm.heap),
                    if let Some(x) = vm.last_popped {
                        Some(x.to_debug_string(&vm.heap))
                    } else {
                        None
                    }
                );
                ok = false;
            }
        } else if let Err(e) = ast {
            eprintln!("Parsing Error: {}", e);
            ok = false;
        }
        println!(
            "BENCH TIME: {:?}\nRESULT: {}\n",
            start1.elapsed(),
            if ok { "OK" } else { "NOTOK" }
        );
        i += 1;
    }
    println!(
        "\n--------------------\nFINAL BENCH TIME: {:?}",
        start.elapsed()
    );
}
