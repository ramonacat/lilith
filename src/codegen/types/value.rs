use inkwell::{
    builder::Builder,
    values::{IntValue, PointerValue},
};

use super::ClassId;
use crate::{
    bytecode::TypeTag,
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
