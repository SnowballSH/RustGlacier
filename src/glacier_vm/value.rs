use std::fmt::{Debug, Formatter};
use std::fmt;

use num::BigInt;

use crate::glacier_vm::error::GlacierError;
use crate::glacier_vm::operators::{apply_operator, apply_unary_operator};
use crate::glacier_vm::vm::Heap;

#[derive(Clone)]
pub struct FT(pub fn(this: &Value, arguments: Vec<Value>, heap: &Heap) -> CallResult);

impl PartialEq for FT {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for FT {}

impl Debug for FT {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Native Function")
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    BigInt(BigInt),
    Int(i64),
    NativeFunction(FT),
    String(String),
    Boolean(bool),

    Null,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    BigInt,
    Int,
    NativeFunction,
    String,
    Boolean,

    Null,
}

impl ValueType {
    pub fn to_string(&self) -> String {
        match self {
            ValueType::BigInt => {
                format!("BigInt")
            }
            ValueType::Int => {
                format!("Integer")
            }
            ValueType::NativeFunction => {
                format!("NativeFunction")
            }
            ValueType::String => {
                format!("String")
            }
            ValueType::Boolean => {
                format!("Boolean")
            }
            ValueType::Null => {
                format!("Null")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConvertResult {
    Ok(Value),
    NotOk,
    SameType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApplyOperatorResult {
    Ok(Value),
    NoSuchOperator,
    Error(GlacierError),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CallResult {
    Ok(Value),
    NotCallable,
    Error(GlacierError),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::BigInt(x) => {
                x.to_string()
            }
            Value::Int(x) => {
                x.to_string()
            }
            Value::NativeFunction(x) => {
                format!("{:?}", x)
            }
            Value::String(x) => {
                x.clone()
            }
            Value::Boolean(x) => {
                x.to_string()
            }
            Value::Null => {
                format!("Null")
            }
        }
    }

    pub fn to_debug_string(&self) -> String {
        match self {
            Value::String(x) => {
                format!("\"{}\"", x)
            }
            _ => self.to_string()
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(x) => {
                *x != 0
            }
            Value::Boolean(x) => {
                *x
            }
            Value::Null => false,
            _ => true
        }
    }

    pub fn apply_operator(&self, name: &str, other: &Value) -> ApplyOperatorResult {
        apply_operator(self, name, other)
    }

    pub fn apply_unary_operator(&self, name: &str) -> ApplyOperatorResult {
        apply_unary_operator(self, name)
    }

    #[inline]
    /// Get the type of object
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::BigInt(_) => ValueType::BigInt,
            Value::Int(_) => ValueType::Int,
            Value::NativeFunction(_) => ValueType::NativeFunction,
            Value::String(_) => ValueType::String,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Null => ValueType::Null,
        }
    }

    /// Try to convert self -> into
    pub fn try_convert(&self, into: ValueType) -> ConvertResult {
        if self.value_type() == into {
            return ConvertResult::SameType;
        }
        match self {
            Value::BigInt(_) => ConvertResult::NotOk,
            Value::Int(x) => match into {
                ValueType::BigInt => ConvertResult::Ok(Value::BigInt(BigInt::from(*x))),
                _ => ConvertResult::NotOk,
            },
            Value::NativeFunction(_) => ConvertResult::NotOk,
            Value::String(_) => ConvertResult::NotOk,
            Value::Boolean(_) => ConvertResult::NotOk,
            Value::Null => ConvertResult::NotOk,
        }
    }

    pub fn call(&self, arguments: Vec<Value>, heap: &Heap) -> CallResult {
        match self {
            Value::NativeFunction(x) => {
                x.0(self, arguments, heap)
            }
            _ => CallResult::NotCallable
        }
    }
}

#[cfg(test)]
mod tests {
    use num::BigInt;

    use crate::glacier_vm::value::{ApplyOperatorResult, ConvertResult, Value, ValueType};

    #[test]
    fn test_operator() {
        let a = Value::Int(6);
        let b = Value::Int(8);
        assert_eq!(a.try_convert(ValueType::Int), ConvertResult::SameType);
        assert_eq!(
            b.try_convert(ValueType::BigInt),
            ConvertResult::Ok(Value::BigInt(BigInt::from(8)))
        );
        assert_eq!(a.apply_operator("+", &b), ApplyOperatorResult::Ok(Value::Int(14)));
        assert_eq!(a.apply_operator("???", &b), ApplyOperatorResult::NoSuchOperator);
    }
}
