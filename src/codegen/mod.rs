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

use context::{Function, Procedure as _};
use context_ergonomics::ContextErgonomics;
use inkwell::{builder::Builder, context::Context};
use llvm_struct::{opaque_struct::LlvmArray, representations::ConstOrValue};
use module::built_module::ModuleInterface as _;
use type_store::TypeStoreInterface;
use types::{
    classes::ClassId,
    functions::{FunctionArgument, FunctionSignatureOpaque, FunctionSignatureProvider},
    values::{ValueOpaque, ValueOpaquePointer, ValueProvider},
};

use crate::bytecode::{ByteCode, ConstValue, Expression, Identifier, TypeTag};

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
        context: &'ctx Context,
    ) -> ValueOpaquePointer<'ctx> {
        match expression {
            Expression::Add(left, right) => {
                // TODO we should check if either of the values implements an interface that allows
                // for the desired addition and execute on it, otherwise throw an error
                let left = self.build_value(left, builder, context);
                let right = self.build_value(right, builder, context);

                let result_value = builder
                    .build_int_add(left.get_raw(builder), right.get_raw(builder), "sum_value")
                    .unwrap();

                // TODO the .llvm_context here is needed because the value needs to know the
                // context type, but perhaps we can switch up to dyn or something there to side-step the
                // issue (I don't think the value should really have the knowledge of context type)
                ValueProvider::new(context).make_value(
                    builder,
                    ValueOpaque {
                        tag: ConstOrValue::Const(TypeTag::U64),
                        unused_0: ConstOrValue::Const(0),
                        class_id: ConstOrValue::Const(ClassId::none()),
                        unused_1: ConstOrValue::Const(0),
                        raw: ConstOrValue::Value(result_value),
                    },
                )
            }
            Expression::Assignment(binding, value) => {
                let expression = self.build_value(value, builder, context);
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

        builtins::register(&execution_engine, &module, self.context);

        let type_store_module = type_store::register(self.context);
        let type_store_api: TypeStoreInterface =
            TypeStoreInterface::expose_to(&module, self.context);

        let arguments = LlvmArray::const_length_new(
            [FunctionArgument {
                name: Identifier::new(1),
                type_id: TypeTag::U64.into(),
            }],
            self.context,
            &builder,
        );

        let signature_ptr = FunctionSignatureProvider::new(self.context).make_value(
            &builder,
            FunctionSignatureOpaque {
                class_id: ConstOrValue::Const(ClassId::none()),
                argument_count: ConstOrValue::Const(1),
                return_type_id: ConstOrValue::Const(TypeTag::U64.into()),
                arguments: ConstOrValue::Value(arguments.as_pointer()),
            },
        );

        let ptr_int = builder
            .build_ptr_to_int(
                signature_ptr.ptr(),
                self.context.i64_type(),
                "signature_value_int",
            )
            .unwrap();

        let signature_value = ValueProvider::new(self.context).make_value(
            &builder,
            ValueOpaque {
                tag: ConstOrValue::Const(TypeTag::FunctionSignature),
                unused_0: ConstOrValue::Const(0),
                class_id: ConstOrValue::Const(ClassId::none()),
                unused_1: ConstOrValue::Const(0),
                raw: ConstOrValue::Value(ptr_int),
            },
        );

        type_store_api.add.build_call(
            &builder,
            (self.context.const_u32(1024), signature_value.ptr()),
        );

        let _first_type = type_store_api
            .get
            .build_call(&builder, self.context.const_u64(1024));

        let mut result = None;
        for instruction in bytecode.instructions {
            result = Some(self.build_expression(instruction, &builder, self.context));
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
        context: &'ctx Context,
    ) -> ValueOpaquePointer<'ctx> {
        match value {
            crate::bytecode::Value::Literal(const_value) => {
                // TODO add some comfort methods for simple i*_type constants
                ValueProvider::new(self.context).make_value(
                    builder,
                    ValueOpaque {
                        tag: ConstOrValue::Const(TypeTag::U64),
                        unused_0: ConstOrValue::Const(0),
                        class_id: ConstOrValue::Const(ClassId::none()),
                        unused_1: ConstOrValue::Const(0),
                        raw: ConstOrValue::Const(match const_value {
                            ConstValue::U64(value) => value,
                        }),
                    },
                )
            }
            crate::bytecode::Value::Local(identifier) => {
                // TODO check here that the var actually exists
                *self.scope.get(&identifier).unwrap()
            }
            crate::bytecode::Value::Computed(expression) => {
                self.build_expression(*expression, builder, context)
            }
        }
    }
}
