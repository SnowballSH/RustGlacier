use crate::glacier_vm::value::ValueType;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum ErrorType<'a> {
    UndefinedVariable(&'a str),
    InvalidBinaryOperation(ValueType, &'a str, ValueType),
}

impl <'a> ErrorType<'a> {
    pub fn to_string(&self) -> String {
        match self {
            ErrorType::UndefinedVariable(name) => {
                format!("Undefined Variable: {}", name)
            }
            ErrorType::InvalidBinaryOperation(a, o, b) => {
                format!("Invalid Binary Operation: {} {} {}", a.to_string(), o, b.to_string())
            }
        }
    }
}

pub type GlacierError<'a> = ErrorType<'a>;
