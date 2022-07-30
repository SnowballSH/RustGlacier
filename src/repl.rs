use crate::parser::*;
use crate::vm::*;

use std::io;
use std::io::Write;

#[derive(Debug, Default)]
pub struct Repl {
    pub vm: vm_bc::VM,
}

impl Repl {
    pub fn run(&mut self) {
        println!("REPL for Glacier 2.0 dev");

        self.vm.repl_mode = true;

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            input = input.trim().to_string();

            if input == ":quit" {
                break;
            }

            self.vm.error = None;
            self.vm.set_source(input.clone());

            let ast_ = parse(input.as_str());
            if let Ok(ast_) = ast_ {
                self.vm.compile(&ast_);

                if let Some(e) = &self.vm.error {
                    println!("{}", e);
                    continue;
                }

                self.vm.optimize();

                // println!("{}", self.vm.disassemble());
                // println!("{:?}", self.vm.current_compiler);

                self.vm.execute();

                // println!("{:?}", self.vm.stack);

                if let Some(e) = &self.vm.error {
                    println!("{}", e);
                    continue;
                }

                if let Some(lp) = &self.vm.last_popped {
                    if let value::Value::Null = lp {
                    } else {
                        println!("#>> {}", lp.debug_format());
                    }
                }
            } else if let Err(e) = ast_ {
                if let pest::error::LineColLocation::Span(start, end) = e.line_col {
                    let line_str = input.split('\n').nth(start.0 - 1).unwrap();
                    println!(
                        "At Line {}:\n{}\n{}{}\nSyntax Error",
                        start.0,
                        line_str,
                        " ".repeat(start.1 - 1),
                        "^".repeat(end.1.min(line_str.len()) - start.1),
                    );
                } else if let pest::error::LineColLocation::Pos(pos) = e.line_col {
                    let line_str = input.split('\n').nth(pos.0 - 1).unwrap();
                    println!(
                        "At Line {}:\n{}\n{}^\nSyntax Error",
                        pos.0,
                        line_str,
                        " ".repeat(pos.1 - 1),
                    );
                }
            }
        }
    }
}
