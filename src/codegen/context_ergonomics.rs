use inkwell::{
    context::Context,
    types::{BasicTypeEnum, StructType},
    values::IntValue,
};

pub(super) trait ContextErgonomics<'ctx> {
    fn const_u32(&'ctx self, value: u32) -> IntValue<'ctx>;
    fn const_u64(&'ctx self, value: u64) -> IntValue<'ctx>;

    fn named_struct(&'ctx self, name: &str, fields: &[BasicTypeEnum<'ctx>]) -> StructType<'ctx>;
}

impl<'ctx> ContextErgonomics<'ctx> for Context {
    fn const_u32(&'ctx self, value: u32) -> IntValue<'ctx> {
        self.i32_type().const_int(u64::from(value), false)
    }

    fn const_u64(&'ctx self, value: u64) -> IntValue<'ctx> {
        self.i64_type().const_int(value, false)
    }

    // TODO get rid of it and have something akin to make_function_type!?
    fn named_struct(&'ctx self, name: &str, fields: &[BasicTypeEnum<'ctx>]) -> StructType<'ctx> {
        let r#struct = self.opaque_struct_type(name);
        r#struct.set_body(fields, false);
        r#struct
    }
}
