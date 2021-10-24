use crate::glacier_vm::value::Value;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Instruction {
    Push(Value),
    Pop,
    Move((usize, usize)),
    MovePush(usize),
    MoveLastFromHeapToStack,

    MoveVar(String),

    MoveFree(usize),

    MoveLocal(usize),
    PushLocal(Value, usize),
    DefineLocal(usize),

    Link(usize, usize, usize),

    BinaryOperator(String),
    UnaryOperator(String),

    Call(usize),
    GetInstance(String),
    MoveLastToHeap,

    Jump(usize),
    JumpIfFalse(usize),

    // code location, name, params, index location
    MakeCode(usize, String, Vec<usize>, usize),
    Ret,

    Noop,

    SetLine(usize),
}

impl Instruction {
    pub fn to_string(&self) -> String {
        match self {
            Instruction::Push(x) => {
                format!("PUSH {}", x.to_string_u())
            }
            Instruction::Pop => {
                format!("POP")
            }
            Instruction::Move(x) => {
                format!("MOVE {} TO {}", x.0, x.1)
            }
            Instruction::MovePush(x) => {
                format!("MOVEPUSH {}", x)
            }
            Instruction::MoveLastFromHeapToStack => {
                format!("MOVE LAST TO STACK")
            }
            Instruction::MoveVar(x) => {
                format!("MOVENAME {}", x)
            }
            Instruction::MoveFree(x) => {
                format!("MOVEFREE {}", x)
            }
            Instruction::MoveLocal(x) => {
                format!("MOVELOCAL {}", x)
            }
            Instruction::PushLocal(a, b) => {
                format!("PUSH {} TO LOCAL {}", a.to_string_u(), b)
            }
            Instruction::DefineLocal(x) => {
                format!("SETLOCAL {}", x)
            }
            Instruction::Link(a, b, c) => {
                format!("LINKFREE {}, {} TO {}", a, b, c)
            }
            Instruction::BinaryOperator(x) => {
                format!("BINARYOP {}", x)
            }
            Instruction::UnaryOperator(x) => {
                format!("UNARYOP {}", x)
            }
            Instruction::Call(x) => {
                format!("CALL {}", x)
            }
            Instruction::GetInstance(x) => {
                format!("GETINSTANCE {}", x)
            }
            Instruction::MoveLastToHeap => {
                format!("MOVE LAST TO HEAP")
            }
            Instruction::Jump(x) => {
                format!("JMP TO {}", x)
            }
            Instruction::JumpIfFalse(x) => {
                format!("JFALSE TO {}", x)
            }
            Instruction::MakeCode(a, b, c, d) => {
                format!("MAKECODE LOC {} NAME {} ARGS {:?} TO {}", a, b, c, d)
            }
            Instruction::Ret => {
                format!("RETURN")
            }
            Instruction::Noop => {
                format!("NOOP")
            }
            Instruction::SetLine(x) => {
                format!("SETLINE {}", x)
            }
        }
    }
}

pub fn format_instructions(ins: Vec<Instruction>) -> String {
    ins.iter()
        .enumerate()
        .map(|(i, x)| format!("{:04}  {}", i, x.to_string()))
        .collect::<Vec<String>>()
        .join("\n")
}
