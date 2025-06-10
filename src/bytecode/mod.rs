// TODO where is this used? do we still need it?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopePath(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO should this be an Option<NonZeroU32>???
pub struct Identifier(u32);

impl Identifier {
    pub(crate) const fn is_none(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// TODO should this be an Option<NonZeroU32>???
pub struct TypeId(u32);
impl TypeId {
    pub(crate) const fn is_none(self) -> bool {
        self.0 == 0
    }
}

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
