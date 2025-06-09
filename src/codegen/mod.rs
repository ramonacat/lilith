pub(in crate::codegen) mod builtins;
pub(in crate::codegen) mod context;
pub(in crate::codegen) mod context_ergonomics;
#[macro_use]
pub(in crate::codegen) mod llvm_struct;
mod type_store;
pub(in crate::codegen) mod types;
pub(in crate::codegen) mod typestore;

use std::collections::HashMap;

use context::CodegenContext;
use inkwell::{builder::Builder, context::Context, values::PointerValue};
use types::value::TypeTag;

use crate::{
    bytecode::{ByteCode, Expression, ResultId},
    codegen::types::functions::ArgumentType,
    types::{ConstValue, Value},
};

impl<'ctx> ConstValue {
    fn to_llvm(
        &self,
        builder: &Builder<'ctx>,
        codegen_context: &CodegenContext<'ctx>,
    ) -> PointerValue<'ctx> {
        match self {
            Self::U64(value) => {
                let target = builder
                    .build_alloca(codegen_context.value_types().llvm_type(), "literal")
                    .unwrap();
                codegen_context
                    .primitive_types()
                    .make_const_u64(*value, builder, target);

                target
            }
        }
    }
}

impl<'ctx> Value {
    // TODO this API is kinda silly, it might make more sense to make this a part of CodeGen
    fn to_llvm(
        &self,
        scope: &HashMap<ResultId, PointerValue<'ctx>>,
        codegen_context: &CodegenContext<'ctx>,
        builder: &Builder<'ctx>,
    ) -> PointerValue<'ctx> {
        match self {
            Self::Const(value) => value.to_llvm(builder, codegen_context),
            Self::Opaque(id) => *scope.get(id).unwrap(),
        }
    }
}

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    scope: HashMap<ResultId, PointerValue<'ctx>>,
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
        builder: &Builder<'ctx>,
        codegen_context: &CodegenContext<'ctx>,
    ) -> PointerValue<'ctx> {
        match expression {
            Expression::Literal(value) => value.to_llvm(builder, codegen_context),
            Expression::Add(left, right) => {
                // TODO we should check if either of the values implements an interface that allows
                // for the desired addition and execute on it, otherwise throw an error
                let left = left.to_llvm(&self.scope, codegen_context, builder);
                let left = codegen_context.value_types().opaque(left).get_raw(builder);

                let right = right.to_llvm(&self.scope, codegen_context, builder);
                let right = codegen_context.value_types().opaque(right).get_raw(builder);

                let result_value = builder
                    .build_int_add(left.into_int_value(), right.into_int_value(), "sum_value")
                    .unwrap();

                let result = builder
                    .build_alloca(codegen_context.value_types().llvm_type(), "sum")
                    .unwrap();
                codegen_context
                    .primitive_types()
                    .make_u64(result_value, builder, result);

                result
            }
            Expression::Assignment(scope_path, expression) => {
                let expression = self.build_expression(*expression, builder, codegen_context);
                self.scope.insert(ResultId(scope_path), expression);
                expression
            }
        }
    }

    pub fn execute(mut self, bytecode: ByteCode) -> u64 {
        let module = self.context.create_module("main");
        let execution_engine = module
            .create_jit_execution_engine(inkwell::OptimizationLevel::Aggressive)
            .unwrap();
        let main = module.add_function("main", self.context.i64_type().fn_type(&[], false), None);
        let builder = self.context.create_builder();

        let entry_block = self.context.append_basic_block(main, "entry");
        builder.position_at_end(entry_block);

        let codegen_context = CodegenContext::new(self.context, &builder, &module);
        builtins::register(&execution_engine, &module, &codegen_context);

        let arguments = codegen_context.function_types().make_function_arguments(
            &builder,
            &[ArgumentType::new(
                self.context.i32_type().const_int(1, false),
                self.context
                    .i32_type()
                    .const_int(TypeTag::U64 as u64, false),
            )],
        );

        let funcsig_slot = codegen_context.type_store().get_slot(257, &builder);

        codegen_context.function_types().make_function_signature(
            &builder,
            self.context.i16_type().const_int(0, false),
            self.context
                .i32_type()
                .const_int(TypeTag::U64 as u64, false),
            arguments,
            funcsig_slot,
        );

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
            let result = codegen_context
                .value_types()
                .opaque(result)
                .get_raw(&builder);
            builder.build_return(Some(&result)).unwrap();
        } else {
            builder.build_return(None).unwrap();
        }

        module.print_to_stderr();
        module.verify().unwrap();

        let _type_store_module = type_store::register(&codegen_context);

        let main = unsafe {
            execution_engine
                .get_function::<unsafe extern "C" fn() -> u64>("main")
                .unwrap()
        };

        unsafe { main.call() }
    }
}
