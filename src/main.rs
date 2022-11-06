pub mod parser;
pub mod repl;
mod tests;
pub mod vm;

use crate::repl::Repl;
use parser::*;
use std::fs::File;
use std::io::Read;
use vm::*;

use crate::vm_bc::VM;
use clap::Parser;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(path) = args.file {
        let file = File::open(path);
        if let Ok(mut file) = file {
            let mut source = String::new();
            file.read_to_string(&mut source).unwrap();
            let mut vm = VM::default();
            vm.set_source(source.clone());

            let ast_ = parse(source.as_str());
            if let Ok(ast_) = ast_ {
                vm.compile(&ast_);

                if let Some(e) = &vm.error {
                    println!("{e}");
                    return;
                }

                vm.optimize();

                // println!("{}", vm.disassemble());

                vm.execute();
                if let Some(e) = &vm.error {
                    println!("{e}");
                }
            } else if let Err(e) = ast_ {
                if let pest::error::LineColLocation::Span(start, end) = e.line_col {
                    let line_str = source.split('\n').nth(start.0 - 1).unwrap();
                    println!(
                        "At Line {}:\n{}\n{}{}\nSyntax Error",
                        start.0,
                        line_str,
                        " ".repeat(start.1 - 1),
                        "^".repeat(end.1.min(line_str.len()) - start.1),
                    );
                } else if let pest::error::LineColLocation::Pos(pos) = e.line_col {
                    let line_str = source.split('\n').nth(pos.0 - 1).unwrap();
                    println!(
                        "At Line {}:\n{}\n{}^\nSyntax Error",
                        pos.0,
                        line_str,
                        " ".repeat(pos.1 - 1),
                    );
                }
            }
        }
    } else {
        let mut repl_ = Repl::default();
        repl_.run();
    }
}
