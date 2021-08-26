use crate::glacier_vm::value::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Instruction {
    Push(Value),
    Pop,
    Move((usize, usize)),
    MovePush(usize),
    MoveLast,
    MoveVar(String),
    Var(String),

    BinaryOperator(String),
    UnaryOperator(String),

    Call(usize),
    GetInstance(String),
    MoveLastToStack,

    Jump(usize),
    JumpIfFalse(usize),

    // code location, name, params
    MakeCode(usize, String, Vec<String>),
    Ret,

    Noop,
    ToggleRef,

    SetLine(usize),
}
