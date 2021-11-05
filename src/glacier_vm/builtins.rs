use std::io;
use std::io::Write;

use lazy_static::lazy_static;

use crate::glacier_vm::error::{ErrorType, GlacierError};
use crate::glacier_vm::value::{CallResult, GetPropertyResult, NModule, Value, FT};
use crate::glacier_vm::vm::Heap;

fn print_fn_internal(_this: &Value, arguments: Vec<usize>, heap: &mut Heap) -> CallResult {
    let mut strings = vec![];
    for a in arguments {
        strings.push(heap.value[a].to_string(heap));
    }
    let res = strings.join(" ");
    println!("{}", res);
    CallResult::Ok(Value::Null)
}

fn get_input_fn_internal(_this: &Value, arguments: Vec<usize>, heap: &mut Heap) -> CallResult {
    let prompt = if arguments.len() == 0 {
        String::new()
    } else if arguments.len() == 1 {
        heap.value[arguments[0]].to_string(heap)
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

fn push_vector_fn_internal(_this: &Value, arguments: Vec<usize>, heap: &mut Heap) -> CallResult {
    if arguments.is_empty() {
        return CallResult::Error(GlacierError::ArgumentError(format!(
            "Expected 1st argument to be vector, got none"
        )));
    }
    let first = unsafe { heap.value.get_unchecked(*arguments.get_unchecked(0)) };
    if let Value::Reference(first_i) = first {
        let first_i = *first_i;
        let first = unsafe { heap.value.get_unchecked_mut(first_i) };
        if let Value::Vector(vector) = first {
            for a in arguments.into_iter().skip(1) {
                vector.push(a);
            }
            return CallResult::Ok(Value::Null);
        }
        return CallResult::Error(GlacierError::ArgumentError(format!(
            "Expected 1st argument to be reference to vector, got type {}",
            first.value_type().to_string()
        )));
    }
    CallResult::Error(GlacierError::ArgumentError(format!(
        "Expected 1st argument to be reference to vector, got type {}",
        first.value_type().to_string()
    )))
}

fn vector_properties(name: &str) -> GetPropertyResult {
    match name {
        "new" => GetPropertyResult::Ok(Value::Vector(Vec::new())),
        "push" => GetPropertyResult::Ok(Value::NativeFunction(FT(push_vector_fn_internal))),
        _ => GetPropertyResult::NoSuchProperty,
    }
}

lazy_static! {
    pub static ref PRINT_FN: Value = Value::NativeFunction(FT(print_fn_internal));
    pub static ref GET_FN: Value = Value::NativeFunction(FT(get_input_fn_internal));
    pub static ref VECTOR_MOD: Value = Value::NativeModule(NModule {
        name: "Vec".to_string(),
        get_property: vector_properties,
    });
}

/// Gets a builtin from a string.
pub fn get_builtin(name: String) -> Option<Value> {
    match name.as_str() {
        "print" => Some((*PRINT_FN).clone()),
        "get" => Some((*GET_FN).clone()),
        "true" => Some(Value::Boolean(true)),
        "false" => Some(Value::Boolean(false)),
        "null" => Some(Value::Null),

        "Vec" => Some((*VECTOR_MOD).clone()),

        _ => None,
    }
}
