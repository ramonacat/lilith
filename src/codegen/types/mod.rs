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

pub(super) fn register(context: &Context) -> Types {
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
pub(super) struct ValueTypes<'ctx> {
    value_type: ValueProvider<'ctx>,
    context: &'ctx Context,
}
// TODO is this a good API? Is the whole Types API situation sensible at all?
impl<'ctx> ValueTypes<'ctx> {
    pub(crate) const fn llvm_type(&self) -> inkwell::types::StructType<'ctx> {
        self.value_type.llvm_type()
    }
}

pub(super) struct Types<'ctx> {
    value: ValueTypes<'ctx>,
    function: FunctionTypes<'ctx>,
    primitive: PrimitiveTypes<'ctx>,
    #[allow(clippy::struct_field_names)]
    types: TypesTypes<'ctx>,
}

impl<'ctx> Types<'ctx> {
    pub(super) const fn value(&self) -> &ValueTypes<'ctx> {
        &self.value
    }

    pub(crate) const fn function(&self) -> &FunctionTypes<'ctx> {
        &self.function
    }

    pub(in crate::codegen) const fn primitive(&self) -> &PrimitiveTypes<'ctx> {
        &self.primitive
    }

    pub(crate) const fn types(&self) -> &TypesTypes<'ctx> {
        &self.types
    }
}
