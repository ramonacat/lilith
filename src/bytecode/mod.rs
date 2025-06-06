// TODO for now this is just an ID, since we don't really have a concept of scope, but this'll have
// to differentiate between modules/classes/methods/closures/etc.
use crate::types::{ConstValue, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopePath(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO the whole scopepath/resultid thing should be handled internally here, and be opaque for
// codegen
pub struct ResultId(pub ScopePath);

pub enum Expression {
    Assignment(ScopePath, Box<Expression>),
    Literal(ConstValue),
    Add(Value, Value),
}

// TODO this probably be at like function granularity or something
pub struct ByteCode {
    // TODO this probably shouldn't be pub
    pub instructions: Vec<Expression>,
}

impl ByteCode {
    pub fn new() -> Self {
        Self {
            instructions: vec![
                Expression::Assignment(
                    ScopePath(1),
                    Box::new(Expression::Literal(ConstValue::U64(4))),
                ),
                Expression::Add(
                    Value::Const(ConstValue::U64(123)),
                    Value::Opaque(ResultId(ScopePath(1))),
                ),
            ],
        }
    }
}
