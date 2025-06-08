mod context_ergonomics;
mod types;
mod typestore;

use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, IntValue},
};
use types::{ArgumentType, TypeTag};

use crate::{
    bytecode::{ByteCode, Expression, ResultId},
    types::{ConstValue, Value},
};

impl<'ctx> ConstValue {
    fn to_llvm(&self, context: &'ctx Context) -> IntValue<'ctx> {
        match self {
            Self::U64(value) => context.i64_type().const_int(*value, false),
        }
    }
}

impl<'ctx> Value {
    // TODO this API is kinda silly, it might make more sense to make this a part of CodeGen
    fn to_llvm(
        &self,
        context: &'ctx Context,
        scope: &HashMap<ResultId, BasicValueEnum<'ctx>>,
    ) -> BasicValueEnum<'ctx> {
        match self {
            Self::Const(const_value) => const_value.to_llvm(context).into(),
            Self::Opaque(id) => *scope.get(id).unwrap(),
        }
    }
}

pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    scope: HashMap<ResultId, BasicValueEnum<'ctx>>,
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
    ) -> BasicValueEnum<'ctx> {
        match expression {
            Expression::Literal(value) => value.to_llvm(self.context).into(),
            Expression::Add(left, right) => {
                // TODO ideally there should be some smart logic here that decides whether to
                // compile the addition or to bail out with an error on type mismatch
                builder
                    .build_int_add(
                        left.to_llvm(self.context, &self.scope).into_int_value(),
                        right.to_llvm(self.context, &self.scope).into_int_value(),
                        "sum",
                    )
                    .unwrap()
                    .into()
            }
            Expression::Assignment(scope_path, expression) => {
                let expression = self.build_expression(*expression, builder);
                self.scope.insert(ResultId(scope_path), expression);
                expression
            }
        }
    }

    pub fn execute(mut self, bytecode: ByteCode) -> u64 {
        let module = self.context.create_module("main");
        let main = module.add_function("main", self.context.i64_type().fn_type(&[], false), None);
        let builder = self.context.create_builder();

        let entry_block = self.context.append_basic_block(main, "entry");
        builder.position_at_end(entry_block);

        let types = types::register(self.context);
        let typestore = typestore::register(self.context, &builder, &module, &types);

        let arguments = types.make_function_arguments(
            &builder,
            &[ArgumentType::new(
                self.context.i32_type().const_int(1, false),
                self.context
                    .i32_type()
                    .const_int(TypeTag::U64 as u64, false),
            )],
        );
        types.make_function_signature(
            &builder,
            self.context.i16_type().const_int(0, false),
            self.context
                .i32_type()
                .const_int(TypeTag::U64 as u64, false),
            arguments,
            typestore.get_slot(257, &builder),
        );

        let mut result = None;
        for instruction in bytecode.instructions {
            result = Some(self.build_expression(instruction, &builder));
        }
        if let Some(result) = result {
            builder.build_return(Some(&result)).unwrap();
        } else {
            builder.build_return(None).unwrap();
        }

        module.print_to_stderr();
        module.verify().unwrap();

        let execution_engine =
            module.create_jit_execution_engine(inkwell::OptimizationLevel::Aggressive);
        let main = unsafe {
            execution_engine
                .unwrap()
                .get_function::<unsafe extern "C" fn() -> u64>("main")
                .unwrap()
        };

        unsafe { main.call() }
    }
}
