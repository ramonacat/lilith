use inkwell::{builder::Builder, values::PointerValue};

use super::{ClassId, ValueTypes, value::TypeTag};

pub(in crate::codegen) struct PrimitiveTypes<'ctx> {
    value_types: ValueTypes<'ctx>,
}

impl<'ctx> PrimitiveTypes<'ctx> {
    pub(in crate::codegen) const fn new(value_types: ValueTypes<'ctx>) -> Self {
        Self { value_types }
    }

    pub(crate) fn make_u64(
        &self,
        value: inkwell::values::IntValue<'ctx>,
        builder: &Builder<'ctx>,
        target: PointerValue<'ctx>,
    ) {
        self.value_types.make_value(
            self.value_types.make_tag(TypeTag::U64),
            self.value_types.make_class_id(ClassId::none()),
            value,
            builder,
            target,
        );
    }
}
