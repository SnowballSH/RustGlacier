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

    // code len, name, params
    MakeCode(Vec<Instruction>, String, Vec<String>),

    Noop,
    ToggleRef,

    SetLine(usize),
}
