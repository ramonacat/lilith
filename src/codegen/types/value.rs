use inkwell::{
    builder::Builder,
    values::{IntValue, PointerValue},
};

use super::ClassId;
use crate::{
    codegen::{
        context_ergonomics::ContextErgonomics,
        llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
        types::ValueTypes,
    },
    llvm_struct,
};

llvm_struct! {
    struct Value {
        tag: TypeTag,
        unused_0: u8,
        class_id: ClassId,
        unused_1: u32,
        raw: u64
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(in crate::codegen) enum TypeTag {
    Primitive = 0,

    U64 = 16,

    FunctionSignature = 128,
}

impl TypeTag {
    pub(in crate::codegen) const fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Primitive),
            16 => Some(Self::U64),
            128 => Some(Self::FunctionSignature),
            _ => None,
        }
    }
}

impl<'ctx> ValueTypes<'ctx> {
    pub(in crate::codegen) fn make_tag(&self, tag: TypeTag) -> IntValue<'ctx> {
        self.context.i8_type().const_int(tag as u64, false)
    }

    pub(in crate::codegen) fn make_class_id(&self, id: ClassId) -> IntValue<'ctx> {
        self.context.i16_type().const_int(u64::from(id.0), false)
    }

    pub(in crate::codegen) fn make_value(
        &self,
        type_tag: IntValue<'ctx>,
        class_id: IntValue<'ctx>,
        value: IntValue<'ctx>,
        builder: &Builder<'ctx>,
        target: PointerValue<'ctx>,
    ) {
        self.value_type.fill_in(
            target,
            builder,
            type_tag,
            self.context.i8_type().const_zero(),
            class_id,
            self.context.i32_type().const_zero(),
            value,
        );
    }
}
