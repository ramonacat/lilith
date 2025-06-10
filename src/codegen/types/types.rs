use inkwell::{context::Context, types::StructType};

use crate::codegen::type_store::{TypeStoreProvider, TypeValueProvider};

pub(in crate::codegen) struct TypesTypes<'ctx> {
    type_store: TypeStoreTypes<'ctx>,
    type_value: TypeValueTypes<'ctx>,
}

impl<'ctx> TypesTypes<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            type_store: TypeStoreTypes::new(context),
            type_value: TypeValueTypes::new(context),
        }
    }

    pub const fn store(&self) -> &TypeStoreTypes<'ctx> {
        &self.type_store
    }

    pub const fn value(&self) -> &TypeValueTypes<'ctx> {
        &self.type_value
    }
}

pub(in crate::codegen) struct TypeStoreTypes<'ctx> {
    type_store_provider: TypeStoreProvider<'ctx>,
}

impl<'ctx> TypeStoreTypes<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        Self {
            type_store_provider: TypeStoreProvider::register(context),
        }
    }

    pub(crate) const fn llvm_type(&self) -> StructType<'ctx> {
        self.type_store_provider.llvm_type()
    }

    pub(crate) const fn provider(&self) -> &TypeStoreProvider<'ctx> {
        &self.type_store_provider
    }
}

pub(in crate::codegen) struct TypeValueTypes<'ctx> {
    type_value_provider: TypeValueProvider<'ctx>,
}

impl<'ctx> TypeValueTypes<'ctx> {
    fn new(context: &'ctx Context) -> Self {
        Self {
            type_value_provider: TypeValueProvider::register(context),
        }
    }

    pub(crate) const fn llvm_type(&self) -> StructType<'ctx> {
        self.type_value_provider.llvm_type()
    }
}
