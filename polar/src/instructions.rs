#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Opcode {
    HLT = 0,
    IGL,
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        return match v {
            0 => Opcode::HLT,
            _ => Opcode::IGL
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction {
            opcode
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_new() {
        let instruction = Instruction::new(Opcode::HLT);
        assert_eq!(instruction.opcode, Opcode::HLT);
    }
}
