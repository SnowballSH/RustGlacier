use std::alloc::{alloc, Layout};
use std::collections::hash_set::HashSet;
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::vm::value::Value;

static LAYOUT: Layout = Layout::new::<Value>();

lazy_static! {
    pub static ref ALL_ALLOCATIONS: Mutex<HashSet<usize>> = Mutex::new(HashSet::new());
}

pub fn alloc_value_ptr() -> *mut Value {
    let ptr = unsafe { alloc(LAYOUT) as *mut Value };
    ALL_ALLOCATIONS.lock().unwrap().insert(ptr as usize);
    ptr
}

pub fn alloc_new_value(val: Value) -> *mut Value {
    let ptr = alloc_value_ptr();
    unsafe {
        *ptr = val;
    }
    ptr
}
