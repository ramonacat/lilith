pub(in crate::codegen) mod type_maker;

use inkwell::{builder::Builder, context::Context, module::Module};

use super::{
    types::{Types, ValueTypes, functions::FunctionTypes, primitive::PrimitiveTypes},
    typestore::TypeStore,
};
use crate::codegen::{context::type_maker::TypeMaker, types::types::TypesTypes};

pub struct CodegenContext<'ctx> {
    llvm_context: &'ctx Context,
    type_maker: TypeMaker<'ctx>,
    types: Types<'ctx>,
    // TODO this is the bad old typestore, remove it
    type_store: TypeStore<'ctx>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub(crate) const fn llvm_context(&self) -> &'ctx Context {
        self.llvm_context
    }

    pub(crate) const fn type_maker(&self) -> &TypeMaker<'ctx> {
        &self.type_maker
    }

    pub(crate) const fn function_types(&self) -> &FunctionTypes<'ctx> {
        self.types.function()
    }

    pub(crate) const fn primitive_types(&self) -> &PrimitiveTypes<'ctx> {
        self.types.primitive()
    }

    pub(crate) const fn value_types(&self) -> &ValueTypes<'ctx> {
        self.types.value()
    }

    pub(crate) const fn types_types(&self) -> &TypesTypes<'ctx> {
        self.types.types()
    }

    // TODO we should not take neither the builder nor module here, but instead generate a module
    // with static constructors and provide an API to accces the declarations here from other
    // modules as needed
    pub(crate) fn new(
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        module: &Module<'ctx>,
    ) -> Self {
        let types = super::types::register(context);
        let type_store = super::typestore::register(context, builder, module, &types);

        Self {
            llvm_context: context,
            types,
            type_store,
            type_maker: TypeMaker::new(context),
        }
    }

    pub(crate) const fn type_store(&self) -> &TypeStore<'ctx> {
        &self.type_store
    }
}
