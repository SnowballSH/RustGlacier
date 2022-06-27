use super::bytecode::*;
use crate::ast::*;
use crate::value::*;

use arrayvec::ArrayVec;
use pest::Span;

pub const BYTECODE_CAP: usize = 1024;
pub const CONSTANT_SIZE: usize = 1024;
pub const CONSTANT_SMALL_INT_SIZE: usize = 512 + 8;
pub const STACK_SIZE: usize = 8192;

#[derive(Debug, Clone)]
pub struct VM {
    pub source: String,

    pub bytecodes: Vec<Byte>,
    // (Line, Start, End)
    pub lines: Vec<(usize, usize, usize)>,
    pub pc: usize,

    pub constants: ArrayVec<Value, CONSTANT_SIZE>,
    pub constant_hash_small_int: [Option<Byte>; CONSTANT_SMALL_INT_SIZE],

    pub stack: ArrayVec<Value, STACK_SIZE>,
}

impl Default for VM {
    fn default() -> Self {
        VM {
            source: String::new(),

            bytecodes: Vec::with_capacity(BYTECODE_CAP),
            lines: Vec::with_capacity(BYTECODE_CAP),
            pc: 0,

            constants: ArrayVec::new(),
            constant_hash_small_int: [None; CONSTANT_SMALL_INT_SIZE],

            stack: ArrayVec::new(),
        }
    }
}

impl VM {
    pub fn set_source(&mut self, source: String) {
        self.source = source;
    }

    #[inline(always)]
    fn span_to_line(&self, span: &Span) -> usize {
        self.source[..span.start()].matches('\n').count()
    }

    fn get_nl_pos(&self, line: usize) -> usize {
        let mut count = 0;
        for (i, ch) in self.source.chars().enumerate() {
            if ch == '\n' {
                count += 1;
                if count == line {
                    return i + 1;
                }
            }
        }
        unreachable!();
    }

    pub fn compile_error(&self, span: &Span, message: String) -> ! {
        let line = self.span_to_line(span);
        let line_str = &self.source.split('\n').nth(line).unwrap();
        let start = self.get_nl_pos(line);

        eprintln!("At Line {}:\n{}\n{}{}", line + 1, line_str, " ".repeat(span.start() - start), "^".repeat(span.end() - span.start()));
        eprintln!("Compile-time Error:\n    {}", message);
        std::process::exit(1)
    }

    pub fn runtime_error(&self, message: String) -> ! {
        let line = self.lines[self.pc].0;
        let line_str = &self.source.split('\n').nth(line).unwrap();
        let start = self.get_nl_pos(line);
        eprintln!("At Line {}:\n{}\n{}{}", line + 1, line_str, " ".repeat(self.lines[self.pc].1 - start), "^".repeat(self.lines[self.pc].2 - self.lines[self.pc].1));
        eprintln!("Runtime Error:\n    {}", message);
        std::process::exit(1)
    }
}

// Compilation
impl VM {
    #[inline(always)]
    fn push_bytecode(&mut self, bytecode: Byte, span: &Span) {
        self.bytecodes.push(bytecode);
        self.lines.push((self.span_to_line(span), span.start(), span.end()));
    }

    pub fn compile(&mut self, program: &Program) {
        for stmt in program {
            self.compile_statement(stmt);
        }
    }

    pub fn compile_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ExprStmt(e) => {
                self.compile_expression(&e.expr);
                self.push_bytecode(POP_LAST, &e.pos);
            }
            Statement::DebugPrint(e) => {
                self.compile_expression(&e.expr);
                self.push_bytecode(DEBUG_PRINT, &e.pos);
            }
        }
    }

    pub fn compile_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Int(num) => {
                let index;
                if num.value < CONSTANT_SMALL_INT_SIZE as u64 && self.constant_hash_small_int[num.value as usize].is_some() {
                    index = self.constant_hash_small_int[num.value as usize].unwrap();
                } else if self.constants.try_push(Value::Int(num.value as i64)).is_err() {
                    self.compile_error(&num.pos, format!("Constant exceeds limit of {}", CONSTANT_SIZE));
                } else {
                    index = self.constants.len() as Byte - 1;
                    if num.value < CONSTANT_SMALL_INT_SIZE as u64 {
                        self.constant_hash_small_int[num.value as usize] = Some(index);
                    }
                }
                self.push_bytecode(LOAD_CONST, &num.pos);
                self.push_bytecode(index, &num.pos);
            }
            Expression::GetVar(_) => {
                unimplemented!()
            }
            Expression::SetVar(_) => { unimplemented!() }
            Expression::Infix(infix) => {
                match infix.operator {
                    "+" => {}
                    _ => {
                        self.compile_error(&infix.pos, format!("Unsupported Operand: {}", infix.operator));
                    }
                }
            }
            Expression::Prefix(prefix) => {
                match prefix.operator {
                    "-" => {
                        self.compile_expression(&prefix.right);
                        self.push_bytecode(UNARY_NEG, &prefix.pos);
                    }
                    _ => {
                        self.compile_error(&prefix.pos, format!("Unsupported Operand: {}", prefix.operator));
                    }
                }
            }
            Expression::Index(_) => { unimplemented!() }
            Expression::If(_) => { unimplemented!() }
        }
    }
}

// Execution
impl VM {
    #[inline(always)]
    fn read_bytecode(&mut self) -> Byte {
        self.pc += 1;
        self.bytecodes[self.pc - 1]
    }

    pub fn execute(&mut self) {
        self.pc = 0;
        while self.pc < self.bytecodes.len() {
            let bc = self.read_bytecode();
            match bc {
                POP_LAST => {
                    self.stack.pop();
                }

                LOAD_CONST => {
                    let index = self.read_bytecode();
                    if self.stack.try_push(self.constants[index as usize].clone()).is_err() {
                        self.runtime_error("Stack overflow".to_string());
                    }
                }

                DEBUG_PRINT => {
                    let value = self.stack.pop().unwrap();
                    println!("{}", value.debug_format());
                }

                UNARY_NEG => {
                    let value = self.stack.pop().unwrap();
                    match value {
                        Value::Int(i) => {
                            self.stack.push(Value::Int(-i));
                        }
                        _ => {
                            self.runtime_error(format!("Unsupported Unary operation: -{}", value.type_name()));
                        }
                    }
                }

                _ => {
                    self.runtime_error(format!("Unknown bytecode: {}", bc));
                }
            }
        }
    }
}
