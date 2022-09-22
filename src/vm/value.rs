use gc::{Finalize, Gc, GcCell, GcCellRefMut, Trace};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(GcCell<String>),
    Bool(bool),
    Null,

    Array(Vec<*mut Value>),
}

impl Value {
    pub fn debug_format(&self) -> String {
        match self {
            Value::Float(f) => format!("{}", f),
            Value::Int(i) => format!("{}", i),
            Value::String(s) => format!("\"{}\"", s.borrow()),
            Value::Bool(b) => format!("{}", b),
            Value::Null => "null".to_string(),

            Value::Array(a) => format!(
                "[{}]",
                a.iter()
                    .map(|v| unsafe { &**v }.debug_format())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Float(_) => "float",
            Value::Int(_) => "int",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",

            Value::Array(_) => "array",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Float(f) => *f != 0.0,
            Value::Int(i) => *i != 0,
            Value::String(s) => !s.borrow().is_empty(),
            Value::Bool(b) => *b,
            Value::Null => false,

            Value::Array(a) => !a.is_empty(),
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn get_element(&self, index: *mut Value) -> Result<*mut Value, String> {
        unsafe {
            match self {
                Value::Array(a) => {
                    if let Value::Int(i) = *index {
                        if i < 0 {
                            Err(format!("Negative index not supported: {}", i))
                        } else if let Some(v) = a.get(i as usize) {
                            Ok(*v)
                        } else {
                            Err(format!("Index out of range: {}", i))
                        }
                    } else {
                        Err(format!(
                            "Array index must be an integer, not {}",
                            (*index).type_name()
                        ))
                    }
                }
                _ => Err(format!("Cannot get element from type {}", self.type_name())),
            }
        }
    }
}
