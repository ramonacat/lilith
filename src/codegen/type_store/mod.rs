mod add;
mod initializer;

use add::make_add;
use initializer::make_type_store_initializer;
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

pub(in crate::codegen) struct TypeStoreModule<'ctx> {
    module: Module<'ctx>,
}

// TODO currently we're storing straight up Value here, but we need to use TypeValue here, so that
// the users can actually get the types
// TODO add methods for getting the types actually
// TODO generalize the code so we can actually have vectors of any type
// TODO replace the uses of the old crate::typestore with this
// TODO create a debug method that will print out the length, capacity, and contained types
// TODO replace this with a hashmap, so that accessing a type isn't a linear-time ordeal
impl<'ctx> TypeStoreModule<'ctx> {
    fn new(codegen_context: &CodegenContext<'ctx>) -> Self {
        // TODO this should be a part of codegen_context prolly?
        let module_builder_provider = module::register(codegen_context);

        // TODO we likely want to do some name mangling and have a naming convention and shit for the
        // builtin modules
        let mut module_builder = module_builder_provider.make_builder("type_store");

        let type_store = module_builder.add_global(
            codegen_context.types_types().store().llvm_type(),
            "type_store",
        );
        type_store.set_initializer(
            &codegen_context
                .types_types()
                .store()
                .llvm_type()
                .const_zero(),
        );

        let type_store_initializer = make_type_store_initializer(
            codegen_context,
            &module_builder,
            type_store.as_pointer_value(),
        );

        module_builder.add_global_constructor(0, type_store_initializer, Some(type_store));
        make_add(
            codegen_context,
            &module_builder,
            codegen_context
                .types_types()
                .store()
                .provider()
                .opaque_pointer(type_store.as_pointer_value()),
        );

        let module = module_builder.build();

        // TODO these things should not be happening here, but instead at some final linking stage
        module.print_to_stderr();
        module.verify().unwrap();

        Self { module }
    }

    // TODO this should return two things in fact - one is the module, another is the object that
    // will allow declaring externs for the API exposed here
    pub(in crate::codegen) fn build(self) -> Module<'ctx> {
        self.module
    }
}
