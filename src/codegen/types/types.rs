use inkwell::types::StructType;

use crate::codegen::{
    context::AsLlvmContext,
    type_store::{TypeStoreProvider, TypeValueProvider},
};

pub(in crate::codegen) struct TypesTypes<'ctx, TContext: AsLlvmContext<'ctx>> {
    type_store: TypeStoreTypes<'ctx, TContext>,
    type_value: TypeValueTypes<'ctx, TContext>,
}

impl<'ctx, TContext: AsLlvmContext<'ctx>> TypesTypes<'ctx, TContext> {
    pub fn new(context: TContext) -> Self {
        Self {
            type_store: TypeStoreTypes::new(context),
            type_value: TypeValueTypes::new(context),
        }
    }

    pub const fn store(&self) -> &TypeStoreTypes<'ctx, TContext> {
        &self.type_store
    }

    pub const fn value(&self) -> &TypeValueTypes<'ctx, TContext> {
        &self.type_value
    }
}

pub(in crate::codegen) struct TypeStoreTypes<'ctx, TContext: AsLlvmContext<'ctx>> {
    type_store_provider: TypeStoreProvider<'ctx, TContext>,
}

impl<'ctx, TContext: AsLlvmContext<'ctx>> TypeStoreTypes<'ctx, TContext> {
    fn new(context: TContext) -> Self {
        Self {
            type_store_provider: TypeStoreProvider::register(context),
        }
    }

    pub(crate) const fn llvm_type(&self) -> StructType<'ctx> {
        self.type_store_provider.llvm_type()
    }

    pub(crate) const fn provider(&self) -> &TypeStoreProvider<'ctx, TContext> {
        &self.type_store_provider
    }
}

pub(in crate::codegen) struct TypeValueTypes<'ctx, TContext: AsLlvmContext<'ctx>> {
    type_value_provider: TypeValueProvider<'ctx, TContext>,
}

impl<'ctx, TContext: AsLlvmContext<'ctx>> TypeValueTypes<'ctx, TContext> {
    fn new(context: TContext) -> Self {
        Self {
            type_value_provider: TypeValueProvider::register(context),
        }
    }

    pub(crate) const fn llvm_type(&self) -> StructType<'ctx> {
        self.type_value_provider.llvm_type()
    }

    // TODO do we want to expose the whole provider?
    pub(crate) const fn provider(&self) -> &TypeValueProvider<'ctx, TContext> {
        &self.type_value_provider
    }
}
