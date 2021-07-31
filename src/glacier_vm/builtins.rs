use lazy_static::lazy_static;

use crate::glacier_vm::error::ErrorType;
use crate::glacier_vm::value::{CallResult, Value, FT};
use crate::glacier_vm::vm::Heap;

fn print_fn_internal(_this: &Value, arguments: Vec<Value>, _heap: &Heap) -> CallResult {
    let mut strings = vec![];
    for a in arguments {
        strings.push(a.to_string());
    }
    let res = strings.join(" ");
    println!("{}", res);
    CallResult::Ok(Value::Null)
}

fn to_bool_fn_internal(_this: &Value, arguments: Vec<Value>, _heap: &Heap) -> CallResult {
    if arguments.len() != 1 {
        return CallResult::Error(ErrorType::ArgumentError(format!(
            "bool() expects 1 argument, got {}",
            arguments.len()
        )));
    }

    CallResult::Ok(Value::Boolean(arguments[0].is_truthy()))
}

lazy_static! {
    pub static ref PRINT_FN: Value = Value::NativeFunction(FT(print_fn_internal));
    pub static ref TO_BOOL_FN: Value = Value::NativeFunction(FT(to_bool_fn_internal));
}

pub fn get_builtin(name: String) -> Option<Value> {
    match name.as_str() {
        "print" => Some((*PRINT_FN).clone()),
        "bool" => Some((*TO_BOOL_FN).clone()),
        "true" => Some(Value::Boolean(true)),
        "false" => Some(Value::Boolean(false)),
        _ => None,
    }
}
