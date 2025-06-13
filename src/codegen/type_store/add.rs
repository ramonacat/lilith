use inkwell::IntPredicate;

use super::TypeStoreOpaquePointer;
use crate::{
    bytecode::Value,
    codegen::{ContextErgonomics as _, context::CodegenContext, module},
};

make_function_type!(TypeStoreAdd, (id:u32, value: *const Value));

pub(super) fn make_add<'ctx>(
    module_builder: &mut module::ModuleBuilder<'ctx, '_>,
    type_store: TypeStoreOpaquePointer<'ctx>,
) {
    module_builder.build_procedure::<_, TypeStoreAdd>(
        module::FunctionVisibility::Public,
        |function, codegen_context, _module| {
            let builder = codegen_context.llvm_context().create_builder();
            let entry = codegen_context
                .llvm_context()
                .append_basic_block(function, "entry");

            builder.position_at_end(entry);

            let store_length = type_store.get_length(&builder);
            let store_capacity = type_store.get_capacity(&builder);

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

            expand_capacity(type_store, codegen_context, &builder);

            builder.build_unconditional_branch(continue_block).unwrap();
            builder.position_at_end(continue_block);

            let store_types = type_store.get_types(&builder);
            let new_value_spot = unsafe {
                builder.build_gep(
                    codegen_context.types_types().value().llvm_type(),
                    store_types,
                    &[store_length],
                    "new_value_spot",
                )
            }
            .unwrap();

            let new_value = builder
                .build_load(
                    codegen_context.value_types().llvm_type(),
                    function.get_nth_param(1).unwrap().into_pointer_value(),
                    "new_value",
                )
                .unwrap();

            codegen_context.types_types().value().provider().fill_in(
                new_value_spot,
                &builder,
                function.get_first_param().unwrap().into_int_value(),
                new_value.into_struct_value(),
            );

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

fn expand_capacity<'ctx>(
    type_store: TypeStoreOpaquePointer<'ctx>,
    codegen_context: &CodegenContext<'ctx>,
    builder: &inkwell::builder::Builder<'ctx>,
) {
    let store_capacity = type_store.get_capacity(builder);

    let new_capacity = builder
        .build_int_mul(
            store_capacity,
            codegen_context.llvm_context().const_u32(2),
            "new_capacity",
        )
        .unwrap();

    builder
        .build_store(type_store.get_capacity_ptr(builder), new_capacity)
        .unwrap();

    let new_types = builder
        .build_array_malloc(
            codegen_context.types_types().value().llvm_type(),
            new_capacity,
            "new_types",
        )
        .unwrap();

    let store_types = type_store.get_types(builder);

    builder
        .build_memmove(new_types, 1, store_types, 1, store_capacity)
        .unwrap();

    builder
        .build_store(type_store.get_types_ptr(builder), new_types)
        .unwrap();
}
