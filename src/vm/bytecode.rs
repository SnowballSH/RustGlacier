pub type Byte = u16;
// Size doesn't really matter
pub type ByteCodes = Vec<Byte>;

macro_rules! bytecodes_internal {
    () => {};
    ($x:ident, $i:expr) => {
        pub const $x: Byte = $i;
    };
    ($x:ident; $($y:ident);+, $i:expr) => {
        pub const $x: Byte = $i;
        bytecodes_internal!($($y);+, $i+1);
    };
}

macro_rules! bytecodes {
    () => {};

    ($x:ident;) => {
        pub const $x: Byte = 0;
    };
    ($x:ident; $($y:ident);+;) => {
        pub const $x: Byte = 0;
        bytecodes_internal!($($y);+, 1);
    }
}

bytecodes! {
    // POP_LAST
    // Stack: [a] -> []
    // Pops a
    POP_LAST;

    // REPLACE address
    // Stack: [addr, ..., a] -> [a, ...]
    // Pops a and puts it at address
    REPLACE;

    // LOAD_CONST address
    // Stack: [] -> [value]
    // Loads const[address] onto stack
    LOAD_CONST;

    // LOAD_LOCAL address
    // Stack: stack[address]=value -> [value]
    // Loads stack[address] onto stack
    LOAD_LOCAL;

    // DEBUG_PRINT
    // Stack: [a] -> []
    // Debug prints a
    DEBUG_PRINT;

    // JUMP_IF_FALSE address
    // Stack: [a] -> []
    // Jumps to address if stack if falsy
    JUMP_IF_FALSE;

    // JUMP_IF_FALSE_NO_POP address
    // Stack: [a] -> [a]
    // Jumps to address if stack if falsy, but don't pop
    JUMP_IF_FALSE_NO_POP;

    // JUMP address
    // Stack: [] -> []
    // Jumps to address
    JUMP;

    // UNARY_NEG
    // Stack: [a] -> [-a]
    // Negates a
    UNARY_NEG;
    // UNARY_NOT
    // Stack: [a] -> [!a]
    // Not a
    UNARY_NOT;

    // BINARY_ADD
    // Stack: [a, b] -> [a + b]
    // Adds b to a
    BINARY_ADD;
    // BINARY_SUB
    // Stack: [a, b] -> [a - b]
    // Subtracts b from a
    BINARY_SUB;
    // BINARY_MUL
    // Stack: [a, b] -> [a * b]
    // Multiplies a by b
    BINARY_MUL;
    // BINARY_DIV
    // Stack: [a, b] -> [a / b]
    // Divides a by b
    BINARY_DIV;
    // BINARY_MOD
    // Stack: [a, b] -> [a % b]
    // a MOD b
    BINARY_MOD;

    // BINARY_EQ
    // Stack: [a, b] -> [a == b]
    // a == b
    BINARY_EQ;
    // BINARY_NE
    // Stack: [a, b] -> [a != b]
    // a != b
    BINARY_NE;

    // BINARY_LT
    // Stack: [a, b] -> [a < b]
    // a < b
    BINARY_LT;
    // BINARY_LE
    // Stack: [a, b] -> [a <= b]
    // a <= b
    BINARY_LE;
    // BINARY_GT
    // Stack: [a, b] -> [a > b]
    // a > b
    BINARY_GT;
    // BINARY_GE
    // Stack: [a, b] -> [a >= b]
    // a >= b
    BINARY_GE;
}

pub fn bytecode_name(bytecode: Byte) -> &'static str {
    match bytecode {
        POP_LAST => "POP_LAST",
        REPLACE => "REPLACE",
        LOAD_CONST => "LOAD_CONST",
        LOAD_LOCAL => "LOAD_LOCAL",
        JUMP_IF_FALSE => "JUMP_IF_FALSE",
        JUMP_IF_FALSE_NO_POP => "JUMP_IF_FALSE_NO_POP",
        JUMP => "JUMP",
        DEBUG_PRINT => "DEBUG_PRINT",
        UNARY_NEG => "UNARY_NEG",
        UNARY_NOT => "UNARY_NOT",
        BINARY_ADD => "BINARY_ADD",
        BINARY_SUB => "BINARY_SUB",
        BINARY_MUL => "BINARY_MUL",
        BINARY_DIV => "BINARY_DIV",
        BINARY_MOD => "BINARY_MOD",
        BINARY_EQ => "BINARY_EQ",
        BINARY_NE => "BINARY_NE",
        BINARY_LT => "BINARY_LT",
        BINARY_LE => "BINARY_LE",
        BINARY_GT => "BINARY_GT",
        BINARY_GE => "BINARY_GE",
        _ => "UNKNOWN",
    }
}
