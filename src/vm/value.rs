#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::memory::alloc_new_value;

pub enum BinOpResult {
    Ok(*mut Value),
    Error(String),
    NoMatch,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f64),
    Int(i64),
    String(String),
    Bool(bool),
    Null,

    Array(Vec<*mut Value>),
}

impl Value {
    pub fn debug_format(&self) -> String {
        match self {
            Value::Float(f) => format!("{f:?}"),
            Value::Int(i) => format!("{i:?}"),
            Value::String(s) => format!("{s:?}"),
            Value::Bool(b) => format!("{b:?}"),
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

    pub fn print_format(&self) -> String {
        match self {
            Value::Float(f) => format!("{f}"),
            Value::Int(i) => format!("{i}"),
            Value::String(s) => s.to_string(),
            Value::Bool(b) => format!("{b}"),
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

    pub fn regular_copy_to(&mut self, dest: *mut Value) -> *mut Value {
        unsafe {
            *dest = self.clone();
        }

        dest
    }

    pub fn shallow_copy(&mut self) -> *mut Value {
        match self {
            Value::Array(_) => self as *mut Value,
            _ => alloc_new_value(self.clone()),
        }
    }

    pub fn deep_copy(&mut self) -> *mut Value {
        match self {
            Value::Array(a) => alloc_new_value(Value::Array(
                a.iter()
                    .map(|v| unsafe { &mut **v }.deep_copy())
                    .collect(),
            )),
            _ => alloc_new_value(self.clone()),
        }
    }

    pub fn referenced_children(&self) -> Option<Vec<*mut Value>> {
        match self {
            Value::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Float(f) => *f != 0.0,
            Value::Int(i) => *i != 0,
            Value::String(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            Value::Null => false,

            Value::Array(a) => !a.is_empty(),
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            (Value::Int(i1), Value::Int(i2)) => i1 == i2,
            (Value::String(s1), Value::String(s2)) => *s1 == *s2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Null, Value::Null) => true,

            (Value::Array(a1), Value::Array(a2)) => {
                if a1.len() != a2.len() {
                    return false;
                }

                for (v1, v2) in a1.iter().zip(a2.iter()) {
                    if !unsafe { &**v1 }.is_equal(unsafe { &**v2 }) {
                        return false;
                    }
                }

                true
            }

            _ => false,
        }
    }

    pub fn binary_add(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1 + f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Int(i1 + i2)))
            }
            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(*i1 as f64 + f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1 + *i1 as f64)))
            }

            (Value::String(s1), Value::String(s2)) => {
                BinOpResult::Ok(alloc_new_value(Value::String(s1.clone() + s2)))
            }

            (Value::Array(a1), Value::Array(a2)) => {
                let mut new_array = a1.clone();
                new_array.extend(a2.clone());

                BinOpResult::Ok(alloc_new_value(Value::Array(new_array)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_sub(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1 - f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Int(i1 - i2)))
            }

            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(*i1 as f64 - f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1 - *i1 as f64)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_mul(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1 * f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Int(i1 * i2)))
            }

            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(*i1 as f64 * f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1 * *i1 as f64)))
            }

            // Shallow repetition
            (Value::Array(a), Value::Int(i)) => {
                if *i < 0 {
                    return BinOpResult::Error("Array shallow repetition multiplier must be nonnegative".to_string());
                }

                let mut arr = Vec::new();
                for _ in 0..*i {
                    for v in a.iter() {
                        unsafe {
                            arr.push((**v).shallow_copy());
                        }
                    }
                }

                BinOpResult::Ok(alloc_new_value(Value::Array(arr)))
            }

            (Value::Array(a), Value::String(s)) => {
                let mut ss = Vec::with_capacity(a.len());
                unsafe {
                    for x in a {
                        ss.push((**x).print_format());
                    }
                }
                BinOpResult::Ok(alloc_new_value(Value::String(ss.join(s))))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_div(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                if *f2 == 0.0 {
                    BinOpResult::Error(format!("Division By Zero: {} / 0.0", *f1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Float(f1 / *f2)))
                }
            }
            (Value::Int(i1), Value::Int(i2)) => {
                if *i2 == 0 {
                    BinOpResult::Error(format!("Division By Zero: {} / 0", *i1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Int(i1 / *i2)))
                }
            }

            (Value::Int(i1), Value::Float(f1)) => {
                if *f1 == 0.0 {
                    BinOpResult::Error(format!("Division By Zero: {} / 0.0", *i1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Float(*i1 as f64 / f1)))
                }
            }
            (Value::Float(f1), Value::Int(i1)) => {
                if *i1 == 0 {
                    BinOpResult::Error(format!("Division By Zero: {} / 0", *f1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Float(f1 / *i1 as f64)))
                }
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_mod(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                if *f2 == 0.0 {
                    BinOpResult::Error(format!("Modulo By Zero: {} % 0.0", *f1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Float((f1 % *f2 + *f2) % *f2)))
                }
            }
            (Value::Int(i1), Value::Int(i2)) => {
                if *i2 == 0 {
                    BinOpResult::Error(format!("Modulo By Zero: {} % 0", *i1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Int((i1 % *i2 + *i2) % *i2)))
                }
            }
            (Value::Int(i1), Value::Float(f1)) => {
                if *f1 == 0.0 {
                    BinOpResult::Error(format!("Modulo By Zero: {} % 0.0", *i1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Float(((*i1 as f64) % f1 + f1) % f1)))
                }
            }
            (Value::Float(f1), Value::Int(i1)) => {
                if *i1 == 0 {
                    BinOpResult::Error(format!("Modulo By Zero: {} % 0", *f1))
                } else {
                    BinOpResult::Ok(alloc_new_value(Value::Float((f1 % (*i1 as f64) + *i1 as f64) % *i1 as f64)))
                }
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_exp(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1.powf(*f2))))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float((*i1 as f64).powf(*i2 as f64))))
            }
            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float((*i1 as f64).powf(*f1))))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Float(f1.powf(*i1 as f64))))
            }

            // Deep repetition
            (Value::Array(a), Value::Int(i)) => {
                if *i < 0 {
                    return BinOpResult::Error("Array deep repetition multiplier must be nonnegative".to_string());
                }

                let mut arr = Vec::new();
                for _ in 0..*i {
                    for v in a.iter() {
                        unsafe {
                            arr.push((**v).deep_copy());
                        }
                    }
                }

                BinOpResult::Ok(alloc_new_value(Value::Array(arr)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_lt(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(f1 < f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(i1 < i2)))
            }
            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool((*i1 as f64) < *f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(*f1 < (*i1 as f64))))
            }
            (Value::String(s1), Value::String(s2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(s1 < s2)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_gt(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(f1 > f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(i1 > i2)))
            }
            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool((*i1 as f64) > *f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(*f1 > (*i1 as f64))))
            }
            (Value::String(s1), Value::String(s2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(s1 > s2)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_le(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(f1 <= f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(i1 <= i2)))
            }
            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool((*i1 as f64) <= *f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(*f1 <= (*i1 as f64))))
            }
            (Value::String(s1), Value::String(s2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(s1 <= s2)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn binary_ge(&self, other: &Value) -> BinOpResult {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(f1 >= f2)))
            }
            (Value::Int(i1), Value::Int(i2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(i1 >= i2)))
            }
            (Value::Int(i1), Value::Float(f1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool((*i1 as f64) >= *f1)))
            }
            (Value::Float(f1), Value::Int(i1)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(*f1 >= (*i1 as f64))))
            }
            (Value::String(s1), Value::String(s2)) => {
                BinOpResult::Ok(alloc_new_value(Value::Bool(s1 >= s2)))
            }

            _ => BinOpResult::NoMatch,
        }
    }

    pub fn get_element(&self, index: *mut Value) -> Result<*mut Value, String> {
        unsafe {
            match self {
                Value::Array(a) => {
                    if let Value::Int(i) = *index {
                        if i < 0 {
                            Err(format!("Negative index not supported: {i}"))
                        } else if let Some(v) = a.get(i as usize) {
                            Ok(*v)
                        } else {
                            Err(format!("Index out of range: {i}"))
                        }
                    } else {
                        Err(format!(
                            "Array index must be an integer, not {}",
                            (*index).type_name()
                        ))
                    }
                }
                Value::String(s) => {
                    if let Value::Int(i) = *index {
                        if i < 0 {
                            Err(format!("Negative index not supported: {i}"))
                        } else if let Some(c) = s.chars().nth(i as usize) {
                            Ok(alloc_new_value(Value::String(c.to_string())))
                        } else {
                            Err(format!("Index out of range: {i}"))
                        }
                    } else {
                        Err(format!(
                            "String index must be an integer, not {}",
                            (*index).type_name()
                        ))
                    }
                }
                _ => Err(format!("Cannot get element from type {}", self.type_name())),
            }
        }
    }
}
