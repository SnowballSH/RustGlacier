use glacier_lang::glacier_compiler::Compiler;
use glacier_lang::glacier_parser::parse;
use glacier_lang::glacier_vm::vm::VM;

fn main() {
    let code = r#"
    a = 2356
    b = 92845
    a + b + a
    "#;
    let ast = parse(code).unwrap();

    let mut compiler = Compiler::new(code);
    compiler.compile(ast);

    dbg!(&compiler.result);

    let mut vm = VM::default();
    vm.run(compiler.result);

    dbg!(&vm);
}
