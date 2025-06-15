use super::TypeStoreOpaquePointer;
use crate::codegen::{AsLlvmContext, ContextErgonomics, module, types::value::Value};

make_function_type!(TypeStoreGet, (id: u64): *const Value);

pub(super) fn make_get<'ctx, TContext: AsLlvmContext<'ctx>>(
    module_builder: &module::ModuleBuilder<'ctx, '_>,
    type_store: TypeStoreOpaquePointer<'ctx, TContext>,
) -> TypeStoreGet<'ctx> {
    module_builder.build_function::<_, _, TypeStoreGet>(|function, codegen_context, module| {
        let builder = codegen_context.llvm_context().create_builder();
        let entry = codegen_context
            .llvm_context()
            .append_basic_block(function, "entry");
        builder.position_at_end(entry);

        let elements = type_store.get_types(&builder);
        let element_type = codegen_context.types_types().value().llvm_type();

        let element_ptr = unsafe {
            builder.build_gep(
                element_type,
                elements,
                // TODO we should actually take the argument we got as the first arg and use it to
                // access the right element based on that ID
                &[codegen_context.const_u64(0)],
                "element",
            )
        }
        .unwrap();

        let result = codegen_context
            .types_types()
            .value()
            .provider()
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
