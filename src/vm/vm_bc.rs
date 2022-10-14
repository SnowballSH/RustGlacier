use std::collections::HashMap;
use std::fmt::Write;

use arrayvec::ArrayVec;

use crate::ast::*;
use crate::value::*;

use super::bytecode::*;
use super::memory::*;

pub const BYTECODE_CAP: usize = 1024;
pub const CONSTANT_SIZE: usize = 1024;
pub const LOCAL_SIZE: usize = 1024;
pub const SCOPE_SIZE: usize = 512;
pub const STACK_SIZE: usize = 8192;

pub const BOOL_FALSE_CONSTANT: usize = 0;
pub const BOOL_TRUE_CONSTANT: usize = 1;
pub const NULL_CONSTANT: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    pub local_map: ArrayVec<HashMap<String, usize>, SCOPE_SIZE>,
    pub scope_depth: usize,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct VM {
    pub source: String,

    pub bytecodes: Vec<Byte>,
    // (Start, End)
    pub lines: Vec<AstSpan>,
    pub pc: usize,

    pub constants: ArrayVec<Value, CONSTANT_SIZE>,
    pub constant_hash_int: HashMap<i64, Byte>,

    pub break_jump_patches: Vec<Vec<usize>>,
    pub next_jump_patches: Vec<Vec<usize>>,

    pub current_compiler: Compiler,

    pub stack: ArrayVec<*mut Value, STACK_SIZE>,

    pub last_popped: Option<*mut Value>,
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
            constant_hash_int: HashMap::new(),

            break_jump_patches: Vec::new(),
            next_jump_patches: Vec::new(),

            current_compiler: Default::default(),

            stack: ArrayVec::new(),

            last_popped: None,
            repl_mode: false,

            error: None,
        };

        v.constants.push(Value::Bool(false));
        v.constants.push(Value::Bool(true));
        v.constants.push(Value::Null);

        v.current_compiler.local_map.push(HashMap::new());

        v
    }
}

impl VM {
    pub fn set_source(&mut self, source: String) {
        self.source = source;
    }

    #[inline(always)]
    fn span_to_line(&self, span: AstSpan) -> usize {
        self.source[..span.start].matches('\n').count()
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

    pub fn compile_error(&mut self, span: AstSpan, message: String) {
        let line = self.span_to_line(span);
        let line_str = &self.source.split('\n').nth(line).unwrap();
        let start = self.get_nl_pos(line);

        self.error = Some(format!(
            "At Line {}:\n{}\n{}{}\nCompile-time Error:\n    {}",
            line + 1,
            line_str,
            " ".repeat(span.start - start),
            "^".repeat(span.end - span.start),
            message
        ));
    }

    pub fn runtime_error(&mut self, message: String) {
        let span = self.lines[self.pc];
        let line = self.span_to_line(span);
        let line_str = &self.source.split('\n').nth(line).unwrap();
        let start = self.get_nl_pos(line);
        self.error = Some(format!(
            "At Line {}:\n{}\n{}{}\nRuntime Error:\n    {}",
            line + 1,
            line_str,
            " ".repeat(span.start - start),
            "^".repeat(span.end - span.start),
            message
        ));
    }
}

// Compilation
impl VM {
    fn push_bytecode(&mut self, bytecode: Byte, span: AstSpan) {
        self.bytecodes.push(bytecode);
        self.lines.push(AstSpan {
            start: span.start,
            end: span.end,
        });
    }

    pub fn begin_scope(&mut self) {
        self.current_compiler.scope_depth += 1;
        self.current_compiler.local_map.push(HashMap::new());
    }

    pub fn end_scope(&mut self) {
        self.current_compiler.scope_depth -= 1;
        self.current_compiler.count -= self.current_compiler.local_map.pop().unwrap().len();
    }

    pub fn add_local(&mut self, name: String) -> usize {
        for i in (0..=self.current_compiler.scope_depth).rev() {
            if let Some(index) = self.current_compiler.local_map[i].get(&name) {
                return *index;
            }
        }
        self.current_compiler.local_map[self.current_compiler.scope_depth]
            .insert(name, self.current_compiler.count);
        self.current_compiler.count += 1;
        self.current_compiler.count - 1
    }

    pub fn resolve_local(&mut self, name: String) -> Option<usize> {
        for i in (0..=self.current_compiler.scope_depth).rev() {
            if let Some(index) = self.current_compiler.local_map[i].get(&name) {
                return Some(*index);
            }
        }
        None
    }

    pub fn compile(&mut self, program: &Program) {
        self.lines.clear();
        self.bytecodes.clear();
        while self.current_compiler.local_map.len() > 1 {
            self.current_compiler.local_map.pop();
        }
        self.current_compiler.count = self.current_compiler.local_map[0].len();
        self.compile_program(program);
    }

    pub fn compile_program(&mut self, program: &Program) -> bool {
        for stmt in program {
            if !self.compile_statement(stmt) {
                return false;
            }
        }
        true
    }

    pub fn compile_statement(&mut self, statement: &Statement) -> bool {
        match statement {
            Statement::ExprStmt(e) => {
                if !self.compile_expression(&e.expr) {
                    return false;
                };
                self.push_bytecode(POP_LAST, e.pos);
            }
            Statement::DebugPrint(e) => {
                if !self.compile_expression(&e.expr) {
                    return false;
                };
                self.push_bytecode(DEBUG_PRINT, e.pos);
            }
            Statement::Break(b) => {
                self.push_bytecode(JUMP, b.pos);
                if let Some(patches) = self.break_jump_patches.last_mut() {
                    patches.push(self.bytecodes.len());
                } else {
                    self.compile_error(
                        b.pos,
                        "Break statement outside of loop is not allowed".to_string(),
                    );
                    return false;
                }
                self.push_bytecode(0, b.pos);
            }
            Statement::Next(b) => {
                self.push_bytecode(JUMP, b.pos);
                if let Some(patches) = self.next_jump_patches.last_mut() {
                    patches.push(self.bytecodes.len());
                } else {
                    self.compile_error(
                        b.pos,
                        "Next statement outside of loop is not allowed".to_string(),
                    );
                    return false;
                }
                self.push_bytecode(0, b.pos);
            }
            Statement::PointerAssign(ptr) => {
                if !self.compile_expression(&ptr.ptr) {
                    return false;
                }

                if !self.compile_expression(&ptr.value) {
                    return false;
                }

                self.push_bytecode(SET_IN_PLACE, ptr.pos);
            }
        }
        true
    }

    pub fn compile_expression(&mut self, expression: &Expression) -> bool {
        match expression {
            Expression::String_(s) => {
                if self
                    .constants
                    .try_push(Value::String(s.value.to_string()))
                    .is_err()
                {
                    self.compile_error(
                        s.pos,
                        format!("Constant exceeds limit of {}", CONSTANT_SIZE),
                    );
                    return false;
                }

                self.push_bytecode(LOAD_CONST, s.pos);
                self.push_bytecode(self.constants.len() as Byte - 1, s.pos);
            }

            Expression::Int(num) => {
                let val = num.value.parse::<i64>();
                if let Ok(val) = val {
                    let index;
                    if let Some(k) = self.constant_hash_int.get(&val) {
                        index = *k;
                    } else if self.constants.try_push(Value::Int(val)).is_err() {
                        self.compile_error(
                            num.pos,
                            format!("Constant exceeds limit of {}", CONSTANT_SIZE),
                        );
                        return false;
                    } else {
                        index = self.constants.len() as Byte - 1;
                        self.constant_hash_int.insert(val, index);
                    }
                    self.push_bytecode(LOAD_CONST, num.pos);
                    self.push_bytecode(index, num.pos);
                } else {
                    self.compile_error(num.pos, "Integer literal too large".to_string());
                    return false;
                }
            }

            Expression::Float(num) => {
                let val = num.value.parse::<f64>();
                if let Ok(val) = val {
                    if self.constants.try_push(Value::Float(val)).is_err() {
                        self.compile_error(
                            num.pos,
                            format!("Constant exceeds limit of {}", CONSTANT_SIZE),
                        );
                        return false;
                    }
                    let index = self.constants.len() as Byte - 1;
                    self.push_bytecode(LOAD_CONST, num.pos);
                    self.push_bytecode(index, num.pos);
                } else {
                    self.compile_error(num.pos, "Integer literal too large".to_string());
                    return false;
                }
            }

            Expression::Bool(b) => {
                let index = if b.value {
                    BOOL_TRUE_CONSTANT
                } else {
                    BOOL_FALSE_CONSTANT
                };
                self.push_bytecode(LOAD_CONST, b.pos);
                self.push_bytecode(index as Byte, b.pos);
            }

            Expression::Array(a) => {
                for x in a.values.iter().rev() {
                    self.compile_expression(x);
                }
                self.push_bytecode(MAKE_ARRAY, a.pos);
                self.push_bytecode(a.values.len() as Byte, a.pos);
            }

            Expression::GetVar(get) => {
                let res = self.resolve_local(get.name.to_string());
                if let Some(index) = res {
                    self.push_bytecode(LOAD_LOCAL, get.pos);
                    self.push_bytecode(index as Byte, get.pos);
                } else {
                    self.compile_error(get.pos, format!("Variable '{}' is not defined", get.name));
                    return false;
                }
            }

            Expression::SetVar(var) => {
                let replace = self.add_local(var.name.to_string());

                if !self.compile_expression(&var.value) {
                    return false;
                }

                self.push_bytecode(REPLACE, var.pos);
                self.push_bytecode(replace as Byte, var.pos);
                self.push_bytecode(LOAD_LOCAL, var.pos);
                self.push_bytecode(replace as Byte, var.pos);
            }

            Expression::Infix(infix) => {
                match infix.operator {
                    "&&" => {
                        if !self.compile_expression(&infix.left) {
                            return false;
                        }

                        self.push_bytecode(JUMP_IF_FALSE_NO_POP, infix.pos);
                        let patch_loc = self.bytecodes.len();
                        self.push_bytecode(0, infix.pos);

                        // pop left operand
                        self.push_bytecode(POP_LAST, infix.pos);

                        if !self.compile_expression(&infix.right) {
                            return false;
                        }
                        self.bytecodes[patch_loc] = self.bytecodes.len() as Byte;
                    }
                    "||" => {
                        if !self.compile_expression(&infix.left) {
                            return false;
                        }

                        self.push_bytecode(JUMP_IF_FALSE_NO_POP, infix.pos);
                        let patch_loc_1 = self.bytecodes.len();
                        self.push_bytecode(0, infix.pos);

                        self.push_bytecode(JUMP, infix.pos);
                        let patch_loc_2 = self.bytecodes.len();
                        self.push_bytecode(0, infix.pos);

                        self.bytecodes[patch_loc_1] = self.bytecodes.len() as Byte;

                        self.push_bytecode(POP_LAST, infix.pos);
                        if !self.compile_expression(&infix.right) {
                            return false;
                        }

                        self.bytecodes[patch_loc_2] = self.bytecodes.len() as Byte;
                    }
                    _ => {
                        if !self.compile_expression(&infix.left) {
                            return false;
                        }
                        if !self.compile_expression(&infix.right) {
                            return false;
                        }
                        match infix.operator {
                            "+" => {
                                self.push_bytecode(BINARY_ADD, infix.pos);
                            }
                            "-" => {
                                self.push_bytecode(BINARY_SUB, infix.pos);
                            }
                            "*" => {
                                self.push_bytecode(BINARY_MUL, infix.pos);
                            }
                            "/" => {
                                self.push_bytecode(BINARY_DIV, infix.pos);
                            }
                            "%" => {
                                self.push_bytecode(BINARY_MOD, infix.pos);
                            }

                            "==" => {
                                self.push_bytecode(BINARY_EQ, infix.pos);
                            }
                            "!=" => {
                                self.push_bytecode(BINARY_NE, infix.pos);
                            }
                            "<" => {
                                self.push_bytecode(BINARY_LT, infix.pos);
                            }
                            "<=" => {
                                self.push_bytecode(BINARY_LE, infix.pos);
                            }
                            ">" => {
                                self.push_bytecode(BINARY_GT, infix.pos);
                            }
                            ">=" => {
                                self.push_bytecode(BINARY_GE, infix.pos);
                            }

                            _ => {
                                self.compile_error(
                                    infix.pos,
                                    format!("Unsupported Operand: {}", infix.operator),
                                );
                                return false;
                            }
                        }
                    }
                }
            }

            Expression::Prefix(prefix) => match prefix.operator {
                "-" => {
                    if !self.compile_expression(&prefix.right) {
                        return false;
                    }
                    self.push_bytecode(UNARY_NEG, prefix.pos);
                }
                "!" => {
                    if !self.compile_expression(&prefix.right) {
                        return false;
                    }
                    self.push_bytecode(UNARY_NOT, prefix.pos);
                }
                _ => {
                    self.compile_error(
                        prefix.pos,
                        format!("Unsupported Operand: {}", prefix.operator),
                    );
                    return false;
                }
            },

            Expression::Index(indexing) => {
                if !self.compile_expression(&indexing.callee) {
                    return false;
                }

                if !self.compile_expression(&indexing.index) {
                    return false;
                }

                self.push_bytecode(GET, indexing.pos);
            }

            Expression::If(iff) => {
                // Compile Condition
                if !self.compile_expression(&iff.cond) {
                    return false;
                }

                // Jump to else if false
                self.push_bytecode(JUMP_IF_FALSE, iff.pos);
                let patch_loc = self.bytecodes.len();
                self.push_bytecode(0, iff.pos);

                if !iff.body.is_empty() {
                    // Compile then block
                    self.begin_scope();
                    if !self.compile_program(&iff.body) {
                        return false;
                    }
                    self.end_scope();

                    // If there is a result, don't pop it
                    if self.bytecodes.last() == Some(&POP_LAST) {
                        self.bytecodes.pop();
                        self.lines.pop();
                    } else {
                        self.push_bytecode(LOAD_CONST, iff.pos);
                        self.push_bytecode(NULL_CONSTANT as Byte, iff.pos);
                    }
                } else {
                    self.push_bytecode(LOAD_CONST, iff.pos);
                    self.push_bytecode(NULL_CONSTANT as Byte, iff.pos);
                }

                // Jump to end
                self.push_bytecode(JUMP, iff.pos);
                let patch_loc_2 = self.bytecodes.len();
                self.push_bytecode(0, iff.pos);

                // Patch jump 1
                self.bytecodes[patch_loc] = self.bytecodes.len() as Byte;

                if iff.other.is_empty() {
                    self.push_bytecode(LOAD_CONST, iff.pos);
                    self.push_bytecode(NULL_CONSTANT as Byte, iff.pos);

                    self.bytecodes[patch_loc_2] = self.bytecodes.len() as Byte;
                } else {
                    if !iff.other.is_empty() {
                        // Compile else block
                        self.begin_scope();
                        if !self.compile_program(&iff.other) {
                            return false;
                        }
                        self.end_scope();

                        // If there is a result, don't pop it
                        if self.bytecodes.last() == Some(&POP_LAST) {
                            self.bytecodes.pop();
                            self.lines.pop();
                        } else {
                            self.push_bytecode(LOAD_CONST, iff.pos);
                            self.push_bytecode(NULL_CONSTANT as Byte, iff.pos);
                        }
                    } else {
                        self.push_bytecode(LOAD_CONST, iff.pos);
                        self.push_bytecode(NULL_CONSTANT as Byte, iff.pos);
                    }

                    self.bytecodes[patch_loc_2] = self.bytecodes.len() as Byte;
                }
            }

            Expression::While(w) => {
                self.break_jump_patches.push(Vec::new());
                self.next_jump_patches.push(Vec::new());

                let loop_start = self.bytecodes.len();

                if !self.compile_expression(&w.cond) {
                    return false;
                }

                self.push_bytecode(JUMP_IF_FALSE, w.pos);
                let patch_loc = self.bytecodes.len();
                self.push_bytecode(0, w.pos);

                self.begin_scope();
                if !self.compile_program(&w.body) {
                    return false;
                }
                self.end_scope();

                self.push_bytecode(JUMP, w.pos);
                self.push_bytecode(loop_start as Byte, w.pos);

                self.bytecodes[patch_loc] = self.bytecodes.len() as Byte;

                let list = self.break_jump_patches.pop().unwrap();
                for i in list {
                    self.bytecodes[i] = self.bytecodes.len() as Byte;
                }

                let list = self.next_jump_patches.pop().unwrap();
                for i in list {
                    self.bytecodes[i] = loop_start as Byte;
                }

                self.push_bytecode(LOAD_CONST, w.pos);
                self.push_bytecode(NULL_CONSTANT as Byte, w.pos);
            }

            Expression::Do(d) => {
                self.begin_scope();
                if !self.compile_program(&d.body) {
                    return false;
                }
                self.end_scope();
                if self.bytecodes.last() == Some(&POP_LAST) {
                    self.bytecodes.pop();
                    self.lines.pop();
                } else {
                    self.push_bytecode(LOAD_CONST, d.pos);
                    self.push_bytecode(NULL_CONSTANT as Byte, d.pos);
                }
            }
        }

        true
    }

    pub fn optimize(&mut self) {
        let mut i = 0;
        while i < self.bytecodes.len() {
            let b = self.bytecodes[i];
            if b == LOAD_LOCAL || b == LOAD_CONST {
                // LOAD xxxx POP
                // This has no effect at all, except at end of file.
                if i + 2 < self.bytecodes.len() - 1 && self.bytecodes[i + 2] == POP_LAST {
                    for j in i..=i + 2 {
                        self.bytecodes[j] = NOOP;
                    }
                    i += 3;
                    continue;
                }
            }

            i += 1;
            i += operands(b);
        }
    }

    pub fn disassemble(&self) -> String {
        let mut s = String::new();
        let mut pc = 0;
        while pc < self.bytecodes.len() {
            let old_pc = pc;
            let byte = self.bytecodes[pc];

            if byte == NOOP {
                pc += 1;
                continue;
            }

            let mut args: Vec<String> = vec![];
            match byte {
                LOAD_CONST => {
                    pc += 1;
                    let address = self.bytecodes[pc] as usize;
                    args.push(format!(
                        "{:04x} ({})",
                        address,
                        self.constants[address].debug_format()
                    ));
                }

                LOAD_LOCAL | MAKE_ARRAY | REPLACE | JUMP_IF_FALSE | JUMP_IF_FALSE_NO_POP | JUMP => {
                    pc += 1;
                    let address = self.bytecodes[pc] as usize;
                    args.push(format!("{:04x}", address));
                }

                _ => (),
            }

            s.write_fmt(format_args!(
                "{:04x}: {} {}\n",
                old_pc,
                bytecode_name(byte),
                args.join(", ")
            ))
            .unwrap();

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
        while self.stack.len() > self.current_compiler.count {
            self.stack.pop();
        }
        self.last_popped = None;
        self.pc = 0;
        while self.pc < self.bytecodes.len() {
            unsafe {
                let bc = self.read_bytecode();
                match bc {
                    // General
                    NOOP => {
                        // Do nothing
                    }

                    POP_LAST => {
                        let v = self.stack.pop();
                        if self.repl_mode {
                            self.last_popped = v;
                        }
                    }

                    REPLACE => {
                        let index = self.read_bytecode();
                        let v = self.stack.pop().unwrap();
                        while self.stack.len() <= index as usize {
                            self.stack
                                .push(&mut self.constants[NULL_CONSTANT] as *mut Value);
                        }
                        self.stack[index as usize] = (*v).shallow_copy();
                    }

                    SET_IN_PLACE => {
                        let v = self.stack.pop().unwrap();
                        let p = self.stack.pop().unwrap();
                        (*v).regular_copy_to(p);
                        self.stack.push(v);
                    }

                    LOAD_CONST => {
                        let index = self.read_bytecode();
                        if self
                            .stack
                            .try_push(alloc_new_value(self.constants[index as usize].clone()))
                            .is_err()
                        {
                            self.runtime_error("Stack overflow".to_string());
                            return;
                        }
                    }

                    LOAD_LOCAL => {
                        let index = self.read_bytecode();
                        self.stack.push(*self.stack.get(index as usize).unwrap());
                    }

                    MAKE_ARRAY => {
                        let length = self.read_bytecode() as usize;
                        let mut array = Vec::with_capacity(length);
                        for _ in 0..length {
                            array.push(self.stack.pop().unwrap());
                        }
                        self.stack
                            .push_unchecked(alloc_new_value(Value::Array(array)));
                    }

                    JUMP_IF_FALSE => {
                        let address = self.read_bytecode();
                        if !(*self.stack.pop().unwrap()).is_truthy() {
                            self.pc = address as usize;
                        }
                    }

                    JUMP_IF_FALSE_NO_POP => {
                        let address = self.read_bytecode();
                        if !(*self.stack.pop().unwrap()).is_truthy() {
                            self.pc = address as usize;
                        }
                    }

                    JUMP => {
                        let address = self.read_bytecode();
                        self.pc = address as usize;
                    }

                    DEBUG_PRINT => {
                        let value = self.stack.pop().unwrap();
                        println!("{}", (*value).debug_format());
                    }

                    GET => {
                        let index = self.stack.pop().unwrap();
                        let callee = self.stack.pop().unwrap();

                        let res = (*callee).get_element(index);
                        if res.is_err() {
                            self.runtime_error(res.err().unwrap());
                            return;
                        }

                        self.stack.push(res.unwrap());
                    }

                    // Prefix operators
                    UNARY_NEG => {
                        let value = &*self.stack.pop().unwrap();
                        match value {
                            Value::Bool(_) => {
                                self.runtime_error(
                                    "Unsupported Unary operation: -bool (Hint: Use !bool instead)"
                                        .to_string(),
                                );
                                return;
                            }
                            Value::Int(i) => {
                                // We just popped an element, so there should be an empty space on the stack.
                                self.stack.push_unchecked(alloc_new_value(Value::Int(
                                    i.saturating_neg(),
                                )));
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
                        let value = &*self.stack.pop().unwrap();
                        self.stack
                            .push_unchecked(alloc_new_value(Value::Bool(!value.is_truthy())));
                    }

                    // Infix operators
                    BINARY_ADD => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_add(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} + {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_SUB => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_sub(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} - {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_MUL => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_mul(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} * {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_DIV => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_div(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} / {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_MOD => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_mod(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} % {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_EQ => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        self.stack
                            .push_unchecked(alloc_new_value(Value::Bool(left.is_equal(right))));
                    }

                    BINARY_NE => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        self.stack
                            .push_unchecked(alloc_new_value(Value::Bool(!left.is_equal(right))));
                    }

                    BINARY_LT => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_lt(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} < {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_LE => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_le(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} <= {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_GT => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_gt(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} > {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
                        }
                    }

                    BINARY_GE => {
                        let right = &*self.stack.pop().unwrap();
                        let left = &*self.stack.pop().unwrap();
                        let res = left.binary_ge(right);
                        if let BinOpResult::Ok(res) = res {
                            self.stack.push_unchecked(res);
                        } else if let BinOpResult::Error(e) = res {
                            self.runtime_error(e);
                            return;
                        } else {
                            self.runtime_error(format!(
                                "Unsupported Binary operation: {} >= {}",
                                left.type_name(),
                                right.type_name()
                            ));
                            return;
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
}
