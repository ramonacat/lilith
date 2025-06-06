use inkwell::{builder::Builder, context::Context, values::BasicValueEnum};

enum Expression {
    Literal(u64),
    Add(Box<Expression>, Box<Expression>),
}

pub struct ByteCode<'ctx> {
    instructions: Option<Expression>,
    context: &'ctx Context,
}

impl<'ctx> ByteCode<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            instructions: Some(Expression::Add(
                Box::new(Expression::Literal(123)),
                Box::new(Expression::Literal(456)),
            )),
        }
    }

    fn build_expression(
        &self,
        expression: Expression,
        builder: &Builder<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match expression {
            Expression::Literal(value) => self.context.i64_type().const_int(value, false).into(),
            Expression::Add(left, right) => {
                let left = self.build_expression(*left, builder).into_int_value();
                let right = self.build_expression(*right, builder).into_int_value();

                builder.build_int_add(left, right, "sum").unwrap().into()
            }
        }
    }

    pub fn execute(mut self) -> u64 {
        let module = self.context.create_module("main");
        let main = module.add_function("main", self.context.i64_type().fn_type(&[], false), None);
        let builder = self.context.create_builder();

        let entry_block = self.context.append_basic_block(main, "entry");
        builder.position_at_end(entry_block);

        let instructions = self.instructions.take().unwrap();
        let result = self.build_expression(instructions, &builder);
        builder.build_return(Some(&result)).unwrap();

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
