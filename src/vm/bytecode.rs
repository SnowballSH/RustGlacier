pub type Byte = u16;  // Size doesn't really matter
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
    // LOAD_CONST address
    // Stack: [] -> [value]
    // Loads address onto stack
    LOAD_CONST;
    // DEBUG_PRINT
    // Stack: [a] -> []
    // Debug prints a
    DEBUG_PRINT;
    // UNARY_NEG
    // Stack: [a] -> [-a]
    // Negates a
    UNARY_NEG;
}