use std::collections::HashMap;

use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::instructions::Instruction;
use crate::glacier_vm::value::Value;

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
        let r = self.value.pop();
        self.length -= 1;
        r.expect("Stack underflow")
    }
}

#[derive(Clone, Debug)]
pub struct VM<'a> {
    pub heap: Heap,
    pub variables: HashMap<String, usize>,
    pub last_popped: Option<Value>,
    pub error: Option<GlacierError<'a>>,
    pub line: usize,
}

impl<'a> Default for VM<'a> {
    fn default() -> Self {
        Self {
            heap: Default::default(),
            variables: HashMap::with_capacity(512),
            last_popped: None,
            error: None,
            line: 0,
        }
    }
}

impl<'a> VM<'a> {
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.heap.push(value);
    }

    #[inline]
    pub fn define_variable(&mut self, name: String) {
        self.variables.insert(name, self.heap.length - 1);
    }

    #[inline]
    pub fn get_variable(&self, name: String) -> Option<&Value> {
        self.variables
            .get(&name)
            .and_then(|x| self.heap.value.get(*x))
    }

    pub fn run(&mut self, instructions: Vec<Instruction<'a>>) {
        for i in instructions {
            match i {
                Instruction::Push(x) => {
                    self.push(x);
                }
                Instruction::Pop => {
                    self.last_popped = Some(self.heap.pop());
                }
                Instruction::Move((from, to)) => {
                    self.heap.value[to] = self
                        .heap
                        .value
                        .get(from)
                        .expect("Move not in range")
                        .clone();
                }
                Instruction::MovePush(from) => {
                    self.push(
                        self.heap
                            .value
                            .get(from)
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
                        self.error = Some(ErrorType::UndefinedVariable(name));
                        return;
                    }
                }
                Instruction::Var(x) => {
                    self.define_variable(x.to_string());
                }

                Instruction::BinaryOperator(x) => {
                    // b x a
                    let a = self.heap.pop();
                    let b = self.heap.pop();
                    let res = b.apply_operator(x, &a);
                    if let Some(y) = res {
                        self.push(y);
                    } else {
                        self.error = Some(ErrorType::InvalidBinaryOperation(
                            b.value_type(),
                            x,
                            a.value_type(),
                        ));
                        return;
                    }
                }

                Instruction::SetLine(x) => {
                    self.line = x;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num::BigInt;

    use crate::glacier_vm::error::ErrorType;
    use crate::glacier_vm::instructions::Instruction::*;
    use crate::glacier_vm::value::{Value, ValueType};
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

        assert_eq!(vm.error, Some(ErrorType::UndefinedVariable("bbc")));
    }

    #[test]
    fn binop() {
        let mut vm = VM::default();

        vm.run(vec![
            Push(Value::Int(-20)),
            Push(Value::Int(10)),
            BinaryOperator("+"),
            Pop,
        ]);

        assert_eq!(vm.last_popped, Some(Value::Int(-10)));
        assert!(vm.error.is_none());

        vm.run(vec![
            Push(Value::Int(-20)),
            Push(Value::Int(10)),
            BinaryOperator("???"),
            Pop,
        ]);

        assert_eq!(
            vm.error,
            Some(ErrorType::InvalidBinaryOperation(
                ValueType::Int,
                "???",
                ValueType::Int
            ))
        );
    }
}
