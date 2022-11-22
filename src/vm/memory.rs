use std::alloc::{alloc, dealloc, Layout};
use std::collections::hash_map::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::vm::value::Value;

static LAYOUT: Layout = Layout::new::<Value>();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GCItemState {
    White,
    Grey,
    Black,
    Persistent,
}

lazy_static! {
    pub static ref ALL_ALLOCATIONS: Mutex<HashMap<usize, GCItemState>> = Mutex::new(HashMap::new());
}

pub static mut LAST_ALLOCATED: usize = 0;

pub const GC_FORCE_COLLECT: usize = 1 << 19;

pub fn alloc_value_ptr() -> *mut Value {
    let ptr = unsafe { alloc(LAYOUT) as *mut Value };
    ALL_ALLOCATIONS
        .lock()
        .unwrap()
        .insert(ptr as usize, GCItemState::White);
    ptr
}

pub fn alloc_new_value(val: Value) -> *mut Value {
    let ptr = alloc_value_ptr();
    unsafe {
        *ptr = val;
    }
    ptr
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn mark(node: *mut Value) {
    let mut all_allocations = ALL_ALLOCATIONS.lock().unwrap();

    let mut grey_objects = Vec::new();

    if let Some(item) = all_allocations.get_mut(&(node as usize)) {
        if *item == GCItemState::White {
            *item = GCItemState::Grey;
            grey_objects.push(node);
        }
    }

    while !grey_objects.is_empty() {
        let g = grey_objects.pop().unwrap();
        if let Some(item) = all_allocations.get_mut(&(g as usize)) {
            if *item == GCItemState::Grey {
                *item = GCItemState::Black;

                if let Some(children) = unsafe { (*g).referenced_children() } {
                    for child in children {
                        if let Some(item) = all_allocations.get_mut(&(child as usize)) {
                            if *item == GCItemState::White {
                                *item = GCItemState::Grey;
                                grey_objects.push(child);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn sweep() {
    let mut all_allocations = ALL_ALLOCATIONS.lock().unwrap();
    let mut to_remove = Vec::new();
    for (ptr, state) in all_allocations.iter() {
        if *state == GCItemState::White {
            unsafe {
                dealloc(*ptr as *mut u8, LAYOUT);
            }
            to_remove.push(*ptr);
        }
    }

    // println!("Swept {} items", to_remove.len());

    for ptr in to_remove {
        all_allocations.remove(&ptr);
    }

    for (_, state) in all_allocations.iter_mut() {
        *state = GCItemState::White;
    }

    unsafe {
        LAST_ALLOCATED = all_allocations.len();
    };
}
