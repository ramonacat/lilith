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
        self.context.const_u8(tag as u8)
    }

    pub(in crate::codegen) fn make_class_id(&self, id: ClassId) -> IntValue<'ctx> {
        self.context.const_u16(id.0)
    }

    pub(in crate::codegen) fn make_value(
        &self,
        type_tag: IntValue<'ctx>,
        class_id: IntValue<'ctx>,
        value: IntValue<'ctx>,
        builder: &Builder<'ctx>,
    ) -> ValueOpaquePointer<'ctx> {
        let target = builder
            .build_malloc(self.value_type.llvm_type(), "target")
            .unwrap();

        self.value_type.fill_in(
            target,
            builder,
            type_tag,
            self.context.const_u8(0),
            class_id,
            self.context.const_u32(0),
            value,
        );

        self.value_type.opaque_pointer(target)
    }
}
