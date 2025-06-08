use inkwell::{context::Context, types::BasicTypeEnum};

use crate::codegen::context_ergonomics::ContextErgonomics as _;

pub(in crate::codegen) trait LlvmRepresentation<'ctx> {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx>;
}

impl<'ctx> LlvmRepresentation<'ctx> for u8 {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.i8()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u16 {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.i16()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u32 {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.i32()
    }
}
