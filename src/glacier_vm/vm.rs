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
    pub map: HashMap<String, usize>,
    pub parent: Option<Box<VariableMap>>,
}

impl Default for VariableMap {
    fn default() -> Self {
        VariableMap {
            map: HashMap::with_capacity(256),
            parent: None,
        }
    }
}

impl VariableMap {
    pub fn child_of(parent: VariableMap) -> Self {
        VariableMap {
            map: HashMap::with_capacity(128),
            parent: Some(Box::new(parent)),
        }
    }

    #[inline]
    pub fn insert(&mut self, key: String, value: usize) {
        self.map.insert(key, value);
    }

    #[inline]
    pub fn get(&self, key: &String) -> Option<&usize> {
        let res = self.map.get(key);
        if res.is_some() {
            res
        } else {
            if let Some(parent) = &self.parent {
                parent.get(key)
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Debug)]
/// Virtual Machine for Glacier. It contains:
/// Heap: main heap of the VM
/// Stack: stack of the VM, to compute with multiple objects without interrupting the heap.
///        Usually an object is popped from heap and pushed to stack, then dropped after computation.
/// Variables: map of name to location.
/// Last Popped: the last object popped. Useful for repl mode.
/// Last Push Location: location of last object pushed.
/// Error: error during interpretation.
/// Line: current line position.
pub struct VM {
    pub heap: Heap,
    pub stack: Heap,
    pub variables: VariableMap,
    pub last_popped: Option<Value>,
    pub last_push_location: Option<usize>,
    pub error: Option<GlacierError>,
    pub line: usize,
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
        }
    }
}

impl VM {
    #[inline]
    /// pushes an object to the heap, do not use free spots, updating [self.last_push_location]
    pub fn push(&mut self, value: Value) {
        self.last_push_location = Some(self.heap.push(value));
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
                self.push(b);
                self.define_variable(name.clone());
                self.get_variable(name.clone())
            } else {
                None
            }
        }
    }

    #[inline]
    /// Creates a new frame/scope
    pub fn new_frame(&mut self) {
        self.variables = VariableMap::child_of(self.variables.clone());
    }

    #[inline]
    /// Exits the nearest scope
    pub fn exit_frame(&mut self) {
        let m = self.variables.clone();
        for x in m.map {
            self.heap.release(x.1);
        }
        self.variables = *m.parent.unwrap();
    }

    /// Runs the instructions.
    pub fn run(&mut self, instructions: Vec<Instruction>) {
        let mut index = 0;
        let l = instructions.len();
        while index < l {
            let i = &instructions[index];

            match i {
                Instruction::Push(x) => {
                    // Look forward to see if t here is a push
                    // TODO there is probably a better way. Maybe use a unique instruction for Push + Var?
                    if let Some(Instruction::Var(_)) = instructions.get(index + 1) {
                        self.push_free(x.clone());
                    } else {
                        self.push(x.clone());
                    }
                }
                Instruction::Pop => {
                    self.last_popped = Some(self.heap.pop());
                }
                Instruction::Move((from, to)) => {
                    self.heap.value[*to] = self
                        .heap
                        .value
                        .get(*from)
                        .expect("Move not in range")
                        .clone();
                }
                Instruction::MovePush(from) => {
                    self.push(
                        self.heap
                            .value
                            .get(*from)
                            .expect("Move not in range")
                            .clone(),
                    );
                }
                Instruction::MoveLast => {
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
                    self.define_variable(x.to_string());
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
                    let callee = self.heap.pop();
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

                Instruction::GetInstance(x) => {
                    let p = self.heap.pop();
                    let r = p.get_instance(x);
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

                Instruction::Jump(x) => {
                    index = *x;
                    continue;
                }

                Instruction::JumpIfFalse(x) => {
                    if !self.heap.pop().is_truthy() {
                        index = *x;
                        continue;
                    }
                }

                Instruction::MoveLastToStack => {
                    self.stack.push(self.heap.pop());
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
mod tests {
    use num::BigInt;

    use crate::glacier_vm::error::{ErrorType, GlacierError};
    use crate::glacier_vm::instructions::Instruction::*;
    use crate::glacier_vm::value::Value;
    use crate::glacier_vm::vm::VM;

    #[test]
    fn basic_vm() {
        let mut vm = VM::default();
        assert!(vm.get_variable("abcd".to_string()).is_none());

        let number = Value::BigInt(BigInt::from(12345678987654321_i128));
        vm.push(number.clone());
        vm.define_variable("abcd".to_string());
        assert_eq!(vm.get_variable("abcd".to_string()), Some(&number));
    }

    #[test]
    fn instructions() {
        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(123454321)),
            Push(Value::Int(987656789)),
            MovePush(0),
            Move((1, 0)),
            Var("onefiveone"),
            MovePush(0),
            Var("ninefivenine"),
            MoveVar("onefiveone"),
        ]);

        assert_eq!(vm.heap.value.last().unwrap(), &Value::Int(123454321));
    }

    #[test]
    fn variables() {
        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(123454321)),
            Var("abc"),
            MoveVar("abc"),
            Pop,
        ]);

        assert_eq!(vm.last_popped, Some(Value::Int(123454321)));
        assert!(vm.error.is_none());

        vm.run(vec![MoveVar("bbc"), Pop]);

        assert_eq!(
            vm.error,
            Some(ErrorType::UndefinedVariable("bbc".to_string()))
        );
    }

    #[test]
    fn frames() {
        let mut vm = VM::default();

        // abc = 12345
        vm.run(vec![Push(Value::Int(12345)), Var("abc")]);

        // {
        vm.new_frame();

        // abc  # 12345 because no change
        vm.run(vec![MoveVar("abc"), Pop]);
        assert!(vm.error.is_none());
        assert_eq!(vm.last_popped, Some(Value::Int(12345)));

        // abc = 123
        vm.run(vec![Push(Value::Int(123)), Var("abc")]);

        // abc  # now abc is 123 in this scope
        vm.run(vec![MoveVar("abc"), Pop]);
        assert!(vm.error.is_none());
        assert_eq!(vm.last_popped, Some(Value::Int(123)));

        // xyz = 543  # frame-scoped
        vm.run(vec![Push(Value::Int(543)), Var("xyz")]);

        // }
        vm.exit_frame();

        // abc  # 12345 because abc is changed to 123 only in the frame
        vm.run(vec![MoveVar("abc"), Pop]);
        assert!(vm.error.is_none());
        assert_eq!(vm.last_popped, Some(Value::Int(12345)));

        // Should have two free spots: abc and xyz
        assert_eq!(vm.heap.available.len(), 2);

        // ii = 8000  # should have a free spot
        vm.run(vec![Push(Value::Int(8000)), Var("ii")]);

        assert_eq!(vm.heap.available.len(), 1);
        // ii would take 1 or 2, technically randomly due to hashmap.
        assert!(*vm.variables.get(&"ii".to_string()).unwrap() >= 1);

        vm.run(vec![MoveVar("xyz"), Pop]);
        // does not exist in outer scope.
        assert!(vm.error.is_some());
    }

    #[test]
    fn binop() {
        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(-20)),
            MoveLastToStack,
            Push(Value::Int(10)),
            MoveLastToStack,
            BinaryOperator("+"),
            Pop,
        ]);

        assert_eq!(vm.last_popped, Some(Value::Int(-10)));
        assert!(vm.error.is_none());

        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(10)),
            MoveLastToStack,
            Push(Value::Int(0)),
            MoveLastToStack,
            BinaryOperator("/"),
            Pop,
        ]);

        assert_eq!(vm.error, Some(GlacierError::ZeroDivisionOrModulo));

        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(-20)),
            MoveLastToStack,
            Push(Value::Int(10)),
            MoveLastToStack,
            BinaryOperator("???"),
            Pop,
        ]);

        assert_eq!(
            vm.error,
            Some(ErrorType::InvalidBinaryOperation(
                Value::Int(-20),
                "???".to_string(),
                Value::Int(10),
            ))
        );

        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(-5)),
            MoveLastToStack,
            UnaryOperator("-"),
            Pop,
        ]);

        assert_eq!(vm.last_popped, Some(Value::Int(5)));
        assert!(vm.error.is_none());

        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(-5)),
            MoveLastToStack,
            UnaryOperator("???"),
            Pop,
        ]);

        assert_eq!(
            vm.error,
            Some(ErrorType::InvalidUnaryOperation(
                Value::Int(-5),
                "???".to_string(),
            ))
        );
    }

    #[test]
    fn if_else() {}
}
