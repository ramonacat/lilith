use inkwell::{module::Module, values::PointerValue};

use super::{
    context::CodegenContext,
    module::{self, GlobalConstructorOpaque},
};
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

// TODO we should provide an API here that will insert the declarations of the API for the type
// store into another module, so it can be used there as well
impl<'ctx> TypeStoreModule<'ctx> {
    // TODO we should actually expose an interface here that will allow registering type
    // definitions
    fn new(codegen_context: &CodegenContext<'ctx>) -> Self {
        // TODO this should be a part of codegen_context prolly?
        let module_builder_provider = module::register(codegen_context);
        let mut module_builder = module_builder_provider.make_builder("type_store");
        let type_store_provider = TypeStoreProvider::register(codegen_context.llvm_context());
        let type_value_provider = TypeValueProvider::register(codegen_context.llvm_context());

        let type_store = module_builder.add_global(type_store_provider.llvm_type(), "type_store");
        type_store.set_initializer(&type_store_provider.llvm_type().const_zero());

        let type_store_initializer = make_type_store_initializer(
            codegen_context,
            &module_builder,
            &type_store_provider,
            type_store.as_pointer_value(),
            &type_value_provider,
        );

        module_builder.add_global_constructor(GlobalConstructorOpaque {
            // TODO add a method to make it easier to create constants
            priority: codegen_context
                .llvm_context()
                .i32_type()
                .const_int(0, false),
            target: type_store_initializer.as_global_value().as_pointer_value(),
            initialized_value: type_store.as_pointer_value(),
        });

        let module = module_builder.build();

        module.print_to_stderr();
        module.verify().unwrap();

        Self {
            // TODO we likely want to do some name mangling and have a naming convention and shit for the
            // builtin modules
            module,
        }
    }

    // TODO this should return two things in fact - one is the module, another is the object that
    // will allow declaring externs for the API exposed here
    pub(in crate::codegen) fn build(self) -> Module<'ctx> {
        self.module
    }
}

fn make_type_store_initializer<'ctx>(
    codegen_context: &CodegenContext<'ctx>,
    module_builder: &module::ModuleBuilder<'ctx>,
    type_store_provider: &TypeStoreProvider<'ctx>,
    type_store: PointerValue<'ctx>,
    type_value_provider: &TypeValueProvider<'ctx>,
) -> inkwell::values::FunctionValue<'ctx> {
    let type_store_initializer = module_builder.add_function(
        "type_store_initializer",
        codegen_context
            .llvm_context()
            .void_type()
            .fn_type(&[], false),
    );

    let entry = codegen_context
        .llvm_context()
        .append_basic_block(type_store_initializer, "entry");
    let builder = codegen_context.llvm_context().create_builder();
    builder.position_at_end(entry);

    let capacity = codegen_context
        .llvm_context()
        .i32_type()
        .const_int(1, false);
    let types = builder
        .build_array_malloc(type_value_provider.llvm_type(), capacity, "types")
        .unwrap();

    type_store_provider.fill_in(
        type_store,
        &builder,
        types,
        codegen_context
            .llvm_context()
            .i32_type()
            .const_int(0, false),
        capacity,
    );
    // TODO actually initialize the type store...
    builder.build_return(None).unwrap();

    type_store_initializer
}
