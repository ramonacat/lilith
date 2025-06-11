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

macro_rules! llvm_representation {
    (@int $type:ty, $width: literal) => {
        impl<'ctx> LlvmRepresentation<'ctx> for $type {
            type LlvmValue = IntValue<'ctx>;
            type LlvmType = IntType<'ctx>;

            fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
                paste::paste!(context.[<i $width _type>]())
            }

            fn assert_valid(context: &'ctx Context, value: Self::LlvmValue) {
                assert!(value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width());
            }
        }
    };
    (@ptr<$generic:ident> $type:ty) => {
        impl<'ctx, $generic> LlvmRepresentation<'ctx> for $type {
            type LlvmValue = PointerValue<'ctx>;
            type LlvmType = PointerType<'ctx>;

            fn llvm_type(context: &'ctx Context) -> Self::LlvmType {
                context.ptr_type(AddressSpace::default())
            }

            fn assert_valid(_context: &'ctx Context, _value: Self::LlvmValue) {
            }
        }
    };
}

pub(in crate::codegen) trait LlvmRepresentation<'ctx> {
    type LlvmValue: BasicValue<'ctx>;
    type LlvmType: BasicType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType;
    fn assert_valid(context: &'ctx Context, value: Self::LlvmValue);
}

llvm_representation!(@int u8, 8);
llvm_representation!(@int u16, 16);
llvm_representation!(@int u32, 32);
llvm_representation!(@int u64, 64);
// TODO We have to check that the structs actually have a repr(C)/repr(transparent) and enums have
// repr(u*), but AFAIK that's only possible with proc macros, so this is a future me problem
llvm_representation!(@int TypeTag, 8);
llvm_representation!(@int ClassId, 16);
llvm_representation!(@int Identifier, 32);
llvm_representation!(@int TypeId, 32);
// TODO probably separate representations for function and non-function values?
llvm_representation!(@ptr<T> *const T);
llvm_representation!(@ptr<T> Option<*const T>);
