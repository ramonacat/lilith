use inkwell::{
    module::{Linkage, Module},
    values::PointerValue,
};

use super::context::CodegenContext;
use crate::codegen::{
    context_ergonomics::ContextErgonomics,
    llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
    types::value::Value,
};

llvm_struct! {
    struct GlobalConstructor {
        priority: u32,
        target: *const fn(),
        initialized_value: Option<*const ()>
    }
}

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
        let type_store_provider = TypeStoreProvider::register(codegen_context.llvm_context());
        let global_constructor_provider =
            GlobalConstructorProvider::register(codegen_context.llvm_context());

        let module = codegen_context.llvm_context().create_module("type_store");
        let type_store = module.add_global(type_store_provider.llvm_type(), None, "type_store");
        type_store.set_initializer(&type_store_provider.llvm_type().const_zero());

        let type_store_initializer = module.add_function(
            "type_store_initializer",
            codegen_context
                .llvm_context()
                .void_type()
                .fn_type(&[], false),
            None,
        );

        let global_constructors_array_type = global_constructor_provider.llvm_type().array_type(1);
        let global_constructors =
            module.add_global(global_constructors_array_type, None, "llvm.global_ctors");
        global_constructors.set_linkage(Linkage::Appending);
        global_constructors.set_initializer(
            &global_constructor_provider
                .llvm_type()
                .const_array(&[global_constructor_provider
                    .llvm_type()
                    .const_named_struct(&[
                        codegen_context
                            .llvm_context()
                            .i32_type()
                            .const_zero()
                            .into(),
                        type_store_initializer
                            .as_global_value()
                            .as_pointer_value()
                            .into(),
                        type_store.as_pointer_value().into(),
                    ])]),
        );

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
