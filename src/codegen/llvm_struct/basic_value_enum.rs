use inkwell::values::{BasicValue, BasicValueEnum, IntValue, PointerValue, StructValue};

pub(in crate::codegen) trait IntoValue<'ctx, T: BasicValue<'ctx>> {
    fn into_value(self) -> T;
}

impl<'ctx> IntoValue<'ctx, IntValue<'ctx>> for BasicValueEnum<'ctx> {
    fn into_value(self) -> IntValue<'ctx> {
        self.into_int_value()
    }
}

impl<'ctx> IntoValue<'ctx, PointerValue<'ctx>> for BasicValueEnum<'ctx> {
    fn into_value(self) -> PointerValue<'ctx> {
        self.into_pointer_value()
    }
}

impl<'ctx> IntoValue<'ctx, StructValue<'ctx>> for BasicValueEnum<'ctx> {
    fn into_value(self) -> StructValue<'ctx> {
        self.into_struct_value()
    }
}
