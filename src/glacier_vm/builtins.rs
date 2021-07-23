use lazy_static::lazy_static;

use crate::glacier_vm::value::{CallResult, FT, Value};
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

lazy_static! {
    pub static ref PRINT_FN: Value = Value::NativeFunction(FT(print_fn_internal));
}

pub fn get_builtin(name: String) -> Option<Value> {
    match name.as_str() {
        "print" => Some((*PRINT_FN).clone()),
        _ => None
    }
}
