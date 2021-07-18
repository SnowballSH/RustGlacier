use crate::glacier_parser::ast::{Program, Statement, Expression};
use crate::glacier_parser::span_to_line;
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::Value::Int;

#[derive(Debug)]
pub struct Compiler<'a> {
    pub source: &'a str,
    pub result: Vec<Instruction<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            result: Vec::with_capacity(64),
        }
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
                self.result
                    .push(Instruction::SetLine(span_to_line(&*self.source, x.pos)));
                self.result.push(Instruction::Push(Int(x.value as i64)));
            }
            Expression::SetVar(x) => {
                self.result
                    .push(Instruction::SetLine(span_to_line(&*self.source, x.pos)));
                self.compile_expression(x.value);
                self.result.push(Instruction::Var(x.name));
                self.result.push(Instruction::MoveLast);
            }
            Expression::GetVar(x) => {
                self.result
                    .push(Instruction::SetLine(span_to_line(&*self.source, x.pos)));
                self.result.push(Instruction::MoveVar(x.name));
            }
            Expression::Infix(x) => {
                self.result
                    .push(Instruction::SetLine(span_to_line(&*self.source, x.pos)));
                self.compile_expression(x.left);
                self.compile_expression(x.right);
                self.result.push(Instruction::BinaryOperator(x.operator));
            }
            _ => unimplemented!(),
        }
    }
}
