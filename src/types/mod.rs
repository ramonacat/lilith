use crate::bytecode::ResultId;

pub enum ConstValue {
    U64(u64),
}

pub enum Value {
    Const(ConstValue),
    Opaque(ResultId),
}
