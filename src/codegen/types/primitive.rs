use inkwell::builder::Builder;

use super::{ClassId, ValueTypes, value::ValueOpaquePointer};
use crate::codegen::{context::AsLlvmContext, types::TypeTag};

pub(in crate::codegen) struct PrimitiveTypes<'ctx, TContext: AsLlvmContext<'ctx>> {
    value_types: ValueTypes<'ctx, TContext>,
}

impl<'ctx, TContext: AsLlvmContext<'ctx>> PrimitiveTypes<'ctx, TContext> {
    pub(in crate::codegen) const fn new(value_types: ValueTypes<'ctx, TContext>) -> Self {
        Self { value_types }
    }

    pub(crate) fn make_u64(
        &self,
        value: inkwell::values::IntValue<'ctx>,
        builder: &Builder<'ctx>,
    ) -> ValueOpaquePointer<'ctx, TContext> {
        self.value_types.make_value(
            self.value_types.make_tag(TypeTag::U64),
            self.value_types.make_class_id(ClassId::none()),
            value,
            builder,
        )
    }
}
