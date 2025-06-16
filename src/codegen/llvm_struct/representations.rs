use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    types::{BasicType, IntType, PointerType},
    values::{BasicValue, IntValue, PointerValue},
};

use crate::{
    bytecode::{Identifier, TypeId, TypeTag},
    codegen::types::classes::ClassId,
};

pub(in crate::codegen) enum ConstOrValue<'ctx, T>
where
    T: LlvmRepresentation<'ctx> + Sized,
{
    Const(T),
    Value(T::LlvmValue),
}

// TODO is there anything we can do so context doesn't need to be passed all the time?
pub(in crate::codegen) trait LlvmRepresentation<'ctx>: Sized {
    type LlvmValue: BasicValue<'ctx>;
    type LlvmType: BasicType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType;
    fn build_store_into(
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        target: PointerValue<'ctx>,
        raw: &ConstOrValue<'ctx, Self>,
    );

    fn assert_valid(context: &'ctx Context, value: &ConstOrValue<'ctx, Self>);
}

macro_rules! llvm_representation {
    (@int $type:ty, $width: literal, $to_int: expr) => {
        impl<'ctx> LlvmRepresentation<'ctx> for $type {
            type LlvmValue = IntValue<'ctx>;
            type LlvmType = IntType<'ctx>;

            fn llvm_type(context: &'ctx inkwell::context::Context) -> Self::LlvmType {
                paste::paste!(context.[<i $width _type>]())
            }

            fn build_store_into(
                context: &'ctx Context,
                builder: &Builder<'ctx>,
                target: PointerValue<'ctx>,
                raw: &ConstOrValue<'ctx, Self>
            ) {
                let value = match raw {
                    ConstOrValue::Value(value) => *value,
                    ConstOrValue::Const(raw) => Self::llvm_type(context).const_int($to_int(raw), false)
                };

                builder.build_store(target, value).unwrap();
            }

            fn assert_valid(context: &'ctx inkwell::context::Context, value: &ConstOrValue<'ctx, Self>) {
                let ConstOrValue::Value(value) = value else {
                    return;
                };

                assert!(
                    value.get_type().get_bit_width() == Self::llvm_type(context).get_bit_width(),
                    "expected {} bit value, got {} bit",
                    Self::llvm_type(context).get_bit_width(),
                    value.get_type().get_bit_width()
                );
            }
        }
    };
    (@ptr<$generic:ident> $type:ty, $to_int:expr) => {
        impl<'ctx, $generic> LlvmRepresentation<'ctx> for $type {
            type LlvmValue = PointerValue<'ctx>;
            type LlvmType = PointerType<'ctx>;

            fn llvm_type(context: &'ctx inkwell::context::Context) -> Self::LlvmType {
                context.ptr_type(AddressSpace::default())
            }

            // TODO this should really be a method on ConstOrValue, that takes self, instead of
            // this unhinged raw dance?
            fn build_store_into(
                context: &'ctx Context,
                builder: &Builder<'ctx>,
                target: PointerValue<'ctx>,
                raw: &ConstOrValue<'ctx, Self>
            ) {
                let value = match raw {
                    ConstOrValue::Const(raw) => context
                        .i64_type()
                        .const_int(
                            $to_int(raw),
                            false
                        )
                        .const_to_pointer(Self::llvm_type(context)),
                    ConstOrValue::Value(value) => *value,
                };
                builder.build_store(target, value).unwrap();
            }

            fn assert_valid(_context: &'ctx inkwell::context::Context, _value: &ConstOrValue<'ctx, Self>) {
            }
        }
    };
}

llvm_representation!(@int u8, 8, |x:&u8| u64::from(*x));
llvm_representation!(@int u16, 16, |x:&u16| u64::from(*x));
llvm_representation!(@int u32, 32, |x:&u32| u64::from(*x));
llvm_representation!(@int u64, 64, |x:&u64| *x);
// TODO We have to check that the structs actually have a repr(C)/repr(transparent) and enums have
// repr(u*), but AFAIK that's only possible with proc macros, so this is a future me problem
// TODO perhaps the structs should use the type_maker instead, and llvm_representation should be
// reserved for primitives?
llvm_representation!(@int TypeTag, 8, |raw:&TypeTag| *raw as u64);
llvm_representation!(@int ClassId, 16, |raw:&ClassId| u64::from(raw.as_u16()));
llvm_representation!(@int Identifier, 32, |raw:&Identifier| u64::from(raw.as_u32()));
llvm_representation!(@int TypeId, 32, |raw:&TypeId| u64::from(raw.as_u32()));
// TODO probably separate representations for function and non-function values?
llvm_representation!(@ptr<T> *const T, |raw: &*const T| *raw as usize as u64);
llvm_representation!(@ptr<T> Option<*const T>, |raw: &Option<*const T>| raw.map_or(0, |x| x as usize as u64));
