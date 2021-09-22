use std::collections::HashMap;

use crate::glacier_vm::builtins::get_builtin;
use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::{ApplyOperatorResult, CallResult, GetInstanceResult, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Heap for Glacier VM.
/// available is the free spots created by GC.
pub struct Heap {
    pub value: Vec<Value>,
    pub available: Vec<usize>,
}

impl Default for Heap {
    fn default() -> Self {
        Self {
            value: Vec::with_capacity(512),
            available: Vec::with_capacity(128),
        }
    }
}

impl Heap {
    pub fn with_capacity(x: usize, y: usize) -> Self {
        Self {
            value: Vec::with_capacity(x),
            available: Vec::with_capacity(y),
        }
    }

    #[inline]
    /// Use a free spot if possible.
    pub fn push_free(&mut self, val: Value) -> usize {
        if let Some(x) = self.available.pop() {
            self.value[x] = val;
            x
        } else {
            self.push(val)
        }
    }

    #[inline]
    /// Push to the back, do not use a free spot.
    pub fn push(&mut self, val: Value) -> usize {
        self.value.push(val);
        self.value.len() - 1
    }

    #[inline]
    /// Returns the last element of the heap.
    pub fn pop(&mut self) -> Value {
        let r = self.value.pop().expect("Stack underflow");
        r
    }

    #[inline]
    /// Releases a location for future variable storage.
    pub fn release(&mut self, pos: usize) {
        self.available.push(pos);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents linking of name to heap location.
/// parent is the outer scope.
pub struct VariableMap {
    pub map: Vec<(HashMap<String, usize>, usize)>,
}

impl Default for VariableMap {
    fn default() -> Self {
        VariableMap {
            map: vec![(HashMap::with_capacity(64), 0)],
        }
    }
}

impl VariableMap {
    pub fn add_child(&mut self, location: usize) {
        self.map.push((HashMap::with_capacity(32), location));
    }

    #[inline]
    pub fn insert(&mut self, key: String, value: usize) {
        self.map.last_mut().unwrap().0.insert(key, value);
    }

    #[inline]
    pub fn get(&self, key: &String) -> Option<&usize> {
        let mut index = self.map.len();
        let mut res = None;
        while index > 0 {
            index -= 1;
            res = self.map[index].0.get(key);
            if res.is_some() {
                break;
            }
        }
        res
    }
}

#[derive(Clone, Debug)]
/// Virtual Machine for Glacier. It contains:
/// Heap: main heap of the VM
/// Stack: stack of the VM, to compute with multiple objects without interrupting the heap.
///        Usually an object is popped from heap and pushed to stack, then dropped after computation.
/// Variables: map of name to location. Also contains previous frames.
/// Last Popped: the last object popped. Useful for repl mode.
/// Last Push Location: location of last object pushed.
/// Error: error during interpretation.
/// Line: current line position.
/// Use Reference: whether to automatically use references. THIS IS EXPERIMENTAL
/// Use GC: whether to use the simple Garbage Collector after function returns.
pub struct VM {
    pub heap: Heap,
    pub stack: Heap,
    pub variables: VariableMap,
    pub last_popped: Option<Value>,
    pub last_push_location: Option<usize>,
    pub error: Option<GlacierError>,
    pub line: usize,
    pub use_reference: bool,
    pub use_gc: bool,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            heap: Heap::default(),
            stack: Heap::with_capacity(32, 16),
            variables: VariableMap::default(),
            last_popped: None,
            last_push_location: None,
            error: None,
            line: 0,
            use_reference: false,
            use_gc: true,
        }
    }
}

impl VM {
    #[inline]
    /// pushes an object to the heap, do not use free spots, updating [self.last_push_location]
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline]
    /// pushes an object to the heap, but use a free spot if possible, updating [self.last_push_location]
    pub fn push_free(&mut self, value: Value) {
        self.last_push_location = Some(self.heap.push_free(value));
    }

    #[inline]
    /// Link [self.last_push_location] to [name]
    pub fn define_variable(&mut self, name: String) {
        self.variables
            .insert(name, self.last_push_location.expect("No value pushed"));
    }

    /// Get a variable and return it.
    pub fn get_variable(&mut self, name: String) -> Option<&Value> {
        let res = self.variables.get(&name);
        if let Some(x) = res {
            return self.heap.value.get(*x);
        } else {
            let b = get_builtin(name.clone());
            if let Some(b) = b {
                self.push_free(b);
                self.define_variable(name.clone());
                self.get_variable(name.clone())
            } else {
                None
            }
        }
    }

    /// Get a variable and return it.
    pub fn get_variable_location(&mut self, name: String) -> Option<usize> {
        let res = self.variables.get(&name);
        if let Some(x) = res {
            return Some(*x);
        } else {
            let b = get_builtin(name.clone());
            if let Some(b) = b {
                self.push(b);
                self.define_variable(name.clone());
                self.last_push_location
            } else {
                None
            }
        }
    }

    #[inline]
    /// Creates a new frame/scope
    pub fn new_frame(&mut self, index: usize) {
        self.last_popped = None;
        self.variables.add_child(index);
    }

    #[inline]
    /// Exits the nearest scope
    pub fn exit_frame(&mut self) -> usize {
        if !self.use_reference {
            let m = self.variables.map.pop().unwrap();
            for x in m.0 {
                self.heap.release(x.1);
            }
            let u = m.1;
            u
        } else {
            let m = self.variables.map.pop().unwrap();
            let u = m.1;
            u
        }
    }

    /// Runs the instructions.
    pub fn run(&mut self, instructions: Vec<Instruction>) {
        self.run_with_start(instructions, 0, 0);
    }

    /// Runs the instructions with starting value.
    pub fn run_with_start(&mut self, instructions: Vec<Instruction>, start: usize, padding: isize) {
        let mut index = start;
        let l = instructions.len();
        while index < l {
            let i = &instructions[index];

            match i {
                Instruction::Push(x) => {
                    self.push(x.clone());
                }
                Instruction::PushVar(x, n) => {
                    self.push_free(x.clone());
                    self.define_variable(n.clone());
                }
                Instruction::Pop => {
                    self.last_popped = Some(self.stack.pop());
                }
                Instruction::Move((from, to)) => {
                    self.stack.value[*to] = self
                        .stack
                        .value
                        .get(*from)
                        .expect("Move not in range")
                        .clone();
                }
                Instruction::MovePush(from) => {
                    self.push(
                        self.stack
                            .value
                            .get(*from)
                            .expect("Move not in range")
                            .clone(),
                    );
                }
                Instruction::MoveLastFromHeapToStack => {
                    self.push(self.heap.value.last().expect("Empty heap").clone());
                }
                Instruction::MoveVar(name) => {
                    if self.use_reference {
                        if let Some(m) = self.get_variable_location(name.to_string()) {
                            self.push(Value::Reference(m));
                        } else {
                            self.error = Some(ErrorType::UndefinedVariable(name.to_string()));
                            return;
                        }
                    } else {
                        if let Some(m) = self.get_variable(name.to_string()).cloned() {
                            self.push(m);
                        } else {
                            self.error = Some(ErrorType::UndefinedVariable(name.to_string()));
                            return;
                        }
                    }
                }
                Instruction::Var(x) => {
                    let v = self.stack.pop();
                    self.push_free(v);
                    self.define_variable(x.clone());
                }

                Instruction::BinaryOperator(x) => {
                    // b x a
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let res = b.apply_operator(x, &a, &self.heap);
                    if let ApplyOperatorResult::Ok(y) = res {
                        self.push(y);
                    } else if let ApplyOperatorResult::Error(e) = res {
                        self.error = Some(e);
                        return;
                    } else {
                        self.error = Some(ErrorType::InvalidBinaryOperation(b, x.to_string(), a));
                        return;
                    }
                }

                Instruction::UnaryOperator(x) => {
                    let a = self.stack.pop();
                    let res = a.apply_unary_operator(x, &self.heap);
                    if let ApplyOperatorResult::Ok(y) = res {
                        self.push(y);
                    } else if let ApplyOperatorResult::Error(e) = res {
                        self.error = Some(e);
                        return;
                    } else {
                        self.error = Some(ErrorType::InvalidUnaryOperation(a, x.to_string()));
                        return;
                    }
                }

                Instruction::Call(x) => {
                    let mut callee = self.stack.pop();
                    while let Value::Reference(addr) = callee {
                        callee = self.heap.value[addr].clone()
                    }
                    if let Value::GlacierFunction(idx, name, params) = callee {
                        if *x != params.len() {
                            self.error = Some(GlacierError::ArgumentError(format!(
                                "When Calling Function {}: Expected {} arguments, got {}",
                                name,
                                params.len(),
                                *x
                            )));
                            return;
                        }

                        // enter a new scope
                        self.new_frame(index);

                        for s in params {
                            let k = self.stack.pop();
                            self.push_free(k);
                            self.define_variable(s);
                        }

                        index = idx - 1;
                    } else {
                        let mut arguments = vec![];
                        for _ in 0..*x {
                            arguments.push(self.stack.pop());
                        }

                        let res = callee.call(arguments, &self.heap);
                        match res {
                            CallResult::Ok(x) => {
                                self.push(x);
                            }
                            CallResult::NotCallable => {
                                self.error = Some(ErrorType::NotCallable(callee.value_type()));
                                return;
                            }
                            CallResult::Error(e) => {
                                self.error = Some(e);
                                return;
                            }
                        }
                    }
                }

                Instruction::GetInstance(x) => {
                    let p = self.stack.pop();
                    let r = p.get_instance(x, &self.stack);
                    if let GetInstanceResult::Ok(k) = r {
                        self.push(k);
                    } else if let GetInstanceResult::Error(e) = r {
                        self.error = Some(e);
                        return;
                    } else {
                        self.error = Some(ErrorType::NoInstance(p, x.to_string()));
                        return;
                    }
                }

                Instruction::MakeCode(x, y, z) => {
                    self.push_free(Value::GlacierFunction(
                        *x + padding as usize,
                        y.clone(),
                        z.clone(),
                    ));
                    self.define_variable(y.clone());
                }

                Instruction::Ret => {
                    if self.variables.map.len() == 1 {
                        return;
                    }
                    index = self.exit_frame()
                }

                Instruction::Jump(x) => {
                    index = (*x as isize + padding) as usize;
                    continue;
                }

                Instruction::JumpIfFalse(x) => {
                    if !self.stack.pop().is_truthy(&self.stack) {
                        index = (*x as isize + padding) as usize;
                        continue;
                    }
                }

                Instruction::MoveLastToHeap => {
                    self.heap.push(self.stack.pop());
                }

                Instruction::Noop => {}
                Instruction::ToggleRef => {
                    self.use_reference = !self.use_reference;
                }

                Instruction::SetLine(x) => {
                    self.line = *x;
                }
            }

            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {}
