use crate::glacier_vm::value::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Instruction<'a> {
    Push(Value),
    Pop,
    Move((usize, usize)),
    MovePush(usize),
    MoveLast,
    MoveVar(&'a str),
    Var(&'a str),
}
