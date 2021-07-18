use crate::glacier_vm::value::ValueType;

#[derive(Debug, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum ErrorType {
    UndefinedVariable(String),
    InvalidBinaryOperation(ValueType, String, ValueType),
    ZeroDivisionOrModulo,
}

impl ErrorType {
    pub fn to_string(&self) -> String {
        match self {
            ErrorType::UndefinedVariable(name) => {
                format!("Undefined Variable: {}", name)
            }
            ErrorType::InvalidBinaryOperation(a, o, b) => {
                format!("Invalid Binary Operation: {} {} {}", a.to_string(), o, b.to_string())
            }
            ErrorType::ZeroDivisionOrModulo => {
                format!("Division or Modulo by Zero")
            }
        }
    }
}

pub type GlacierError = ErrorType;
