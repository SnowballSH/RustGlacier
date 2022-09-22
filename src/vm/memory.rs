use std::alloc::{alloc, Layout};

use lazy_static::lazy_static;

use crate::vm::value::Value;

lazy_static! {
    static ref LAYOUT: Layout = Layout::new::<Value>();
}

pub fn alloc_value_ptr() -> *mut Value {
    unsafe { alloc(*LAYOUT) as *mut Value }
}

pub fn alloc_new_value(val: Value) -> *mut Value {
    let ptr = alloc_value_ptr();
    unsafe {
        *ptr = val;
    }
    ptr
}
