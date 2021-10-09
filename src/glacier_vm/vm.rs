use crate::glacier_vm::builtins::get_builtin;
use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::{ApplyOperatorResult, CallResult, GetInstanceResult, Value};
use rand::rngs::mock::StepRng;
use rand::RngCore;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Heap for Glacier VM.
/// available is the free spots created by GC.
pub struct Heap {
    pub value: Vec<Value>,
    pub available: Vec<usize>,
}

impl Default for Heap {
    fn default() -> Self {
        Self::with_capacity(32, 32)
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
pub struct VariableDefinition {
    pub name: String,
    pub heap_index: usize,
    pub frame_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents linking of name to heap location.
/// parent is the outer scope.
pub struct VariableMap {
    pub variables: Vec<VariableDefinition>,
}

impl Default for VariableMap {
    fn default() -> Self {
        VariableMap {
            variables: Vec::with_capacity(32),
        }
    }
}

impl VariableMap {
    #[inline]
    pub fn insert(&mut self, key: String, value: usize, frame_id: u64) {
        self.variables.push(VariableDefinition {
            name: key,
            heap_index: value,
            frame_id,
        });
    }

    #[inline]
    pub fn get(&self, key: &String) -> Option<usize> {
        for item in self.variables.iter().rev() {
            if &item.name == key {
                return Some(item.heap_index);
            }
        }
        None
    }

    pub fn release(&mut self, id: u64) {
        let mut to_remove = vec![];
        for (i, item) in self.variables.iter().enumerate().rev() {
            if item.frame_id == id {
                to_remove.push(i);
            } else {
                break;
            }
        }
        for i in to_remove {
            self.variables.remove(i);
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FrameItem {
    pub position: usize,
    pub id: u64,
}

#[derive(Clone, Debug)]
/// Virtual Machine for Glacier.
pub struct VM {
    /// main heap of the VM
    pub heap: Heap,
    /// stack of the VM, to compute with multiple objects without interrupting the heap.
    ///        Usually an object is popped from heap and pushed to stack, then dropped after computation.
    pub stack: Heap,
    /// map of name to location. Also contains previous frames.
    pub variables: VariableMap,
    /// the last object popped. Useful for repl mode.
    pub last_popped: Option<Value>,
    /// location of last object pushed.
    pub last_push_location: Option<usize>,
    /// error during interpretation.
    pub error: Option<GlacierError>,
    /// current line position.
    pub line: usize,
    /// frame stack
    pub frames: Vec<FrameItem>,
    /// frame id rng
    pub frame_id_rng: StepRng,
    /// whether to use the simple Garbage Collector after function returns.
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
            frames: vec![FrameItem { position: 0, id: 0 }],
            frame_id_rng: StepRng::new(1, 1),
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
        self.variables.insert(
            name,
            self.last_push_location.expect("No value pushed"),
            self.frames.last().unwrap().id,
        );
    }

    /// Get a variable and return it.
    pub fn get_variable(&mut self, name: String) -> Option<&Value> {
        let res = self.variables.get(&name);
        if let Some(x) = res {
            return self.heap.value.get(x);
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

    /// Get a variable and return its location.
    pub fn get_variable_location(&mut self, name: String) -> Option<usize> {
        let res = self.variables.get(&name);
        if let Some(x) = res {
            return Some(x);
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
        self.frames.push(FrameItem {
            position: index,
            id: self.frame_id_rng.next_u64(),
        });
        // println!("ENTER FRAME: {}", &self.frames.last().unwrap().id);
    }

    #[inline]
    /// Exits the nearest scope
    pub fn exit_frame(&mut self) -> FrameItem {
        // println!("EXIT FRAME: POP {}", &self.frames.last().unwrap().id);
        self.variables.release(self.frames.last().unwrap().id);
        self.frames.pop().expect("No frame to pop")
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
                    if let Some(m) = self.get_variable(name.to_string()).cloned() {
                        self.push(m);
                    } else {
                        self.error = Some(ErrorType::UndefinedVariable(name.to_string()));
                        return;
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
                    if self.variables.variables.len() == 1 {
                        return;
                    }
                    index = self.exit_frame().position;
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
