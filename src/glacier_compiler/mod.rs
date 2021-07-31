use pest::Span;

use crate::glacier_parser::ast::{Expression, Program, Statement};
use crate::glacier_parser::span_to_line;
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::Value;

#[derive(Debug)]
pub struct Compiler<'a> {
    pub source: &'a str,
    pub result: Vec<Instruction<'a>>,
    last_line: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            result: Vec::with_capacity(64),
            last_line: 0,
        }
    }

    fn update_line(&mut self, pos: Span) {
        let l = span_to_line(&*self.source, pos);
        if self.last_line == l {
            return;
        }
        self.last_line = l;
        self.result
            .push(Instruction::SetLine(l));
    }

    pub fn compile(&mut self, ast: Program<'a>) {
        for s in ast {
            self.compile_statement(s);
        }
    }

    pub fn compile_statement(&mut self, stmt: Statement<'a>) {
        match stmt {
            Statement::ExprStmt(x) => {
                self.compile_expression(x.expr);
                self.result.push(Instruction::Pop);
            }
            Statement::FunctionDeclare(_) => unimplemented!(),
        }
    }

    pub fn compile_expression(&mut self, expr: Expression<'a>) {
        match expr {
            Expression::Int(x) => {
                self.update_line(x.pos);
                self.result.push(Instruction::Push(Value::Int(x.value as i64)));
            }
            Expression::String(x) => {
                self.update_line(x.pos);
                self.result.push(Instruction::Push(Value::String(x.value)));
            }
            Expression::SetVar(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.value);
                self.result.push(Instruction::Var(x.name));
                self.result.push(Instruction::MoveLast);
            }
            Expression::GetVar(x) => {
                self.update_line(x.pos);
                self.result.push(Instruction::MoveVar(x.name));
            }
            Expression::Infix(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.left);
                self.result.push(Instruction::MoveLastToStack);
                self.compile_expression(x.right);
                self.result.push(Instruction::MoveLastToStack);
                self.result.push(Instruction::BinaryOperator(x.operator));
            }
            Expression::Prefix(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.right);
                self.result.push(Instruction::MoveLastToStack);
                self.result.push(Instruction::UnaryOperator(x.operator));
            }
            Expression::Call(x) => {
                let mut x = x.clone();
                self.update_line(x.pos);
                x.arguments.reverse();
                let mut k = 0;
                for m in x.arguments {
                    self.compile_expression(m);
                    self.result.push(Instruction::MoveLastToStack);
                    k += 1;
                }
                self.compile_expression(x.callee);
                self.result.push(Instruction::Call(k));
            }
            Expression::If(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.cond);
                let pos1 = self.result.len();
                self.result.push(Instruction::Noop);
                self.compile(x.body);

                if self.result.last() == Some(&Instruction::Pop) {
                    self.result.pop();
                } else {
                    self.result.push(Instruction::Push(Value::Null));
                }
                self.result[pos1] = Instruction::JumpIfFalse(self.result.len());
                let pos2 = self.result.len();
                self.result.push(Instruction::Noop);
                self.compile(x.other);

                if self.result.last() == Some(&Instruction::Pop) {
                    self.result.pop();
                } else {
                    self.result.push(Instruction::Push(Value::Null));
                }
                self.result[pos2] = Instruction::Jump(self.result.len() - 1);
            }
            _ => unimplemented!(),
        }
    }
}
