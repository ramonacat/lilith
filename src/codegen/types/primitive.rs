use inkwell::{builder::Builder, context::Context, values::PointerValue};

use super::{ClassId, ValueTypes, value::TypeTag};

pub(in crate::codegen) struct PrimitiveTypes<'ctx> {
    value_types: ValueTypes<'ctx>,
    context: &'ctx Context,
}

impl<'ctx> PrimitiveTypes<'ctx> {
    pub(in crate::codegen) const fn new(
        value_types: ValueTypes<'ctx>,
        context: &'ctx Context,
    ) -> Self {
        Self {
            value_types,
            context,
        }
    }

    pub(in crate::codegen) fn make_const_u64(
        &self,
        value: u64,
        builder: &Builder<'ctx>,
        target: PointerValue<'ctx>,
    ) {
        self.value_types.make_value(
            self.value_types.make_tag(TypeTag::U64),
            self.value_types.make_class_id(ClassId::none()),
            self.context.i64_type().const_int(value, false),
            builder,
            target,
        );
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
