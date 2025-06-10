use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicType, IntType, PointerType},
    values::{BasicValue, IntValue, PointerValue},
};

use crate::{
    bytecode::{Identifier, TypeId, TypeTag},
    codegen::types::ClassId,
};

// TODO we should probably have a macro here, because most of the impls are really just int
// representations of varying lengths and there's a lot of repeated code for that reason

pub(in crate::codegen) trait LlvmRepresentation<'ctx> {
    type LlvmValue: BasicValue<'ctx>;
    type LlvmType: BasicType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType;
    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue);
}

impl<'ctx> LlvmRepresentation<'ctx> for u8 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i8_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u16 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i16_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u32 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i32_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for u64 {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i64_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
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

    fn assert_valid(_context: &'ctx Context, _value: Self::LlvmValue) {}
}

impl<'ctx, T> LlvmRepresentation<'ctx> for Option<*const T> {
    type LlvmValue = PointerValue<'ctx>;
    type LlvmType = PointerType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.ptr_type(AddressSpace::default())
    }

    fn assert_valid(_context: &'ctx Context, _value: Self::LlvmValue) {}
}

impl<'ctx> LlvmRepresentation<'ctx> for TypeTag {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i8_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for ClassId {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i16_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for Identifier {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i32_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}

impl<'ctx> LlvmRepresentation<'ctx> for TypeId {
    type LlvmValue = IntValue<'ctx>;
    type LlvmType = IntType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
        context.i32_type()
    }

    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
        assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
    }
}
