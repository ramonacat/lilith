use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicType, IntType, PointerType},
    values::{BasicValue, IntValue, PointerValue},
};

use crate::codegen::{TypeTag, types::ClassId};

pub(in crate::codegen) trait LlvmRepresentation<'ctx> {
    type LlvmValue: BasicValue<'ctx>;
    type LlvmType: BasicType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType;
}

impl<'ctx> LlvmRepresentation<'ctx> for u8 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i8_type()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u16 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i16_type()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u32 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i32_type()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u64 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i64_type()
    }
}

impl<'ctx, T> LlvmRepresentation<'ctx> for *const T {
    type LlvmValue = PointerValue<'ctx>;
    type LlvmType = PointerType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        // TODO do we want separate address spaces for code & data for archs where it matters? any
        // other reasons to have more than default?
        context.ptr_type(AddressSpace::default())
    }
}

impl<'ctx, T> LlvmRepresentation<'ctx> for Option<*const T> {
    type LlvmValue = PointerValue<'ctx>;
    type LlvmType = PointerType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.ptr_type(AddressSpace::default())
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for TypeTag {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i8_type()
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for ClassId {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i16_type()
    }
}
