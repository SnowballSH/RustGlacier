#[repr(C)]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(&'static str),
    Bool(bool),
    Null,
}

impl Value {
    pub fn debug_format(&self) -> String {
        match self {
            Value::Float(f) => format!("{}", f),
            Value::Int(i) => format!("{}", i),
            Value::String(s) => format!("\"{}\"", s),
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
            Value::String(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            Value::Null => false,
        }
    }
}
