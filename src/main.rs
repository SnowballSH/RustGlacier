use glacier_lang::glacier_compiler::Compiler;
use glacier_lang::glacier_parser::parse;
use glacier_lang::glacier_vm::vm::VM;

fn main() {
    let code = r#"
    l = abc = 123454321
    abc
    l
    "#;
    let ast = parse(code).unwrap();

    let mut compiler = Compiler::default();
    compiler.compile(ast);

    dbg!(&compiler.result);

    let mut vm = VM::default();
    vm.run(compiler.result);

    dbg!(&vm.heap);
    dbg!(&vm.variables);
    dbg!(vm.get_variable("abc"));
}
