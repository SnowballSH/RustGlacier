use inner::inner;
use num::BigInt;
use crate::glacier_vm::error::GlacierError;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    BigInt(BigInt),
    Int(i64),
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
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueType {
    BigInt,
    Int,
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

impl Value {
    pub fn apply_operator(&self, name: &str, other: &Value) -> ApplyOperatorResult {
        match self {
            Value::BigInt(_) => ApplyOperatorResult::NoSuchOperator,
            Value::Int(_) => match name {
                "+" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Int(
                            inner!(self, if Value::Int) + inner!(x, if Value::Int),
                        )),
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Int(
                            inner!(self, if Value::Int) + inner!(other, if Value::Int),
                        )),
                    }
                }
                "-" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Int(
                            inner!(self, if Value::Int) - inner!(x, if Value::Int),
                        )),
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Int(
                            inner!(self, if Value::Int) - inner!(other, if Value::Int),
                        )),
                    }
                }
                "*" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Int(
                            inner!(self, if Value::Int) * inner!(x, if Value::Int),
                        )),
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Int(
                            inner!(self, if Value::Int) * inner!(other, if Value::Int),
                        )),
                    }
                }
                "/" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            let o = inner!(x, if Value::Int);
                            if o == 0 {
                                return ApplyOperatorResult::Error(GlacierError::ZeroDivisionOrModulo);
                            }
                            ApplyOperatorResult::Ok(Value::Int(
                                inner!(self, if Value::Int) / o,
                            ))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => {
                            let o = *inner!(other, if Value::Int);
                            if o == 0 {
                                return ApplyOperatorResult::Error(GlacierError::ZeroDivisionOrModulo);
                            }
                            ApplyOperatorResult::Ok(Value::Int(
                                inner!(self, if Value::Int) / o,
                            ))
                        },
                    }
                }
                _ => ApplyOperatorResult::NoSuchOperator,
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
