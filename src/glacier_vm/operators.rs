use inner::inner;

use crate::glacier_vm::error::GlacierError;
use crate::glacier_vm::value::{ApplyOperatorResult, ConvertResult, Value, ValueType};
use crate::glacier_vm::vm::Heap;

/// try to apply an infix operator
pub fn apply_operator(
    self_: &Value,
    name: &str,
    other: &Value,
    heap: &Heap,
) -> ApplyOperatorResult {
    if let Value::Reference(addr) = other {
        return self_.apply_operator(name, heap.value.get(*addr).unwrap(), heap);
    }
    match name {
        "==" => ApplyOperatorResult::Ok(Value::Boolean(self_ == other)),
        "!=" => ApplyOperatorResult::Ok(Value::Boolean(self_ != other)),
        "||" => ApplyOperatorResult::Ok(Value::Boolean(
            self_.is_truthy(heap) || other.is_truthy(heap),
        )),
        "&&" => ApplyOperatorResult::Ok(Value::Boolean(
            self_.is_truthy(heap) && other.is_truthy(heap),
        )),
        _ => match self_ {
            Value::Reference(addr) => heap
                .value
                .get(*addr)
                .unwrap()
                .apply_operator(name, other, heap),
            Value::Int(i) => match name {
                "+" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Int(i + inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => {
                            ApplyOperatorResult::Ok(Value::Int(i + inner!(other, if Value::Int)))
                        }
                    }
                }
                "-" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Int(i - inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => {
                            ApplyOperatorResult::Ok(Value::Int(i - inner!(other, if Value::Int)))
                        }
                    }
                }
                "*" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Int(i * inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => {
                            ApplyOperatorResult::Ok(Value::Int(i * inner!(other, if Value::Int)))
                        }
                    }
                }
                "/" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            let o = inner!(x, if Value::Int);
                            if o == 0 {
                                return ApplyOperatorResult::Error(
                                    GlacierError::ZeroDivisionOrModulo,
                                );
                            }
                            ApplyOperatorResult::Ok(Value::Int(i / o))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => {
                            let o = *inner!(other, if Value::Int);
                            if o == 0 {
                                return ApplyOperatorResult::Error(
                                    GlacierError::ZeroDivisionOrModulo,
                                );
                            }
                            ApplyOperatorResult::Ok(Value::Int(i / o))
                        }
                    }
                }
                "%" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            let o = inner!(x, if Value::Int);
                            if o == 0 {
                                return ApplyOperatorResult::Error(
                                    GlacierError::ZeroDivisionOrModulo,
                                );
                            }
                            ApplyOperatorResult::Ok(Value::Int(i % o))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => {
                            let o = *inner!(other, if Value::Int);
                            if o == 0 {
                                return ApplyOperatorResult::Error(
                                    GlacierError::ZeroDivisionOrModulo,
                                );
                            }
                            ApplyOperatorResult::Ok(Value::Int(i % o))
                        }
                    }
                }

                ">" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Boolean(i > &inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                            i > inner!(other, if Value::Int),
                        )),
                    }
                }
                "<" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Boolean(i <= &inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                            i <= inner!(other, if Value::Int),
                        )),
                    }
                }
                ">=" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Boolean(i >= &inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                            i >= inner!(other, if Value::Int),
                        )),
                    }
                }
                "<=" => {
                    let other_int_try = other.try_convert(ValueType::Int);

                    match other_int_try {
                        ConvertResult::Ok(x) => {
                            ApplyOperatorResult::Ok(Value::Boolean(i <= &inner!(x, if Value::Int)))
                        }
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::Boolean(
                            i <= inner!(other, if Value::Int),
                        )),
                    }
                }

                _ => ApplyOperatorResult::NoSuchOperator,
            },
            Value::String(s) => match name {
                "+" => {
                    let other_string_try = other.try_convert(ValueType::String);

                    match other_string_try {
                        ConvertResult::Ok(x) => ApplyOperatorResult::Ok(Value::String(
                            s.to_owned() + &*inner!(x, if Value::String),
                        )),
                        ConvertResult::NotOk => ApplyOperatorResult::NoSuchOperator,
                        ConvertResult::SameType => ApplyOperatorResult::Ok(Value::String(
                            s.to_owned() + &*inner!(other, if Value::String),
                        )),
                    }
                }
                _ => ApplyOperatorResult::NoSuchOperator,
            },
            _ => ApplyOperatorResult::NoSuchOperator,
        },
    }
}

pub fn apply_unary_operator(self_: &Value, name: &str, heap: &Heap) -> ApplyOperatorResult {
    match self_ {
        Value::Reference(addr) => heap
            .value
            .get(*addr)
            .unwrap()
            .apply_unary_operator(name, heap),
        Value::Int(x) => match name {
            "-" => ApplyOperatorResult::Ok(Value::Int(-*x)),
            "+" => ApplyOperatorResult::Ok(Value::Int(*x)),
            _ => ApplyOperatorResult::NoSuchOperator,
        },
        Value::Boolean(x) => match name {
            "!" => ApplyOperatorResult::Ok(Value::Boolean(!*x)),
            _ => ApplyOperatorResult::NoSuchOperator,
        },
        _ => ApplyOperatorResult::NoSuchOperator,
    }
}
