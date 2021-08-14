use std::io;
use std::io::Write;

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

fn get_input_fn_internal(_this: &Value, arguments: Vec<Value>, _heap: &Heap) -> CallResult {
    let prompt = if arguments.len() == 0 {
        String::new()
    } else if arguments.len() == 1 {
        arguments[0].to_string()
    } else {
        return CallResult::Error(ErrorType::ArgumentError(format!(
            "get() requires 0 or 1 argument, got {}",
            arguments.len()
        )));
    };
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().expect("flush failed!");
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    CallResult::Ok(Value::String(input.trim_end().to_string()))
}

lazy_static! {
    pub static ref PRINT_FN: Value = Value::NativeFunction(FT(print_fn_internal));
    pub static ref GET_FN: Value = Value::NativeFunction(FT(get_input_fn_internal));
}

/// Gets a builtin from a string.
pub fn get_builtin(name: String) -> Option<Value> {
    match name.as_str() {
        "print" => Some((*PRINT_FN).clone()),
        "get" => Some((*GET_FN).clone()),
        "true" => Some(Value::Boolean(true)),
        "false" => Some(Value::Boolean(false)),
        _ => None,
    }
}
