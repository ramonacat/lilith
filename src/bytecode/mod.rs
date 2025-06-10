#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO implement debug manually so that the first 256 values are treated as TypeTag, also ensure
// this can only be constructed this way
pub struct TypeId(u32);

// TODO is this used anywhere?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackVariable(u64);

#[derive(Clone, Copy)]
pub enum ConstValue {
    U64(u64),
}

pub enum Value {
    Literal(ConstValue),
    Local(Identifier),
    Computed(Box<Expression>),
}

pub enum Expression {
    Assignment(Identifier, Value),
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
                Expression::Assignment(Identifier(1), Value::Literal(ConstValue::U64(100))),
                Expression::Assignment(Identifier(2), Value::Literal(ConstValue::U64(10))),
                Expression::Add(
                    Value::Literal(ConstValue::U64(1)),
                    Value::Computed(Box::new(Expression::Add(
                        Value::Local(Identifier(1)),
                        Value::Local(Identifier(2)),
                    ))),
                ),
            ],
        }
    }
}
