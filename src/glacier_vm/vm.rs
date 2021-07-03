use std::collections::HashMap;

use crate::glacier_vm::value::Value;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct VM<'a> {
    pub heap: Heap,
    pub variables: HashMap<&'a str, usize>,
}

impl<'a> Default for VM<'a> {
    fn default() -> Self {
        Self {
            heap: Default::default(),
            variables: HashMap::with_capacity(512),
        }
    }
}

impl<'a> VM<'a> {
    pub fn push(&mut self, value: Value) {
        self.heap.push(value);
    }

    pub fn define_variable(&mut self, name: &'a str) {
        self.variables.insert(name, self.heap.length - 1);
    }

    pub fn get_variable(&self, name: &'a str) -> Option<&Value> {
        self.variables
            .get(name)
            .and_then(|x| self.heap.value.get(*x))
    }
}

#[cfg(test)]
mod tests {
    use crate::glacier_vm::value::Value;
    use crate::glacier_vm::vm::VM;
    use num::BigInt;

    #[test]
    fn basic_vm() {
        let mut vm = VM::default();
        assert!(vm.get_variable("abcd").is_none());

        let number = Value::BigInt(BigInt::from(12345678987654321_i128));
        vm.push(number.clone());
        vm.define_variable("abcd");
        assert_eq!(vm.get_variable("abcd"), Some(&number));
    }
}
