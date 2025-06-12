pub(in crate::codegen) mod builtins;
pub(in crate::codegen) mod context;
pub(in crate::codegen) mod context_ergonomics;
#[macro_use]
pub(in crate::codegen) mod llvm_struct;
pub(in crate::codegen) mod module;
pub(in crate::codegen) mod type_store;
pub(in crate::codegen) mod types;
pub(in crate::codegen) mod typestore;

use std::collections::HashMap;

use context::CodegenContext;
use context_ergonomics::ContextErgonomics;
use inkwell::{builder::Builder, context::Context};
use types::value::ValueOpaquePointer;

use crate::{
    bytecode::{ByteCode, ConstValue, Expression, Identifier, TypeTag},
    codegen::types::functions::ArgumentType,
};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    scope: HashMap<Identifier, ValueOpaquePointer<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            scope: HashMap::new(),
        }
    }

    fn build_expression(
        &mut self,
        expression: Expression,
        // TODO the following two should probably be fields on self (might need to introduce one
        // more level of abstraction tho, idk)
        builder: &Builder<'ctx>,
        codegen_context: &CodegenContext<'ctx>,
    ) -> ValueOpaquePointer<'ctx> {
        match expression {
            Expression::Add(left, right) => {
                // TODO we should check if either of the values implements an interface that allows
                // for the desired addition and execute on it, otherwise throw an error
                let left = self.build_value(left, builder, codegen_context);
                let right = self.build_value(right, builder, codegen_context);

                let result_value = builder
                    .build_int_add(left.get_raw(builder), right.get_raw(builder), "sum_value")
                    .unwrap();

                codegen_context
                    .primitive_types()
                    .make_u64(result_value, builder)
            }
            Expression::Assignment(binding, value) => {
                let expression = self.build_value(value, builder, codegen_context);
                self.scope.insert(binding, expression);
                expression
            }
        }
    }

    pub fn execute(mut self, bytecode: ByteCode) -> u64 {
        // TODO the main module should also use the api from crate::codegen::module, instead of
        // straight up calling the inkwell apis
        let module = self.context.create_module("main");
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::Aggressive)
            .unwrap();
        let builder = self.context.create_builder();

        let main = module.add_function(
            "main",
            // TODO we should use the type_maker here, but that requires first that CodegenContext
            // does not use builder
            self.context.i64_type().fn_type(&[], false),
            None,
        );
        let entry_block = self.context.append_basic_block(main, "entry");
        builder.position_at_end(entry_block);

        let codegen_context = CodegenContext::new(self.context, &builder, &module);

        builtins::register(&execution_engine, &module, &codegen_context);

        let arguments = codegen_context.function_types().make_function_arguments(
            &builder,
            &[ArgumentType::new(
                self.context.const_u32(1),
                self.context.const_u32(TypeTag::U64 as u32),
            )],
        );

        let signature = codegen_context.function_types().make_function_signature(
            &builder,
            self.context.const_u16(0),
            self.context.const_u32(TypeTag::U64 as u32),
            arguments,
        );

        let funcsig_slot = codegen_context.type_store().get_slot(257, &builder);
        builder
            .build_memmove(
                funcsig_slot,
                8,
                signature.ptr(),
                8,
                codegen_context
                    .function_types()
                    .signature_llvm_type()
                    .size_of()
                    .unwrap(),
            )
            .unwrap();

        let mut result = None;
        for instruction in bytecode.instructions {
            result = Some(self.build_expression(instruction, &builder, &codegen_context));
        }

        builder
            .build_call(
                module.get_function("debug_type_definition").unwrap(),
                &[funcsig_slot.into()],
                "type_definition_debug",
            )
            .unwrap();
        if let Some(result) = result {
            // TODO we should codegen an actual check here to ensure this is an actual u64 and
            // we're not just returning random whatever
            builder
                .build_return(Some(&result.get_raw(&builder)))
                .unwrap();
        } else {
            builder.build_return(None).unwrap();
        }

        module.print_to_stderr();
        module.verify().unwrap();

        let type_store_module = type_store::register(&codegen_context);
        module.link_in_module(type_store_module.build()).unwrap();

        execution_engine.run_static_constructors();
        let main = unsafe {
            execution_engine
                .get_function::<unsafe extern "C" fn() -> u64>("main")
                .unwrap()
        };
        execution_engine.run_static_destructors();

        unsafe { main.call() }
    }

    fn build_value(
        &mut self,
        value: crate::bytecode::Value,
        builder: &Builder<'ctx>,
        codegen_context: &CodegenContext<'ctx>,
    ) -> ValueOpaquePointer<'ctx> {
        match value {
            crate::bytecode::Value::Literal(const_value) => {
                // TODO add some comfort methods for simple i*_type constants
                codegen_context.value_types().make_value(
                    codegen_context.llvm_context().const_u8(TypeTag::U64 as u8),
                    codegen_context.llvm_context().const_u16(0),
                    codegen_context.llvm_context().const_u64(match const_value {
                        ConstValue::U64(value) => value,
                    }),
                    builder,
                )
            }
            crate::bytecode::Value::Local(identifier) => {
                // TODO check here that the var actually exists
                *self.scope.get(&identifier).unwrap()
            }
            crate::bytecode::Value::Computed(expression) => {
                self.build_expression(*expression, builder, codegen_context)
            }
        }
    }
}
