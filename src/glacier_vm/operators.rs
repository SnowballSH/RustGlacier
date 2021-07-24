use inner::inner;

use crate::glacier_vm::error::GlacierError;
use crate::glacier_vm::value::{ApplyOperatorResult, ConvertResult, Value, ValueType};

/// try to apply an infix operator
pub fn apply_operator(self_: &Value, name: &str, other: &Value) -> ApplyOperatorResult {
    match name {
        "==" => ApplyOperatorResult::Ok(Value::Boolean(self_ == other)),
        "!=" => ApplyOperatorResult::Ok(Value::Boolean(self_ != other)),
        _ => {
            match self_ {
                Value::Int(i) => match name {
                    "+" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Int(
                                i + inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Int(
                                i + inner!(other, if Value::Int),
                            )),
                        }
                    }
                    "-" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Int(
                                i - inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Int(
                                i - inner!(other, if Value::Int),
                            )),
                        }
                    }
                    "*" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Int(
                                i * inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Int(
                                i * inner!(other, if Value::Int),
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
                                    i / o,
                                ))
                            }
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => {
                                let o = *inner!(other, if Value::Int);
                                if o == 0 {
                                    return ApplyOperatorResult::Error(GlacierError::ZeroDivisionOrModulo);
                                }
                                ApplyOperatorResult::Ok(Value::Int(
                                    i / o,
                                ))
                            }
                        }
                    }
                    "%" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => {
                                let o = inner!(x, if Value::Int);
                                if o == 0 {
                                    return ApplyOperatorResult::Error(GlacierError::ZeroDivisionOrModulo);
                                }
                                ApplyOperatorResult::Ok(Value::Int(
                                    i % o,
                                ))
                            }
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => {
                                let o = *inner!(other, if Value::Int);
                                if o == 0 {
                                    return ApplyOperatorResult::Error(GlacierError::ZeroDivisionOrModulo);
                                }
                                ApplyOperatorResult::Ok(Value::Int(
                                    i % o,
                                ))
                            }
                        }
                    }

                    ">" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Boolean(
                                i > &inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                                i > inner!(other, if Value::Int),
                            )),
                        }
                    }
                    "<" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Boolean(
                                i <= &inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                                i <= inner!(other, if Value::Int),
                            )),
                        }
                    }
                    ">=" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Boolean(
                                i >= &inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                                i >= inner!(other, if Value::Int),
                            )),
                        }
                    }
                    "<=" => {
                        let other_int_try = other.try_convert(ValueType::Int);

                        match other_int_try {
                            ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::Boolean(
                                i <= &inner!(x, if Value::Int),
                            )),
                            ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                            ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                                i <= inner!(other, if Value::Int),
                            )),
                        }
                    }

                    _ => ApplyOperatorResult::NoSuchOperator,
                },
                _ => ApplyOperatorResult::NoSuchOperator
            }
        }
    }
}

pub fn apply_unary_operator(self_: &Value, name: &str) -> ApplyOperatorResult {
    match self_ {
        Value::Int(x) => match name {
            "-" => {
                ApplyOperatorResult::Ok(Value::Int(-*x))
            }
            "+" => {
                ApplyOperatorResult::Ok(Value::Int(*x))
            }
            _ => ApplyOperatorResult::NoSuchOperator,
        },
        Value::Boolean(x) => match name {
            "!" => {
                ApplyOperatorResult::Ok(Value::Boolean(!*x))
            }
            _ => ApplyOperatorResult::NoSuchOperator,
        }
        _ => ApplyOperatorResult::NoSuchOperator
    }
}
