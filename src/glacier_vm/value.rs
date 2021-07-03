use num::BigInt;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Value {
    BigInt(BigInt),
    Int(i64),
}
