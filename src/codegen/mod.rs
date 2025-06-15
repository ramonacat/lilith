pub(in crate::codegen) mod builtins;
#[macro_use]
pub(in crate::codegen) mod context;
pub(in crate::codegen) mod context_ergonomics;
#[macro_use]
pub(in crate::codegen) mod llvm_struct;
pub(in crate::codegen) mod module;
pub(in crate::codegen) mod type_store;
pub(in crate::codegen) mod types;

use std::collections::HashMap;

use context::{
    AsLlvmContext, CodegenContext,
    type_maker::{Function, Procedure as _},
};
use context_ergonomics::ContextErgonomics;
use inkwell::{builder::Builder, context::Context};
use module::built_module::ModuleInterface as _;
use type_store::TypeStoreInterface;
use types::{
    functions::{FunctionArgumentOpaque, FunctionArgumentProvider, FunctionSignatureProvider},
    value::{ValueOpaquePointer, ValueProvider},
};

use crate::bytecode::{ByteCode, ConstValue, Expression, Identifier, TypeTag};

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    scope: HashMap<Identifier, ValueOpaquePointer<'ctx, &'ctx Context>>,
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
    ) -> ValueOpaquePointer<'ctx, &'ctx Context> {
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

        let codegen_context = CodegenContext::new(self.context);

        builtins::register(&execution_engine, &module, &codegen_context);

        let type_store_module = type_store::register(&codegen_context);
        // TODO instead of a HashMap do we want a strongly-typed struct here?
        let type_store_api: TypeStoreInterface =
            TypeStoreInterface::expose_to(&module, &codegen_context);

        let arguments = FunctionArgumentProvider::register(&codegen_context).make_array(
            &builder,
            &[FunctionArgumentOpaque {
                name: self.context.const_u32(1),
                type_id: self.context.const_u32(TypeTag::U64 as u32),
            }],
        );

        let signature_ptr = FunctionSignatureProvider::register(self.context).make_value(
            &builder,
            self.context.const_u16(0),
            self.context.const_u16(1), // argument count
            self.context.const_u32(TypeTag::U64 as u32),
            arguments,
        );

        let ptr_int = builder
            .build_ptr_to_int(
                signature_ptr.ptr(),
                self.context.i64_type(),
                "signature_value_int",
            )
            .unwrap();

        let signature_value = ValueProvider::register(self.context).make_value(
            &builder,
            self.context.const_u8(TypeTag::FunctionSignature as u8),
            self.context.const_u8(0),
            self.context.const_u16(0),
            self.context.const_u32(0),
            ptr_int,
        );

        type_store_api.add.build_call(
            &builder,
            (
                codegen_context.llvm_context().const_u32(1024),
                signature_value.ptr(),
            ),
        );

        let _first_type = type_store_api
            .get
            .build_call(&builder, self.context.const_u64(1024));

        let mut result = None;
        for instruction in bytecode.instructions {
            result = Some(self.build_expression(instruction, &builder, &codegen_context));
        }

        if let Some(result) = result {
            // TODO we should codegen an actual check here to ensure this is an actual u64 and
            // we're not just returning random whatever
            builder
                .build_return(Some(&result.get_raw(&builder)))
                .unwrap();
        } else {
            builder.build_return(None).unwrap();
        }

        type_store_module.print_to_stderr();
        type_store_module.verify().unwrap();

        module.print_to_stderr();
        module.verify().unwrap();

        module.link_in_module(type_store_module).unwrap();

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
    ) -> ValueOpaquePointer<'ctx, &'ctx Context> {
        match value {
            crate::bytecode::Value::Literal(const_value) => {
                // TODO add some comfort methods for simple i*_type constants
                ValueProvider::register(self.context).make_value(
                    builder,
                    codegen_context.llvm_context().const_u8(TypeTag::U64 as u8),
                    codegen_context.llvm_context().const_u8(0),
                    codegen_context.llvm_context().const_u16(0),
                    codegen_context.llvm_context().const_u32(0),
                    codegen_context.llvm_context().const_u64(match const_value {
                        ConstValue::U64(value) => value,
                    }),
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
