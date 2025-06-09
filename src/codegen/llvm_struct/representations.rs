use inkwell::{context::Context, types::BasicTypeEnum};

use crate::codegen::{TypeTag, context_ergonomics::ContextErgonomics as _, types::ClassId};

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

impl<'ctx> LlvmRepresentation<'ctx> for u64 {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.i64()
    }
}

impl<'ctx, T> LlvmRepresentation<'ctx> for *const T {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.ptr()
    }
}

impl<'ctx, T> LlvmRepresentation<'ctx> for Option<*const T> {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.ptr()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for TypeTag {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.i8()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for ClassId {
    fn llvm_type(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        context.i16()
    }
}
