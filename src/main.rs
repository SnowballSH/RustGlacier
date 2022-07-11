pub mod parser;
pub mod vm;
pub mod repl;

use parser::*;
use vm::*;
use crate::repl::Repl;

fn main() {
    let mut repl_ = Repl::default();
    repl_.run();
}
