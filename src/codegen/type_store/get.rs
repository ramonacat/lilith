use super::{TypeStoreOpaquePointer, TypeValueProvider};
use crate::codegen::{ContextErgonomics, module, types::values::Value};

make_function_type!(TypeStoreGet, (id: u64): *const Value);

pub(super) fn make_get<'ctx>(
    module_builder: &module::ModuleBuilder<'ctx>,
    type_store: TypeStoreOpaquePointer<'ctx>,
) -> TypeStoreGet<'ctx> {
    module_builder.build_function::<_, _, TypeStoreGet>(|function, context, module| {
        let builder = context.create_builder();
        let entry = context.append_basic_block(function, "entry");
        builder.position_at_end(entry);

        // TODO the types here should have some kinda array type, so we don't have to duck around
        // with the GEP manually
        let elements = type_store.get_types(&builder);
        let element_type = TypeValueProvider::new(context).llvm_type();

        let element_ptr = unsafe {
            builder.build_gep(
                element_type,
                elements,
                // TODO we should actually take the argument we got as the first arg and use it to
                // access the right element based on that ID
                &[context.const_u64(0)],
                "element",
            )
        }
        .unwrap();

        let result = TypeValueProvider::new(context)
            .opaque_pointer(element_ptr)
            .get_type_ptr(&builder);

        builder
            .build_call(
                module.get_function("debug_type_definition").unwrap(),
                &[result.into()],
                "type_definition_debug",
            )
            .unwrap();
        builder.build_return(Some(&result)).unwrap();
    })
}
