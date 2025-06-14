mod debug;

use debug::debug_type_definition_impl;
use inkwell::{execution_engine::ExecutionEngine, module::Module};

use super::{
    context::{AsLlvmContext, type_maker::Procedure},
    types::value::Value,
};
use crate::make_function_type;

make_function_type!(DebugTypeDefinition, (value: *const Value));

pub(in crate::codegen) fn register<'ctx>(
    execution_engine: &ExecutionEngine<'ctx>,
    module: &Module<'ctx>,
    codegen_context: impl AsLlvmContext<'ctx>,
) {
    let debug_type_definition = module.add_function(
        "debug_type_definition",
        // this should really be a type argument, and not a value argument
        DebugTypeDefinition::llvm_type(codegen_context),
        None,
    );

    execution_engine.add_global_mapping(
        &debug_type_definition,
        debug_type_definition_impl as extern "C" fn(*const Value) as usize,
    );
}
