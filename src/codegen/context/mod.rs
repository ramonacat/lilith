#[macro_use]
pub(in crate::codegen) mod type_maker;

use inkwell::context::Context;

use super::types::{Types, primitive::PrimitiveTypes};
use crate::codegen::types::types::TypesTypes;

pub(in crate::codegen) trait AsLlvmContext<'ctx>: Copy {
    fn llvm_context(self) -> &'ctx Context;
}

impl<'ctx> AsLlvmContext<'ctx> for &CodegenContext<'ctx> {
    fn llvm_context(self) -> &'ctx Context {
        self.llvm_context
    }
}

impl<'ctx> AsLlvmContext<'ctx> for &'ctx Context {
    fn llvm_context(self) -> &'ctx Context {
        self
    }
}

pub struct CodegenContext<'ctx> {
    llvm_context: &'ctx Context,
    types: Types<'ctx, &'ctx Context>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub(crate) const fn llvm_context(&self) -> &'ctx Context {
        self.llvm_context
    }

    pub(crate) const fn primitive_types(&self) -> &PrimitiveTypes<'ctx, &'ctx Context> {
        self.types.primitive()
    }

    pub(crate) const fn types_types(&self) -> &TypesTypes<'ctx, &'ctx Context> {
        self.types.types()
    }

    // TODO we should not take neither the builder nor module here, but instead generate a module
    // with static constructors and provide an API to accces the declarations here from other
    // modules as needed
    pub(crate) fn new(context: &'ctx Context) -> Self {
        let types = super::types::register(context);

        Self {
            llvm_context: context,
            types,
        }
    }
}
