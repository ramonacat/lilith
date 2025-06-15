pub(in crate::codegen) mod functions;
#[allow(clippy::module_inception)]
pub(in crate::codegen) mod types;
pub(in crate::codegen) mod value;

use inkwell::context::Context;
use types::TypesTypes;

use super::context::AsLlvmContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// The value of 0 means no class
pub(super) struct ClassId(u16);

impl ClassId {}

pub(super) fn register(context: &Context) -> Types<&Context> {
    Types {
        types: TypesTypes::new(context),
    }
}

pub(super) struct Types<'ctx, TContext: AsLlvmContext<'ctx>> {
    #[allow(clippy::struct_field_names)]
    types: TypesTypes<'ctx, TContext>,
}

impl<'ctx, TContext: AsLlvmContext<'ctx>> Types<'ctx, TContext> {
    pub(crate) const fn types(&self) -> &TypesTypes<'ctx, TContext> {
        &self.types
    }
}
