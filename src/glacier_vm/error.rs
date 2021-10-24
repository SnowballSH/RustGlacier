use crate::glacier_vm::value::{Value, ValueType};

#[derive(Debug, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum ErrorType {
    UndefinedVariable(String),
    InvalidBinaryOperation(Value, String, Value),
    InvalidUnaryOperation(Value, String),
    ZeroDivisionOrModulo,
    NotCallable(ValueType),
    NoInstance(Value, String),
    ArgumentError(String),
    ConversionError(String),
    InFunction(String, Box<ErrorType>),
    Failure(String),
}

impl ErrorType {
    pub fn to_string(&self) -> String {
        match self {
            ErrorType::UndefinedVariable(name) => {
                format!("Undefined Variable: {}", name)
            }
            ErrorType::InvalidBinaryOperation(a, o, b) => {
                format!(
                    "Invalid Binary Operation: {} {} {}",
                    a.to_debug_string(),
                    o,
                    b.to_debug_string()
                )
            }
            ErrorType::InvalidUnaryOperation(a, o) => {
                format!("Invalid Unary Operation: {}{}", o, a.to_debug_string())
            }
            ErrorType::ZeroDivisionOrModulo => {
                format!("Division or Modulo by Zero")
            }
            ErrorType::NotCallable(t) => {
                format!("Type {:?} is not callable", t)
            }
            ErrorType::NoInstance(val, name) => {
                format!(
                    "Instance '{}' does not exist on {}",
                    name,
                    val.to_debug_string()
                )
            }
            ErrorType::ArgumentError(x) | ErrorType::ConversionError(x) | ErrorType::Failure(x) => {
                x.clone()
            }
            ErrorType::InFunction(x, y) => {
                format!("In Function {}:\n{}", x, y.to_string())
            }
        }
    }
}

pub type GlacierError = ErrorType;
