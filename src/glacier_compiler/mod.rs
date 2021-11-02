use pest::Span;

use crate::glacier_parser::ast::{Expression, Program, Statement};
use crate::glacier_parser::span_to_line;
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerVarDef {
    pub name: String,
    pub var_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents linking of name to var location.
pub struct CompilerVariableMap {
    pub variables: Vec<Vec<CompilerVarDef>>,
    pub frees: Vec<Vec<CompilerVarDef>>,
}

impl Default for CompilerVariableMap {
    fn default() -> Self {
        CompilerVariableMap {
            variables: Vec::with_capacity(4),
            frees: Vec::with_capacity(4),
        }
    }
}

impl CompilerVariableMap {
    pub fn insert(&mut self, key: String) -> usize {
        Self::insert_for(key, self.variables.last_mut().unwrap())
    }

    pub fn insert_free(&mut self, key: String) -> usize {
        Self::insert_for(key, self.frees.last_mut().unwrap())
    }

    pub fn insert_for(key: String, one: &mut Vec<CompilerVarDef>) -> usize {
        let last_count = if one.is_empty() {
            0
        } else {
            one.last().unwrap().var_index + 1
        };
        one.push(CompilerVarDef {
            name: key,
            var_index: last_count,
        });
        last_count
    }

    pub fn get(&mut self, key: &String) -> Option<usize> {
        for item in self.variables.last().unwrap() {
            if &item.name == key {
                return Some(item.var_index);
            }
        }
        None
    }

    pub fn get_free(&mut self, key: &String) -> Option<usize> {
        for item in self.frees.last().unwrap() {
            if &item.name == key {
                return Some(item.var_index);
            }
        }
        None
    }

    pub fn try_capture(&mut self, key: &String) -> Option<(usize, usize, usize)> {
        let mut starti = None;
        let mut fval = None;
        'f: for (i, m) in self.variables.iter().enumerate().rev() {
            for item in m {
                if &item.name == key {
                    starti = Some(i);
                    fval = Some(item.clone());
                    break 'f;
                }
            }
        }

        if starti.is_none() {
            return None;
        }

        Some((
            starti.unwrap(),
            fval.unwrap().var_index,
            self.insert_free(key.clone()),
        ))
    }

    pub fn release(&mut self) {
        self.variables.pop();
        self.frees.pop();
    }

    pub fn new_frame(&mut self) {
        self.variables.push(Vec::new());
        self.frees.push(Vec::new());
    }
}

#[derive(Debug)]
pub struct Compiler<'a> {
    pub source: &'a str,
    pub result: Vec<Instruction>,
    pub last_line: usize,
    pub variable_map: CompilerVariableMap,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            result: Vec::with_capacity(64),
            last_line: 0,
            variable_map: CompilerVariableMap::default(),
        }
    }

    pub fn clean_up(&mut self) {
        for i in 1..self.result.len() {
            if let Some(Instruction::Push(v)) = self.result.get(i - 1) {
                if let Instruction::DefineLocal(n) = unsafe { self.result.get_unchecked(i) } {
                    self.result[i - 1] = Instruction::PushLocal(v.clone(), n.clone());
                    self.result[i] = Instruction::Noop;
                }
            }
        }
    }

    fn update_line(&mut self, pos: Span) {
        let l = span_to_line(&*self.source, pos);
        if self.last_line == l {
            return;
        }
        self.last_line = l;
        self.result.push(Instruction::SetLine(l));
    }

    #[inline]
    pub fn compile(&mut self, ast: Program<'a>) {
        self.variable_map.new_frame();
        self.compile_no_new_frame(ast);
        self.variable_map.release();
    }

    #[inline]
    pub fn compile_no_new_frame(&mut self, ast: Program<'a>) {
        for s in ast {
            self.compile_statement(s);
        }
        self.clean_up();
    }

    pub fn compile_statement(&mut self, stmt: Statement<'a>) {
        match stmt {
            Statement::ExprStmt(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.expr);
                self.result.push(Instruction::Pop);
            }
            Statement::FunctionDeclare(x) => {
                let jump_index = self.result.len();
                self.result.push(Instruction::Noop);
                let index = self.result.len();

                let loc = self.variable_map.insert(x.name.to_string());

                self.variable_map.new_frame();
                let mut argloc = vec![];
                for y in x.args {
                    argloc.push(self.variable_map.insert(y.to_string()));
                }

                self.compile_no_new_frame(x.body);
                if let Some(Instruction::Ret) = self.result.last() {
                } else if let Some(Instruction::Pop) = self.result.last() {
                    self.result.pop();
                    self.result.push(Instruction::Ret);
                } else {
                    self.result.push(Instruction::Push(Value::Null));
                    self.result.push(Instruction::Ret);
                }

                self.result[jump_index] = Instruction::Jump(self.result.len());

                self.variable_map.release();

                self.update_line(x.pos);
                self.result.push(Instruction::MakeCode(
                    index,
                    x.name.to_string(),
                    argloc,
                    loc,
                ));
            }
            Statement::EmptyReturn(x) => {
                self.update_line(x.pos);
                self.result.push(Instruction::Push(Value::Null));
                self.result.push(Instruction::Ret);
            }
            Statement::Return(x) => {
                self.compile_expression(x.expr);
                self.update_line(x.pos);
                self.result.push(Instruction::Ret);
            }
        }
    }

    pub fn compile_expression(&mut self, expr: Expression<'a>) {
        match expr {
            Expression::Int(x) => {
                self.update_line(x.pos);
                self.result
                    .push(Instruction::Push(Value::Int(x.value as i64)));
            }
            Expression::String(x) => {
                self.update_line(x.pos);
                self.result.push(Instruction::Push(Value::String(x.value)));
            }
            Expression::SetVar(x) => {
                self.update_line(x.pos);
                let vari = if let Some(e) = self.variable_map.get(&x.name.to_string()) {
                    e
                } else {
                    self.variable_map.insert(x.name.to_string())
                };
                self.compile_expression(x.value);
                self.result.push(Instruction::DefineLocal(vari));
                self.result.push(Instruction::MoveLastFromHeapToStack);
            }
            Expression::GetVar(x) => {
                self.update_line(x.pos);
                if let Some(vari) = self.variable_map.get(&x.name.to_string()) {
                    self.result.push(Instruction::MoveLocal(vari));
                } else if let Some(k) = self.variable_map.get_free(&x.name.to_string()) {
                    self.result.push(Instruction::MoveFree(k));
                } else if let Some((a, b, c)) = self.variable_map.try_capture(&x.name.to_string()) {
                    self.result.push(Instruction::Link(a, b, c));
                } else {
                    self.result.push(Instruction::MoveVar(x.name.to_string()));
                }
            }
            Expression::Infix(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.left);
                self.compile_expression(x.right);
                self.result
                    .push(Instruction::BinaryOperator(x.operator.to_string()));
            }
            Expression::Prefix(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.right);
                self.result
                    .push(Instruction::UnaryOperator(x.operator.to_string()));
            }
            Expression::Call(x) => {
                let mut x = x.clone();
                self.update_line(x.pos);
                x.arguments.reverse();
                let mut k = 0;
                for m in x.arguments {
                    self.compile_expression(m);
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
                self.compile_no_new_frame(x.body);

                if self.result.last() == Some(&Instruction::Pop) {
                    self.result.pop();
                } else {
                    self.result.push(Instruction::Push(Value::Null));
                }
                self.result[pos1] = Instruction::JumpIfFalse(self.result.len() + 1);
                let pos2 = self.result.len();
                self.result.push(Instruction::Noop);
                self.compile_no_new_frame(x.other);

                if self.result.last() == Some(&Instruction::Pop) {
                    self.result.pop();
                } else {
                    self.result.push(Instruction::Push(Value::Null));
                }
                self.result[pos2] = Instruction::Jump(self.result.len());
            }
            Expression::While(x) => {
                self.update_line(x.pos);
                let org = self.result.len();
                self.compile_expression(x.cond);
                let pos = self.result.len();
                self.result.push(Instruction::Noop);
                self.compile_no_new_frame(x.body);
                self.result.push(Instruction::Jump(org));
                self.result[pos] = Instruction::JumpIfFalse(self.result.len());
                self.result.push(Instruction::Push(Value::Null));
            }

            Expression::GetProperty(x) => {
                self.update_line(x.pos);
                self.compile_expression(x.parent);
                self.result
                    .push(Instruction::GetProperty(x.name.to_string()));
            }
            _ => unimplemented!(),
        }
    }
}
