use inner::inner;
use num::BigInt;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    BigInt(BigInt),
    Int(i64),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    BigInt,
    Int,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConvertResult {
    Ok(Value),
    NotOk,
    SameType,
}

impl Value {
    pub fn apply_operator(&self, name: &str, other: &Value) -> Option<Self> {
        match self {
            Value::BigInt(_) => None,
            Value::Int(_) => match name {
                "+" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => Some(Value::Int(
                            inner!(self, if Value::Int) + inner!(x, if Value::Int),
                        )),
                        ConvertResult::NotOk => None,
                        ConvertResult::SameType => Some(Value::Int(
                            inner!(self, if Value::Int) + inner!(other, if Value::Int),
                        )),
                    }
                }
                _ => None,
            },
        }
    }

    #[inline]
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::BigInt(_) => ValueType::BigInt,
            Value::Int(_) => ValueType::Int,
        }
    }

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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::glacier_vm::value::{ConvertResult, Value, ValueType};
    use num::BigInt;

    #[test]
    fn test_operator() {
        let a = Value::Int(6);
        let b = Value::Int(8);
        assert_eq!(a.try_convert(ValueType::Int), ConvertResult::SameType);
        assert_eq!(
            b.try_convert(ValueType::BigInt),
            ConvertResult::Ok(Value::BigInt(BigInt::from(8)))
        );
        assert_eq!(a.apply_operator("+", &b), Some(Value::Int(14)));
        assert_eq!(a.apply_operator("???", &b), None);
    }
}
