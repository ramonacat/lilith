use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicTypeEnum, StructType},
};

pub(super) trait ContextErgonomics<'ctx> {
    fn ptr(&'ctx self) -> BasicTypeEnum<'ctx>;

    fn named_struct(&'ctx self, name: &str, fields: &[BasicTypeEnum<'ctx>]) -> StructType<'ctx>;
}

impl<'ctx> ContextErgonomics<'ctx> for Context {
    // TODO do we want separate address spaces for functions and data (for archs where that
    // matters)
    fn ptr(&'ctx self) -> BasicTypeEnum<'ctx> {
        self.ptr_type(AddressSpace::default()).into()
    }

    fn named_struct(&'ctx self, name: &str, fields: &[BasicTypeEnum<'ctx>]) -> StructType<'ctx> {
        let r#struct = self.opaque_struct_type(name);
        r#struct.set_body(fields, false);
        r#struct
    }
}
