use super::bytecode::*;
use crate::ast::*;
use crate::value::*;

use arrayvec::ArrayVec;
use pest::Span;

pub const BYTECODE_CAP: usize = 1024;
pub const CONSTANT_SIZE: usize = 1024;
pub const CONSTANT_SMALL_INT_SIZE: usize = 512 + 8;
pub const STACK_SIZE: usize = 8192;

pub const BOOL_FALSE_CONSTANT: usize = 0;
pub const BOOL_TRUE_CONSTANT: usize = 1;

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

    pub last_popped: Option<Value>,
    pub repl_mode: bool,

    pub error: Option<String>,
}

impl Default for VM {
    fn default() -> Self {
        let mut v = VM {
            source: String::new(),

            bytecodes: Vec::with_capacity(BYTECODE_CAP),
            lines: Vec::with_capacity(BYTECODE_CAP),
            pc: 0,

            constants: ArrayVec::new(),
            constant_hash_small_int: [None; CONSTANT_SMALL_INT_SIZE],

            stack: ArrayVec::new(),

            last_popped: None,
            repl_mode: false,

            error: None,
        };

        v.constants.push(Value::Bool(false));
        v.constants.push(Value::Bool(true));

        v
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
        if line == 0 {
            return 0;
        }
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

    pub fn compile_error(&mut self, span: &Span, message: String) {
        let line = self.span_to_line(span);
        let line_str = &self.source.split('\n').nth(line).unwrap();
        let start = self.get_nl_pos(line);

        self.error = Some(
            format!(
                "At Line {}:\n{}\n{}{}\nCompile-time Error:\n    {}",
                line + 1,
                line_str,
                " ".repeat(span.start() - start),
                "^".repeat(span.end() - span.start()),
                message
            )
        );
    }

    pub fn runtime_error(&mut self, message: String) {
        let line = self.lines[self.pc].0;
        let line_str = &self.source.split('\n').nth(line).unwrap();
        let start = self.get_nl_pos(line);
        self.error = Some(
            format!(
                "At Line {}:\n{}\n{}{}\nRuntime Error:\n    {}",
                line + 1,
                line_str,
                " ".repeat(self.lines[self.pc].1 - start),
                "^".repeat(self.lines[self.pc].2 - self.lines[self.pc].1),
                message
            )
        );
        self.stack.clear();
    }
}

// Compilation
impl VM {
    #[inline(always)]
    fn push_bytecode(&mut self, bytecode: Byte, span: &Span) {
        self.bytecodes.push(bytecode);
        self.lines
            .push((self.span_to_line(span), span.start(), span.end()));
    }

    pub fn compile(&mut self, program: &Program) {
        self.lines.clear();
        self.bytecodes.clear();
        for stmt in program {
            self.compile_statement(stmt);
        }
    }

    pub fn compile_statement(&mut self, statement: &Statement) -> bool {
        match statement {
            Statement::ExprStmt(e) => {
                if !self.compile_expression(&e.expr) {
                    return false;
                };
                self.push_bytecode(POP_LAST, &e.pos);
            }
            Statement::DebugPrint(e) => {
                if !self.compile_expression(&e.expr) {
                    return false;
                };
                self.push_bytecode(DEBUG_PRINT, &e.pos);
            }
        }
        true
    }

    pub fn compile_expression(&mut self, expression: &Expression) -> bool {
        match expression {
            Expression::Int(num) => {
                let index;
                if num.value < CONSTANT_SMALL_INT_SIZE as u64
                    && self.constant_hash_small_int[num.value as usize].is_some()
                {
                    index = self.constant_hash_small_int[num.value as usize].unwrap();
                } else if self
                    .constants
                    .try_push(Value::Int(num.value as i64))
                    .is_err()
                {
                    self.compile_error(
                        &num.pos,
                        format!("Constant exceeds limit of {}", CONSTANT_SIZE),
                    );
                    return false;
                } else {
                    index = self.constants.len() as Byte - 1;
                    if num.value < CONSTANT_SMALL_INT_SIZE as u64 {
                        self.constant_hash_small_int[num.value as usize] = Some(index);
                    }
                }
                self.push_bytecode(LOAD_CONST, &num.pos);
                self.push_bytecode(index, &num.pos);
            }

            Expression::Bool(b) => {
                let index = if b.value {
                    BOOL_TRUE_CONSTANT
                } else {
                    BOOL_FALSE_CONSTANT
                };
                self.push_bytecode(LOAD_CONST, &b.pos);
                self.push_bytecode(index as Byte, &b.pos);
            }

            Expression::GetVar(_) => {
                unimplemented!()
            }
            Expression::SetVar(_) => {
                unimplemented!()
            }
            Expression::Infix(infix) => {
                self.compile_expression(&infix.left);
                self.compile_expression(&infix.right);
                match infix.operator {
                    "+" => {
                        self.push_bytecode(BINARY_ADD, &infix.pos);
                    }
                    "-" => {
                        self.push_bytecode(BINARY_SUB, &infix.pos);
                    }
                    "*" => {
                        self.push_bytecode(BINARY_MUL, &infix.pos);
                    }
                    "/" => {
                        self.push_bytecode(BINARY_DIV, &infix.pos);
                    }
                    "%" => {
                        self.push_bytecode(BINARY_MOD, &infix.pos);
                    }
                    "==" => {
                        self.push_bytecode(BINARY_EQ, &infix.pos);
                    }
                    "!=" => {
                        self.push_bytecode(BINARY_NE, &infix.pos);
                    }
                    _ => {
                        self.compile_error(
                            &infix.pos,
                            format!("Unsupported Operand: {}", infix.operator),
                        );
                        return false;
                    }
                }
            }
            Expression::Prefix(prefix) => match prefix.operator {
                "-" => {
                    self.compile_expression(&prefix.right);
                    self.push_bytecode(UNARY_NEG, &prefix.pos);
                }
                "!" => {
                    self.compile_expression(&prefix.right);
                    self.push_bytecode(UNARY_NOT, &prefix.pos);
                }
                _ => {
                    self.compile_error(
                        &prefix.pos,
                        format!("Unsupported Operand: {}", prefix.operator),
                    );
                    return false;
                }
            },
            Expression::Index(_) => {
                unimplemented!()
            }
            Expression::If(_) => {
                unimplemented!()
            }
        }

        true
    }

    pub fn disassemble(&self) -> String {
        let mut s = String::new();
        let mut pc = 0;
        while pc < self.bytecodes.len() {
            let old_pc = pc;
            let byte = self.bytecodes[pc];
            let mut args: Vec<String> = vec![];
            match byte {
                LOAD_CONST => {
                    pc += 1;
                    let address = self.bytecodes[pc] as usize;
                    args.push(format!(
                        "{:02x} ({})",
                        address,
                        self.constants[address].debug_format()
                    ));
                }
                _ => (),
            }

            s.push_str(&format!(
                "{:04x}: {} {}\n",
                old_pc,
                bytecode_name(byte),
                args.join(", ")
            ));

            pc += 1;
        }
        s
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
        self.stack.clear();
        self.last_popped = None;
        self.pc = 0;
        while self.pc < self.bytecodes.len() {
            let bc = self.read_bytecode();
            match bc {
                // General
                POP_LAST => {
                    let v = self.stack.pop();
                    if self.repl_mode {
                        self.last_popped = v;
                    }
                }

                LOAD_CONST => {
                    let index = self.read_bytecode();
                    if self
                        .stack
                        .try_push(self.constants[index as usize].clone())
                        .is_err()
                    {
                        self.runtime_error("Stack overflow".to_string());
                        return;
                    }
                }

                DEBUG_PRINT => {
                    let value = self.stack.pop().unwrap();
                    println!("{}", value.debug_format());
                }

                // Prefix operators
                UNARY_NEG => {
                    let value = self.stack.pop().unwrap();
                    match value {
                        Value::Bool(_) => {
                            self.runtime_error(
                                "Unsupported Unary operation: -bool (Hint: Use !bool instead)"
                                    .to_string(),
                            );
                            return;
                        }
                        Value::Int(i) => {
                            unsafe {
                                // We just popped an element, so there should be an empty space on the stack.
                                self.stack.push_unchecked(Value::Int(i.saturating_neg()));
                            }
                        }
                        _ => {
                            self.runtime_error(format!(
                                "Unsupported Unary operation: -{}",
                                value.type_name()
                            ));
                            return;
                        }
                    }
                }

                UNARY_NOT => {
                    let value = self.stack.pop().unwrap();
                    unsafe {
                        self.stack.push_unchecked(Value::Bool(!value.is_truthy()));
                    }
                }

                // Infix operators
                BINARY_ADD => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (&left, &right) {
                        (Value::Int(l), Value::Int(r)) => unsafe {
                            self.stack.push_unchecked(Value::Int(l.wrapping_add(*r)));
                        },
                        _ => {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} + {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }
                }

                BINARY_SUB => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (&left, &right) {
                        (Value::Int(l), Value::Int(r)) => unsafe {
                            self.stack.push_unchecked(Value::Int(l.wrapping_sub(*r)));
                        },
                        _ => {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} - {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }
                }

                BINARY_MUL => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (&left, &right) {
                        (Value::Int(l), Value::Int(r)) => unsafe {
                            self.stack.push_unchecked(Value::Int(l.wrapping_mul(*r)));
                        },
                        _ => {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} * {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }
                }

                BINARY_DIV => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (&left, &right) {
                        (Value::Int(l), Value::Int(r)) => {
                            if *r == 0 {
                                self.runtime_error(format!(
                                    "Division by zero: {} / 0",
                                    left.debug_format()
                                ));
                                return;
                            }
                            unsafe {
                                self.stack.push_unchecked(Value::Int(l.wrapping_div(*r)));
                            }
                        }
                        _ => {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} / {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }
                }

                BINARY_MOD => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (&left, &right) {
                        (Value::Int(l), Value::Int(r)) => {
                            if *r == 0 {
                                self.runtime_error(format!(
                                    "Modulo by zero: {} % 0",
                                    left.debug_format()
                                ));
                                return;
                            }
                            unsafe {
                                self.stack.push_unchecked(Value::Int(l.wrapping_rem(*r)));
                            }
                        }
                        _ => {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} % {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }
                }

                BINARY_EQ => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    unsafe {
                        self.stack.push_unchecked(Value::Bool(left == right));
                    }
                }

                BINARY_NE => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    unsafe {
                        self.stack.push_unchecked(Value::Bool(left != right));
                    }
                }

                // Invalid
                _ => {
                    self.runtime_error(format!("Unknown bytecode: {}", bc));
                    return;
                }
            }
        }
    }
}
