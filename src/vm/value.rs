use std::alloc::{dealloc, Layout};
use std::slice::from_raw_parts;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Value {
    Float(f64),
    Int(i64),
    String((*mut u8, usize)),
    Bool(bool),
    Null,
}

impl Value {
    pub fn debug_format(&self) -> String {
        match self {
            Value::Float(f) => format!("{}", f),
            Value::Int(i) => format!("{}", i),
            Value::String(s) => format!("\"{}\"", unsafe {
                String::from_utf8_lossy(from_raw_parts(s.0, s.1))
            }),
            Value::Bool(b) => format!("{}", b),
            Value::Null => "null".to_string(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Float(_) => "float",
            Value::Int(_) => "int",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Float(f) => *f != 0.0,
            Value::Int(i) => *i != 0,
            Value::String(s) => s.1 != 0,
            Value::Bool(b) => *b,
            Value::Null => false,
        }
    }

    pub fn free(&self) {
        match self {
            Value::String(s) => unsafe {
                dealloc(s.0, Layout::for_value(from_raw_parts(s.0, s.1)))
            },
            _ => (),
        }
    }
}
