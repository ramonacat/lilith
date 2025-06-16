use inkwell::{AddressSpace, types::PointerType, values::PointerValue};

use super::representations::{LlvmRepresentation, OperandValue};
use crate::codegen::{ConstOrValue, ContextErgonomics as _};

#[repr(transparent)]
pub(in crate::codegen) struct RawConstArray<T>(*const T);

impl<T> RawConstArray<T> {
    pub unsafe fn iter(&self, length: usize) -> impl Iterator<Item = &T> {
        (0..length).map(|x| unsafe { &*self.0.add(x) })
    }
}

impl<'ctx, T: LlvmRepresentation<'ctx>> LlvmRepresentation<'ctx> for RawConstArray<T> {
    // TODO should this be a struct that also keeps the length, so it's not kept externally?
    type LlvmValue = PointerValue<'ctx>;
    type LlvmType = PointerType<'ctx>;

    fn llvm_type(context: &'ctx inkwell::context::Context) -> Self::LlvmType {
        context.ptr_type(AddressSpace::default())
    }

    fn assert_valid(_context: &'ctx inkwell::context::Context, value: &ConstOrValue<'ctx, Self>) {
        match value {
            ConstOrValue::Const(v) => assert!(!v.0.is_null()),
            ConstOrValue::Value(_) => {}
        }
    }
}

impl<'ctx, T: LlvmRepresentation<'ctx>> OperandValue<'ctx>
    for ConstOrValue<'ctx, RawConstArray<T>>
{
    fn build_move_into(
        self,
        context: &'ctx inkwell::context::Context,
        builder: &inkwell::builder::Builder<'ctx>,
        target: PointerValue<'ctx>,
    ) {
        let value = match self {
            ConstOrValue::Const(c) => context.const_ptr(c.0),
            ConstOrValue::Value(v) => v,
        };

        builder.build_store(target, value).unwrap();
    }
}
