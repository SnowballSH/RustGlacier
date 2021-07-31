use std::collections::HashMap;

use crate::glacier_vm::builtins::get_builtin;
use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::{ApplyOperatorResult, CallResult, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Heap {
    pub value: Vec<Value>,
    pub length: usize,
}

impl Default for Heap {
    fn default() -> Self {
        Self {
            value: Vec::with_capacity(512),
            length: 0,
        }
    }
}

impl Heap {
    pub fn push(&mut self, val: Value) {
        self.value.push(val);
        self.length += 1;
    }

    pub fn pop(&mut self) -> Value {
        let r = self.value.pop().expect("Stack underflow");
        self.length -= 1;
        r
    }
}

#[derive(Clone, Debug)]
pub struct VM {
    pub heap: Heap,
    pub stack: Heap,
    pub variables: HashMap<String, usize>,
    pub last_popped: Option<Value>,
    pub error: Option<GlacierError>,
    pub line: usize,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            heap: Default::default(),
            stack: Default::default(),
            variables: HashMap::with_capacity(512),
            last_popped: None,
            error: None,
            line: 0,
        }
    }
}

impl VM {
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.heap.push(value);
    }

    #[inline]
    pub fn define_variable(&mut self, name: String) {
        self.variables.insert(name, self.heap.length - 1);
    }

    #[inline]
    pub fn get_variable(&mut self, name: String) -> Option<&Value> {
        let res = self.variables
            .get(&name);
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

    pub fn run(&mut self, instructions: Vec<Instruction>) {
        let mut index = 0;
        let l = instructions.len();
        while index < l {
            let i = &instructions[index];

            // dbg!(&i);

            match i {
                Instruction::Push(x) => {
                    self.push(x.clone());
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
                        self.error = Some(ErrorType::InvalidBinaryOperation(
                            b,
                            x.to_string(),
                            a,
                        ));
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
                        self.error = Some(ErrorType::InvalidUnaryOperation(
                            a,
                            x.to_string(),
                        ));
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

                Instruction::Jump(x) => {
                    index = *x;
                }

                Instruction::JumpIfFalse(x) => {
                    if !self.heap.pop().is_truthy() {
                        index = *x;
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

        assert_eq!(vm.error, Some(ErrorType::UndefinedVariable("bbc".to_string())));
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
    fn if_else() {

    }
}
