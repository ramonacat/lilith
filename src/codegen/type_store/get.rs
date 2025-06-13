use super::TypeStoreOpaquePointer;
use crate::codegen::{module, types::value::Value};

make_function_type!(TypeStoreGet, (id: u64): *const Value);

pub(super) fn make_get<'ctx>(
    module_builder: &mut module::ModuleBuilder<'ctx, '_>,
    type_store: TypeStoreOpaquePointer<'ctx>,
) {
    module_builder.build_function::<_, _, TypeStoreGet>(
        module::FunctionVisibility::Public,
        |function, codegen_context, module| {
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
                    &[function.get_first_param().unwrap().into_int_value()],
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
        },
    );
}
