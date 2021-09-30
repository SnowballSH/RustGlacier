use std::collections::BTreeSet;

use crate::backends::CodeGen;
use crate::glacier_parser::ast::{Expression, Program, Statement};

pub const JS_LIB: &'static str = include_str!("glacier.min.js");

#[derive(Debug, Clone)]
pub struct JSCodeGen {
    pub initialized_vars: Vec<BTreeSet<String>>,
}

impl Default for JSCodeGen {
    fn default() -> Self {
        JSCodeGen {
            initialized_vars: vec![BTreeSet::new()],
        }
    }
}

impl JSCodeGen {
    pub fn gen_stmt<'a>(&mut self, stmt: &'a Statement<'a>) -> String {
        let mut res = String::new();
        match stmt {
            Statement::ExprStmt(e) => {
                res.push_str(&*self.gen_expr(&e.expr));
                res.push(';');
            }
            Statement::FunctionDeclare(f) => {
                self.initialized_vars.push(BTreeSet::new());
                let mut code = format!(
                    "function {}({}){{{}",
                    f.name,
                    f.args.join(","),
                    self.gen_program(&f.body)
                );

                let iv = self.initialized_vars.pop().unwrap();
                if !iv.is_empty() {
                    code.push_str("var ");
                    for v in iv {
                        code.push_str(&*format!("{},", v));
                    }
                    code.pop();
                    code.push(';');
                }
                code.push('}');
                res.push_str(&*code);
            }
            Statement::Return(r) => {
                res.push_str(&*format!("return {};", self.gen_expr(&r.expr)));
            }
            Statement::EmptyReturn(_) => {
                res.push_str("return;");
            }
        }
        res
    }

    pub fn gen_expr<'a>(&mut self, expr: &'a Expression) -> String {
        let mut res = String::new();
        match expr {
            Expression::Int(i) => {
                res.push_str(&*i.value.to_string());
            }
            Expression::String(s) => {
                res.push_str(&*format!("\"{}\"", s.value));
            }
            Expression::GetVar(g) => {
                res.push_str(g.name);
            }
            Expression::SetVar(s) => {
                if self.initialized_vars.last().unwrap().contains(s.name) {
                } else {
                    self.initialized_vars
                        .last_mut()
                        .unwrap()
                        .insert(s.name.to_string());
                }
                res.push_str(&*format!("({}={})", s.name, self.gen_expr(&s.value)));
            }
            Expression::Infix(i) => {
                res.push_str(&*format!(
                    "({}{}{})",
                    self.gen_expr(&i.left),
                    i.operator,
                    self.gen_expr(&i.right)
                ));
            }
            Expression::Prefix(p) => {
                res.push_str(&*format!("({}{})", p.operator, self.gen_expr(&p.right)));
            }
            Expression::Call(c) => {
                let mut args: Vec<String> = vec![];
                for arg in &c.arguments {
                    args.push(self.gen_expr(arg));
                }
                res.push_str(&*format!(
                    "{}({})",
                    self.gen_expr(&c.callee),
                    args.join(",")
                ));
            }
            Expression::GetInstance(g) => {
                res.push_str(&*format!("{}.{}", self.gen_expr(&g.parent), g.name));
            }
            Expression::Index(i) => {
                res.push_str(&*format!(
                    "{}[{}]",
                    self.gen_expr(&i.callee),
                    self.gen_expr(&i.index)
                ));
            }
            Expression::Vec_(v) => {
                res.push_str(&*format!(
                    "[{}]",
                    v.values
                        .iter()
                        .map(|x| self.gen_expr(x))
                        .collect::<Vec<String>>()
                        .join(",")
                ));
            }
            Expression::If(i) => {
                res.push_str(&*format!(
                    "if({}){{{}}}else{{{}}}",
                    self.gen_expr(&i.cond),
                    self.gen_program(&i.body),
                    self.gen_program(&i.other)
                ));
            }
            Expression::While(w) => {
                res.push_str(&*format!(
                    "while({}){{{}}}",
                    self.gen_expr(&w.cond),
                    self.gen_program(&w.body)
                ));
            }
        }
        res
    }

    pub fn gen_program<'a>(&mut self, program: &'a Program) -> String {
        let mut res = String::new();
        for stmt in program {
            res.push_str(&*self.gen_stmt(stmt));
        }
        res
    }
}

impl CodeGen for JSCodeGen {
    type ResType = String;
    type OptionType = ();

    fn generate<'a>(
        &mut self,
        program: &'a Program<'a>,
        _options: Self::OptionType,
    ) -> Self::ResType {
        let mut full = JS_LIB.to_string();
        full.push_str(&*self.gen_program(program));
        let iv = self.initialized_vars.pop().unwrap();
        if !iv.is_empty() {
            full.push_str("var ");
            for v in iv {
                full.push_str(&*format!("{},", v));
            }
            full.pop();
            full.push(';');
        }
        full
    }
}
