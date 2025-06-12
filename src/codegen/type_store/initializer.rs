use inkwell::values::PointerValue;

use crate::codegen::{ContextErgonomics as _, context::CodegenContext, module};

pub(super) fn make_type_store_initializer<'ctx>(
    codegen_context: &CodegenContext<'ctx>,
    module_builder: &module::ModuleBuilder<'ctx, '_>,
    type_store: PointerValue<'ctx>,
) -> inkwell::values::FunctionValue<'ctx> {
    module_builder.build_function(
        "type_store_initializer",
        module::FunctionVisibility::Private,
        codegen_context.type_maker().make_function(None, &[]),
        |function, codegen_context| {
            let entry = codegen_context
                .llvm_context()
                .append_basic_block(function, "entry");
            let builder = codegen_context.llvm_context().create_builder();
            builder.position_at_end(entry);

            let capacity = codegen_context.llvm_context().const_u32(1);
            let types = builder
                .build_array_malloc(
                    codegen_context.types_types().value().llvm_type(),
                    capacity,
                    "types",
                )
                .unwrap();

            codegen_context.types_types().store().provider().fill_in(
                type_store,
                &builder,
                types,
                codegen_context.llvm_context().const_u32(0),
                capacity,
            );
            builder.build_return(None).unwrap();
        },
    )
}
