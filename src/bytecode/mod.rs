use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Identifier(u32);

impl Identifier {
    // TODO this function shouldn't be so public, but we first need to really have interning to be
    // able to avoid it
    pub(crate) const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TypeTag {
    Primitive = 0,

    U64 = 16,

    FunctionSignature = 128,
}

impl From<TypeTag> for TypeId {
    fn from(value: TypeTag) -> Self {
        Self(value as u32)
    }
}

impl TypeTag {
    pub(crate) const fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Primitive),
            16 => Some(Self::U64),
            128 => Some(Self::FunctionSignature),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// TODO ensure this can only be constructed so that first 256 values represent TypeTag
// TODO this might need to be converted into some more opaque concept, as some types have IDs that
// only get created at runtime (otoh we can just convert the value, so no biggie?)
pub struct TypeId(u32);

impl TypeId {
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl Debug for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(tag) = u8::try_from(self.0) {
            write!(f, "TypeId({:?})", TypeTag::from_value(tag).unwrap())
        } else {
            write!(f, "TypeTag({})", self.0)
        }
    }
}

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
