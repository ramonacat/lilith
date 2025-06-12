use inkwell::IntPredicate;

use super::TypeStoreOpaquePointer;
use crate::codegen::{
    ContextErgonomics as _,
    context::{CodegenContext, type_maker::TypeDeclaration},
    module,
};

pub(super) fn make_add<'ctx>(
    codegen_context: &CodegenContext<'ctx>,
    module_builder: &module::ModuleBuilder<'ctx, '_>,
    type_store: TypeStoreOpaquePointer<'ctx>,
) {
    module_builder.build_function(
        "add",
        module::FunctionVisibility::Public,
        // TODO consider createing our own simpler API for declaring types?
        codegen_context
            .type_maker()
            .make_function(None, &[TypeDeclaration::Pointer]),
        |function, codegen_context| {
            let builder = codegen_context.llvm_context().create_builder();
            let entry = codegen_context
                .llvm_context()
                .append_basic_block(function, "entry");

            builder.position_at_end(entry);

            let store_length = type_store.get_length(&builder);
            let store_capacity = type_store.get_capacity(&builder);
            let store_types = type_store.get_types(&builder);

            let is_capacity_too_small = builder
                .build_int_compare(
                    IntPredicate::ULE,
                    store_capacity,
                    store_length,
                    "is_at_capacity",
                )
                .unwrap();
            let add_capacity_block = codegen_context
                .llvm_context()
                .append_basic_block(function, "add_capacity");
            let continue_block = codegen_context
                .llvm_context()
                .append_basic_block(function, "continue");
            builder
                .build_conditional_branch(is_capacity_too_small, add_capacity_block, continue_block)
                .unwrap();

            builder.position_at_end(add_capacity_block);

            let new_capacity = builder
                .build_int_mul(
                    store_capacity,
                    codegen_context.llvm_context().const_u32(2),
                    "new_capacity",
                )
                .unwrap();

            builder
                .build_store(type_store.get_capacity_ptr(&builder), new_capacity)
                .unwrap();

            let new_types = builder
                .build_array_malloc(
                    codegen_context.value_types().llvm_type(),
                    new_capacity,
                    "new_types",
                )
                .unwrap();

            builder
                .build_memmove(new_types, 1, store_types, 1, store_capacity)
                .unwrap();

            builder
                .build_store(type_store.get_types_ptr(&builder), new_types)
                .unwrap();

            builder.build_unconditional_branch(continue_block).unwrap();
            builder.position_at_end(continue_block);

            let new_value_spot = unsafe {
                builder.build_gep(
                    codegen_context.value_types().llvm_type(),
                    store_types,
                    &[store_length],
                    "new_value_spot",
                )
            }
            .unwrap();

            builder
                .build_memmove(
                    new_value_spot,
                    1,
                    function.get_first_param().unwrap().into_pointer_value(),
                    1,
                    codegen_context.value_types().llvm_type().size_of().unwrap(),
                )
                .unwrap();

            let new_length = builder
                .build_int_add(
                    store_length,
                    codegen_context.llvm_context().const_u32(1),
                    "added_length",
                )
                .unwrap();
            builder
                .build_store(type_store.get_length_ptr(&builder), new_length)
                .unwrap();

            builder.build_return(None).unwrap();
        },
    );
}
