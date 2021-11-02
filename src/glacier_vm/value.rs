use std::fmt;
use std::fmt::{Debug, Formatter};

use num::{BigInt, Integer, One, Zero};

use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::operators::{apply_operator, apply_unary_operator};

#[derive(Clone)]
pub struct FT(pub fn(this: &Value, arguments: Vec<Value>) -> CallResult);

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

#[derive(Clone)]
pub struct NModule {
    pub name: String,
    pub get_instance: fn(name: &str) -> GetInstanceResult,
}

impl PartialEq for NModule {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for NModule {}

impl Debug for NModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&*format!("Native Module {}", self.name))
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    BigInt(BigInt),
    Int(i64),
    NativeFunction(FT),
    NativeModule(NModule),
    GlacierFunction(usize, String, Vec<usize>),
    String(String),
    Boolean(bool),

    Vector(Vec<usize>),

    Null,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    BigInt,
    Int,
    NativeFunction,
    NativeModule,
    GlacierFunction,
    String,
    Boolean,

    Vector,

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
            ValueType::NativeModule => {
                format!("NativeModule")
            }
            ValueType::GlacierFunction => {
                format!("GlacierFunction")
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
            ValueType::Vector => {
                format!("Vector")
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
pub enum GetInstanceResult {
    Ok(Value),
    NoSuchInstance,
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
            Value::BigInt(x) => x.to_string(),
            Value::Int(x) => x.to_string(),
            Value::NativeFunction(x) => {
                format!("{:?}", x)
            }
            Value::NativeModule(x) => {
                format!("{:?}", x)
            }
            Value::GlacierFunction(_, y, _) => {
                format!("Glacier Function {} {:p}", y, self)
            }
            Value::String(x) => x.clone(),
            Value::Boolean(x) => x.to_string(),
            Value::Vector(x) => {
                format!("Vector [{}]", x.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))
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
            Value::Vector(x) => {
                format!("Vector [{}]", x.iter().map(|x| x.to_debug_string()).collect::<Vec<String>>().join(", "))
            }
            _ => self.to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(x) => *x != 0,
            Value::Boolean(x) => *x,
            Value::Null => false,
            _ => true,
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
            Value::NativeModule(_) => ValueType::NativeModule,
            Value::GlacierFunction(..) => ValueType::GlacierFunction,
            Value::String(_) => ValueType::String,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Vector(_) => ValueType::Vector,
            Value::Null => ValueType::Null,
        }
    }

    /// Try to convert self -> into
    pub fn try_convert(&self, into: ValueType) -> ConvertResult {
        if self.value_type() == into {
            return ConvertResult::SameType;
        }
        match self {
            Value::Int(x) => match into {
                ValueType::BigInt => ConvertResult::Ok(Value::BigInt(BigInt::from(*x))),
                ValueType::Boolean => ConvertResult::Ok(Value::Boolean(*x != 0)),
                _ => ConvertResult::NotOk,
            },
            Value::Boolean(x) => match into {
                ValueType::Int => ConvertResult::Ok(Value::Int(*x as i64)),
                _ => ConvertResult::NotOk,
            },
            _ => ConvertResult::NotOk,
        }
    }

    pub fn get_instance(&self, name: &str) -> GetInstanceResult {
        if let Value::NativeModule(nm) = self {
            return (nm.get_instance)(name);
        }
        match name {
            "b" => GetInstanceResult::Ok(Value::Boolean(self.is_truthy())),
            "s" => GetInstanceResult::Ok(Value::String(self.to_string())),
            "r" => GetInstanceResult::Ok(Value::String(self.to_debug_string())),
            _ => match self {
                Value::String(s) => match name {
                    "i" => {
                        let try_ = s.parse::<i64>();
                        if let Ok(x) = try_ {
                            GetInstanceResult::Ok(Value::Int(x))
                        } else {
                            GetInstanceResult::Error(ErrorType::ConversionError(format!(
                                "Failed to parse {:?} to 64-bit integer",
                                s
                            )))
                        }
                    }
                    "i?" => {
                        let try_ = s.parse::<i64>();
                        if let Ok(_) = try_ {
                            GetInstanceResult::Ok(Value::Boolean(true))
                        } else {
                            GetInstanceResult::Ok(Value::Boolean(false))
                        }
                    }
                    _ => GetInstanceResult::NoSuchInstance,
                },

                Value::Boolean(s) => match name {
                    "i" => GetInstanceResult::Ok(Value::Int(*s as i64)),
                    _ => GetInstanceResult::NoSuchInstance,
                },

                Value::Int(i) => match name {
                    "zero?" => GetInstanceResult::Ok(Value::Boolean(i.is_zero())),
                    "one?" => GetInstanceResult::Ok(Value::Boolean(i.is_one())),
                    "pos?" => GetInstanceResult::Ok(Value::Boolean(i.is_positive())),
                    "neg?" => GetInstanceResult::Ok(Value::Boolean(i.is_negative())),
                    "even?" => GetInstanceResult::Ok(Value::Boolean(i.is_even())),
                    "odd?" => GetInstanceResult::Ok(Value::Boolean(i.is_odd())),
                    "abs" => GetInstanceResult::Ok(Value::Int(i.abs())),
                    _ => GetInstanceResult::NoSuchInstance,
                }

                _ => GetInstanceResult::NoSuchInstance,
            },
        }
    }

    pub fn call(&self, arguments: Vec<Value>) -> CallResult {
        match self {
            Value::NativeFunction(x) => x.0(self, arguments),
            _ => CallResult::NotCallable,
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
        assert_eq!(
            a.apply_operator("+", &b),
            ApplyOperatorResult::Ok(Value::Int(14))
        );
        assert_eq!(
            a.apply_operator("???", &b),
            ApplyOperatorResult::NoSuchOperator
        );
    }
}
