use crate::glacier_vm::value::ValueType;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum ErrorType<'a> {
    UndefinedVariable(&'a str),
    InvalidBinaryOperation(ValueType, &'a str, ValueType),
}

pub type GlacierError<'a> = ErrorType<'a>;
