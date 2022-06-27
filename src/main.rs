pub mod parser;
pub mod vm;

use parser::*;
use vm::*;

fn main() {
    let code = "
        debug 3
        debug -2
    ";
    let ast_ = parse(code).unwrap();
    // println!("{:?}", ast_);
    let mut vm_ = vm_bc::VM::default();
    vm_.set_source(code.to_string());
    vm_.compile(&ast_);
    println!("{:?}", vm_.bytecodes);
    vm_.execute();
}
