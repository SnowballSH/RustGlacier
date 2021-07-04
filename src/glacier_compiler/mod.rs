use crate::glacier_vm::instructions::Instruction;
use crate::glacier_parser::ast::*;
use crate::glacier_vm::value::Value::Int;

#[derive(Debug, Default)]
pub struct Compiler<'a> {
    pub result: Vec<Instruction<'a>>,
}

impl<'a> Compiler<'a> {
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
            Statement::FunctionDeclare(_) => unimplemented!()
        }
    }

    pub fn compile_expression(&mut self, expr: Expression<'a>) {
        match expr {
            Expression::Int(x) => {
                self.result.push(Instruction::Push(Int(x.value as i64)));
            }
            Expression::SetVar(x) => {
                self.compile_expression(x.value);
                self.result.push(Instruction::Var(x.name));
                self.result.push(Instruction::MoveLast);
            }
            Expression::GetVar(x) => {
                self.result.push(Instruction::MoveVar(x.name));
            }
            _ => unimplemented!()
        }
    }
}
