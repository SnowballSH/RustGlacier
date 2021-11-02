use crate::glacier_vm::builtins::get_builtin;
use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::{ApplyOperatorResult, CallResult, GetPropertyResult, Value};

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
    pub fn push_free(&mut self, val: Value) -> (usize, &Value) {
        if let Some(x) = self.available.pop() {
            self.value[x] = val;
            (x, unsafe { self.value.get_unchecked(x) })
        } else {
            self.push(val)
        }
    }

    #[inline]
    /// Push to the back, do not use a free spot.
    pub fn push(&mut self, val: Value) -> (usize, &Value) {
        self.value.push(val);
        (self.value.len() - 1, self.value.last().unwrap())
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

type VariableDefinition = Option<usize>;

pub const VAR_SIZE: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Represents linking of name to heap location.
pub struct VariableMap {
    pub variables: Vec<Vec<VariableDefinition>>,
}

impl Default for VariableMap {
    fn default() -> Self {
        VariableMap {
            variables: Vec::with_capacity(32),
        }
    }
}

impl VariableMap {
    pub fn insert_map(map: &mut Vec<VariableDefinition>, key: usize, value: usize) {
        while map.len() <= key {
            map.push(None);
        }
        map[key] = Some(value);
    }

    #[inline]
    pub fn insert(&mut self, key: usize, value: usize) {
        Self::insert_map(self.variables.last_mut().unwrap(), key, value);
    }

    #[inline]
    pub fn get(&self, key: &usize) -> Option<usize> {
        let items = self.variables.last().unwrap();
        if let Some(x) = items.get(*key) {
            return *x;
        }
        None
    }

    pub fn release(&mut self) {
        self.variables.pop();
    }

    pub fn new_frame(&mut self) {
        self.variables.push(Vec::new());
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FrameItem {
    pub position: usize,
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
    /// Free variables
    pub frees: VariableMap,
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
    /// whether to use the simple Garbage Collector after function returns.
    pub use_gc: bool,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            heap: Heap::default(),
            stack: Heap::with_capacity(32, 16),
            variables: VariableMap::default(),
            frees: VariableMap::default(),
            last_popped: None,
            last_push_location: None,
            error: None,
            line: 0,
            frames: vec![FrameItem { position: 0 }],
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
    pub fn push_free(&mut self, value: Value) -> &Value {
        let (l, r) = self.heap.push_free(value);
        self.last_push_location = Some(l);
        r
    }

    #[inline]
    /// Link [self.last_push_location] to [name]
    pub fn define_variable(&mut self, name: usize) {
        self.variables
            .insert(name, self.last_push_location.expect("No value pushed"));
    }

    /// Get a variable and return it.
    pub fn get_variable(&mut self, name: String) -> Option<&Value> {
        let b = get_builtin(name.clone());
        if let Some(b) = b {
            let r = self.push_free(b);
            Some(r)
        } else {
            None
        }
    }

    /// Get a variable from int index and return it.
    pub fn get_variable_int(&mut self, name: usize) -> Option<&Value> {
        let res = self.variables.get(&name);
        if let Some(x) = res {
            return self.heap.value.get(x);
        } else {
            None
        }
    }

    #[inline]
    /// Creates a new frame/scope
    pub fn new_frame(&mut self, index: usize) {
        self.variables.new_frame();
        self.frees.new_frame();
        self.frames.push(FrameItem {
            position: index,
        });
    }

    #[inline]
    /// Exits the nearest scope
    pub fn exit_frame(&mut self) -> FrameItem {
        self.variables.release();
        self.frees.release();
        self.frames.pop().expect("No frame to pop")
    }

    /// Runs the instructions.
    pub fn run(&mut self, instructions: Vec<Instruction>) {
        self.variables.new_frame();
        self.run_with_start(instructions, 0, 0);
        self.variables.release();
    }

    /// Runs the instructions with starting value.
    /// Please ensure [self.variables] is initialized with new_frame()
    pub fn run_with_start(&mut self, instructions: Vec<Instruction>, start: usize, padding: isize) {
        // println!("{}", format_instructions(instructions.clone()));
        let mut index = start;
        let l = instructions.len();
        while index < l {
            let i = &instructions[index];

            match i {
                Instruction::Push(x) => {
                    self.push(x.clone());
                }

                Instruction::Pop => {
                    self.last_popped = Some(self.stack.pop());
                }
                Instruction::Move((from, to)) => {
                    self.stack.value[*to] = self.stack.value.get(*from).unwrap().clone()
                }
                Instruction::MovePush(from) => {
                    self.push(self.stack.value.get(*from).unwrap().clone());
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

                Instruction::MoveFree(name) => {
                    self.push(
                        self.heap
                            .value
                            .get(self.frees.get(name).unwrap())
                            .unwrap()
                            .clone(),
                    );
                }

                Instruction::Link(a, b, c) => {
                    VariableMap::insert_map(
                        self.frees.variables.last_mut().unwrap(),
                        *c,
                        self.variables.variables[*a][*b].unwrap(),
                    );
                    let k = unsafe {
                        self.heap
                            .value
                            .get_unchecked(self.variables.variables[*a][*b].unwrap())
                            .clone()
                    };
                    self.push(k);
                }

                Instruction::MoveLocal(name) => {
                    if let Some(m) = self.get_variable_int(*name).cloned() {
                        self.push(m);
                    } else {
                        self.error = Some(ErrorType::UndefinedVariable(name.to_string()));
                        return;
                    }
                }

                Instruction::PushLocal(x, n) => {
                    self.push_free(x.clone());
                    self.define_variable(*n);
                }
                Instruction::DefineLocal(x) => {
                    let v = self.stack.pop();
                    self.push_free(v);
                    self.define_variable(*x);
                }

                Instruction::BinaryOperator(x) => {
                    // b x a
                    let a = self.stack.pop();
                    let b = self.stack.pop();
                    let res = b.apply_operator(x, &a);
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
                    let res = a.apply_unary_operator(x);
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
                    let callee = self.stack.pop();
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

                Instruction::GetProperty(x) => {
                    let p = self.stack.pop();
                    let r = p.get_property(x, &mut self.heap);
                    if let GetPropertyResult::Ok(k) = r {
                        self.push(k);
                    } else if let GetPropertyResult::Error(e) = r {
                        self.error = Some(e);
                        return;
                    } else {
                        self.error = Some(ErrorType::NoProperty(p, x.to_string()));
                        return;
                    }
                }

                Instruction::MakeCode(x, y, z, l) => {
                    self.push_free(Value::GlacierFunction(
                        *x + padding as usize,
                        y.clone(),
                        z.clone(),
                    ));
                    self.define_variable(*l);
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
                    if !self.stack.pop().is_truthy() {
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
