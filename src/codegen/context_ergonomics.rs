use inkwell::{
    types::{BasicTypeEnum, StructType},
    values::IntValue,
};

use super::context::AsLlvmContext;

pub(super) trait ContextErgonomics<'ctx> {
    fn const_u8(&self, value: u8) -> IntValue<'ctx>;
    fn const_u16(&self, value: u16) -> IntValue<'ctx>;
    fn const_u32(&self, value: u32) -> IntValue<'ctx>;
    fn const_u64(&self, value: u64) -> IntValue<'ctx>;

    fn named_struct(&self, name: &str, fields: &[BasicTypeEnum<'ctx>]) -> StructType<'ctx>;
}

impl<'ctx, T: AsLlvmContext<'ctx>> ContextErgonomics<'ctx> for T {
    fn const_u8(&self, value: u8) -> IntValue<'ctx> {
        self.llvm_context()
            .i8_type()
            .const_int(u64::from(value), false)
    }

    fn const_u16(&self, value: u16) -> IntValue<'ctx> {
        self.llvm_context()
            .i16_type()
            .const_int(u64::from(value), false)
    }

    fn const_u32(&self, value: u32) -> IntValue<'ctx> {
        self.llvm_context()
            .i32_type()
            .const_int(u64::from(value), false)
    }

    fn const_u64(&self, value: u64) -> IntValue<'ctx> {
        self.llvm_context().i64_type().const_int(value, false)
    }

    // TODO get rid of it and have something akin to make_function_type!?
    fn named_struct(&self, name: &str, fields: &[BasicTypeEnum<'ctx>]) -> StructType<'ctx> {
        let r#struct = self.llvm_context().opaque_struct_type(name);
        r#struct.set_body(fields, false);
        r#struct
    }
}
