use inkwell::{context::Context, module::Module, values::PointerValue};

use crate::codegen::{
    context::{Function, Procedure},
    llvm_struct::representations::OperandValue,
    module::ModuleBuilder,
};
pub(in crate::codegen) mod add;
pub(in crate::codegen) mod get;
pub(in crate::codegen) mod initializer;

use add::{TypeStoreAdd, make_add};
use get::{TypeStoreGet, make_get};
use initializer::make_type_store_initializer;

use super::module::{self, built_module::ModuleInterface};
use crate::{
    codegen::{
        context_ergonomics::ContextErgonomics,
        llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
        types::values::Value,
    },
    make_module_interface,
};

// TODO This is good enough for a prototype, but should really be replaced with a hashtable of some
// sorts, because otherwise finding a type will require a linear scan
llvm_struct! {
    struct TypeValue {
        id: u32,
        r#type: Value
    }
}

llvm_struct! {
    struct TypeStore {
        types: *const TypeValue,
        length: u32,
        capacity: u32
    }
}

// TODO should we just kill TypeStore and rename this to TypeStore?
make_module_interface!(@builder(TypeStoreBuilderImpl<'ctx>) struct TypeStoreInterface {
    add: TypeStoreAdd<'ctx>,
    get: TypeStoreGet<'ctx>
});

pub(in crate::codegen) struct TypeStoreBuilderImpl<'ctx> {
    type_store: TypeStoreOpaquePointer<'ctx>,
}

impl<'ctx> TypeStoreInterfaceBuilder<'ctx, '_> for TypeStoreBuilderImpl<'ctx> {
    fn add(
        &self,
        builder: &mut ModuleBuilder<'ctx>,
        // TODO remove this argument completely
        _context: &'ctx Context,
    ) -> TypeStoreAdd<'ctx> {
        make_add(builder, self.type_store)
    }

    fn get(
        &self,
        builder: &mut ModuleBuilder<'ctx>,
        // TODO remove this argument completely
        _context: &'ctx Context,
    ) -> TypeStoreGet<'ctx> {
        make_get(builder, self.type_store)
    }
}

pub(in crate::codegen) fn register(context: &Context) -> Module<'_> {
    // TODO this should be initialized at a higher level
    let module_builder_provider = module::register(context);

    // TODO we likely want to do some name mangling and have a naming convention and shit for the
    // builtin modules
    let mut module_builder = module_builder_provider.make_builder("type_store");
    let value_store_provider = TypeStoreProvider::new(context);

    // TODO separate add_global (that takes optional intializer, otherwise zeroes) and
    // add_global_import for importing global from other modules
    let type_store = module_builder.add_global(value_store_provider.llvm_type(), "type_store");
    type_store.set_initializer(&value_store_provider.llvm_type().const_zero());

    let type_store_initializer =
        make_type_store_initializer(&module_builder, type_store.as_pointer_value());

    module_builder.add_global_constructor(0, &type_store_initializer, Some(type_store));

    TypeStoreInterface::register(
        &TypeStoreBuilderImpl {
            type_store: TypeStoreProvider::new(context)
                .opaque_pointer(type_store.as_pointer_value()),
        },
        &mut module_builder,
        context,
    );

    module_builder.build()
}
