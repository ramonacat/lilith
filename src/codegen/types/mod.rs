pub(in crate::codegen) mod functions;
pub(in crate::codegen) mod primitive;
#[allow(clippy::module_inception)]
pub(in crate::codegen) mod types;
pub(in crate::codegen) mod value;

use functions::FunctionTypes;
use inkwell::context::Context;
use primitive::PrimitiveTypes;
use types::TypesTypes;
use value::ValueProvider;

use super::context::AsLlvmContext;
use crate::bytecode::TypeTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// The value of 0 means no class
pub(super) struct ClassId(u16);

impl ClassId {
    const fn none() -> Self {
        Self(0)
    }
}

pub(super) fn register(context: &Context) -> Types<&Context> {
    let value_type = ValueProvider::register(context);
    let value_types = ValueTypes {
        value_type,
        context,
    };

    Types {
        value: value_types,
        function: FunctionTypes::new(context, value_types),
        primitive: PrimitiveTypes::new(value_types),
        types: TypesTypes::new(context),
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ValueTypes<'ctx, TContext: AsLlvmContext<'ctx>> {
    value_type: ValueProvider<'ctx, TContext>,
    context: TContext,
}
// TODO is this a good API? Is the whole Types API situation sensible at all?
impl<'ctx, TContext: AsLlvmContext<'ctx>> ValueTypes<'ctx, TContext> {
    pub(crate) const fn llvm_type(&self) -> inkwell::types::StructType<'ctx> {
        self.value_type.llvm_type()
    }
}

pub(super) struct Types<'ctx, TContext: AsLlvmContext<'ctx>> {
    value: ValueTypes<'ctx, TContext>,
    function: FunctionTypes<'ctx, TContext>,
    primitive: PrimitiveTypes<'ctx, TContext>,
    #[allow(clippy::struct_field_names)]
    types: TypesTypes<'ctx, TContext>,
}

impl<'ctx, TContext: AsLlvmContext<'ctx>> Types<'ctx, TContext> {
    pub(super) const fn value(&self) -> &ValueTypes<'ctx, TContext> {
        &self.value
    }

    pub(crate) const fn function(&self) -> &FunctionTypes<'ctx, TContext> {
        &self.function
    }

    pub(in crate::codegen) const fn primitive(&self) -> &PrimitiveTypes<'ctx, TContext> {
        &self.primitive
    }

    pub(crate) const fn types(&self) -> &TypesTypes<'ctx, TContext> {
        &self.types
    }
}
