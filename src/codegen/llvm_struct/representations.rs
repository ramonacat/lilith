use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    types::{AnyType, IntType, PointerType},
    values::{AnyValue, IntValue, PointerValue},
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

pub(in crate::codegen) trait OperandValue<'ctx> {
    fn build_move_into(
        self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        target: PointerValue<'ctx>,
    );
}

// TODO is there anything we can do so context doesn't need to be passed all the time?
pub(in crate::codegen) trait LlvmRepresentation<'ctx>: Sized {
    type LlvmValue: AnyValue<'ctx>;
    type LlvmType: AnyType<'ctx>;

    fn llvm_type(context: &'ctx Context) -> Self::LlvmType;
    fn assert_valid(context: &'ctx Context, value: &ConstOrValue<'ctx, Self>);
}

/// This macro is intended to implement representations for types that can be expressed as
/// primitive values. For more complicated structures, `llvm_struct!` should be used, as it will be
/// actually able to handle the more complex conversion required there.
macro_rules! llvm_representation {
    (@int $type:ty, $width: literal, $to_int: expr) => {
        impl<'ctx> LlvmRepresentation<'ctx> for $type {
            type LlvmValue = IntValue<'ctx>;
            type LlvmType = IntType<'ctx>;

            fn llvm_type(context: &'ctx inkwell::context::Context) -> Self::LlvmType {
                paste::paste!(context.[<i $width _type>]())
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

        impl<'ctx> OperandValue<'ctx> for ConstOrValue<'ctx, $type> {
            fn build_move_into(
                self,
                context: &'ctx Context,
                builder: &Builder<'ctx>,
                target: PointerValue<'ctx>,
            ) {
                let value = match self {
                    ConstOrValue::Value(value) => value,
                    ConstOrValue::Const(raw) => <$type as LlvmRepresentation<'ctx>>::llvm_type(context)
                        .const_int($to_int(&raw), false)
                };

                builder.build_store(target, value).unwrap();
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

            fn assert_valid(_context: &'ctx inkwell::context::Context, _value: &ConstOrValue<'ctx, Self>) {
            }
        }

        impl<'ctx, $generic> OperandValue<'ctx> for ConstOrValue<'ctx, $type> {
            fn build_move_into(
                self,
                context: &'ctx Context,
                builder: &Builder<'ctx>,
                target: PointerValue<'ctx>,
            ) {
                let value = match self {
                    ConstOrValue::Const(raw) => context
                        .i64_type()
                        .const_int(
                            $to_int(&raw),
                            false
                        )
                        .const_to_pointer(<$type>::llvm_type(context)),
                    ConstOrValue::Value(value) => value,
                };
                builder.build_store(target, value).unwrap();
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
llvm_representation!(@int TypeTag, 8, |raw:&TypeTag| *raw as u64);
llvm_representation!(@int ClassId, 16, |raw:&ClassId| u64::from(raw.as_u16()));
llvm_representation!(@int Identifier, 32, |raw:&Identifier| u64::from(raw.as_u32()));
llvm_representation!(@int TypeId, 32, |raw:&TypeId| u64::from(raw.as_u32()));
llvm_representation!(@ptr<T> *const T, |raw: &*const T| *raw as usize as u64);
llvm_representation!(@ptr<T> Option<*const T>, |raw: &Option<*const T>| raw.map_or(0, |x| x as usize as u64));
