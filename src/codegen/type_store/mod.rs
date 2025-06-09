use inkwell::{module::Module, values::PointerValue};

use super::{context::CodegenContext, module};
use crate::codegen::{
    context_ergonomics::ContextErgonomics,
    llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
    types::value::Value,
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

pub(in crate::codegen) fn register<'ctx>(
    codegen_context: &CodegenContext<'ctx>,
) -> TypeStoreModule<'ctx> {
    TypeStoreModule::new(codegen_context)
}

#[allow(unused)]
pub(in crate::codegen) struct TypeStoreModule<'ctx> {
    module: Module<'ctx>,
    type_store_provider: TypeStoreProvider<'ctx>,
}

// TODO we should provide an API here that will insert the declarations of the API for the type
// store into another module, so it can be used there as well
impl<'ctx> TypeStoreModule<'ctx> {
    // TODO This method is such a mess, we should have some abstractions for registering global
    // constructors
    // TODO we should actually expose an interface here that will allow registering type
    // definitions
    fn new(codegen_context: &CodegenContext<'ctx>) -> Self {
        // TODO this should be a part of codegen_context prolly?
        let module_builder_provider = module::register(codegen_context);
        let mut module_builder = module_builder_provider.make_builder("type_store");
        let type_store_provider = TypeStoreProvider::register(codegen_context.llvm_context());

        let type_store =
            module_builder
                .module
                .add_global(type_store_provider.llvm_type(), None, "type_store");
        type_store.set_initializer(&type_store_provider.llvm_type().const_zero());

        // TODO this function probably would be more useful if it had any implementation
        let type_store_initializer = module_builder.module.add_function(
            "type_store_initializer",
            codegen_context
                .llvm_context()
                .void_type()
                .fn_type(&[], false),
            None,
        );

        module_builder.add_global_constructor((
            codegen_context
                .llvm_context()
                .i32_type()
                .const_int(0, false),
            type_store_initializer.as_global_value().as_pointer_value(),
            type_store.as_pointer_value(),
        ));

        let module = module_builder.build();

        module.print_to_stderr();
        module.verify().unwrap();

        Self {
            // TODO we likely want to do some name mangling and have a naming convention and shit for the
            // builtin modules
            module,
            type_store_provider,
        }
    }
}
