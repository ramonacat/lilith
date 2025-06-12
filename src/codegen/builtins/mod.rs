mod debug;

use debug::debug_type_definition_impl;
use inkwell::{execution_engine::ExecutionEngine, module::Module};

use super::{
    context::{CodegenContext, type_maker::TypeDeclaration},
    types::value::Value,
};

pub(in crate::codegen) fn register<'ctx>(
    execution_engine: &ExecutionEngine<'ctx>,
    module: &Module<'ctx>,
    codegen_context: &CodegenContext<'ctx>,
) {
    let debug_type_definition = module.add_function(
        "debug_type_definition",
        codegen_context
            .type_maker()
            .make_function(None, &[TypeDeclaration::Pointer]),
        None,
    );

    execution_engine.add_global_mapping(
        &debug_type_definition,
        debug_type_definition_impl as extern "C" fn(*const Value) as usize,
    );
}
