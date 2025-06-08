use inkwell::{builder::Builder, context::Context, module::Module};

use super::{types::Types, typestore::TypeStore};

pub struct CodegenContext<'ctx> {
    llvm_context: &'ctx Context,
    types: Types<'ctx>,
    type_store: TypeStore<'ctx>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub(crate) const fn llvm_context(&self) -> &'ctx Context {
        self.llvm_context
    }

    pub(crate) const fn types(&self) -> &Types<'ctx> {
        &self.types
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
        }
    }

    pub(crate) const fn type_store(&self) -> &TypeStore<'ctx> {
        &self.type_store
    }
}
