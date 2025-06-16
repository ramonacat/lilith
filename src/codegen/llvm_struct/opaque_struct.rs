// TODO rename this mod to llvm_array or smth
use std::marker::PhantomData;

use inkwell::{
    builder::Builder,
    context::Context,
    types::BasicType,
    values::{BasicValue, PointerValue},
};

use super::representations::{LlvmRepresentation, OperandValue};
use crate::codegen::{ConstOrValue, ContextErgonomics};

pub(in crate::codegen) struct LlvmArray<'ctx, T: LlvmRepresentation<'ctx>> {
    pointer: PointerValue<'ctx>,
    length: ConstOrValue<'ctx, u64>,

    phantom: PhantomData<T>,
}

impl<'ctx, T: LlvmRepresentation<'ctx> + OperandValue<'ctx>> LlvmArray<'ctx, T>
where
    <T as LlvmRepresentation<'ctx>>::LlvmValue: BasicValue<'ctx>,
    <T as LlvmRepresentation<'ctx>>::LlvmType: BasicType<'ctx>,
{
    pub(in crate::codegen) fn new_uninitialized(
        length: ConstOrValue<'ctx, u64>,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
    ) -> Self {
        let allocation = builder
            .build_array_malloc(
                T::llvm_type(context),
                match length {
                    ConstOrValue::Const(c) => context.const_u64(c),
                    ConstOrValue::Value(v) => v,
                },
                "array",
            )
            .unwrap();

        Self {
            pointer: allocation,
            length,
            phantom: PhantomData,
        }
    }

    pub(in crate::codegen) fn const_length_new<const LENGTH: usize>(
        values: [T; LENGTH],
        context: &'ctx Context,
        builder: &Builder<'ctx>,
    ) -> Self {
        let uninitialized =
            Self::new_uninitialized(ConstOrValue::Const(LENGTH as u64), context, builder);

        uninitialized.fill_const(values, context, builder);

        uninitialized
    }

    fn fill_const<const LENGTH: usize>(
        &self,
        values: [T; LENGTH],
        context: &'ctx Context,
        builder: &Builder<'ctx>,
    ) {
        for (index, value) in values.into_iter().enumerate() {
            let entry = self.raw_entry(index, context, builder);

            value.build_move_into(context, builder, entry);
        }
    }

    fn raw_entry(
        &self,
        index: usize,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
    ) -> PointerValue<'ctx> {
        if let ConstOrValue::Const(length) = self.length {
            assert!(index < usize::try_from(length).unwrap());
        }
        // TODO generate dynamic code to verify the index is in bounds in case self.length is a ::Value

        unsafe {
            builder.build_gep(
                T::llvm_type(context),
                self.pointer,
                &[context.const_u64(index as u64)],
                "array_entry",
            )
        }
        .unwrap()
    }

    pub(crate) const fn as_pointer(&self) -> PointerValue<'ctx> {
        self.pointer
    }
}
