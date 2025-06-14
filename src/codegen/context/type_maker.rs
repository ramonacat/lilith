use inkwell::{
    builder::Builder,
    context::Context,
    types::FunctionType,
    values::{FunctionValue, GlobalValue},
};

use crate::codegen::llvm_struct::representations::LlvmRepresentation;

// TODO the Procedure/Function duality exists mostly because VoidType in inkwell is not BasicType
// and this complicates... everything. But maybe there's a way to avoid having those as separate
// traits?
// TODO There's a lot of repeated code all around here, clean it up
pub(in crate::codegen) trait Procedure<'ctx, TArguments> {
    const NAME: &'static str;

    fn llvm_type(context: &'ctx Context) -> FunctionType<'ctx>;

    fn new(value: FunctionValue<'ctx>) -> Self;
    #[allow(unused)] // TODO we gotta rizz up the API for ModuleGenerator so that it knows the
    // types exactly and can actually do strongly typed calls
    fn build_call(&self, builder: &Builder<'ctx>, arguments: TArguments);
    // TODO I don't love that API, it is required for global constructors, can we get rid of it? I
    // don't feel like we should expose the function pointer directly...
    fn as_global_value(&self) -> GlobalValue<'ctx>;
}

pub(in crate::codegen) trait Function<'ctx, TReturn: LlvmRepresentation<'ctx>, TArguments> {
    const NAME: &'static str;

    fn llvm_type(context: &'ctx Context) -> FunctionType<'ctx>;

    fn new(value: FunctionValue<'ctx>) -> Self;
    #[allow(unused)] // TODO we gotta rizz up the API for ModuleGenerator so that it knows the
    // types exactly and can actually do strongly typed calls
    fn build_call(&self, builder: &Builder<'ctx>, arguments: TArguments) -> TReturn::LlvmValue;
}

#[macro_export]
macro_rules! make_llvm_type_instance {
    ($context:expr, $type:ty) => {
        <$type as $crate::codegen::llvm_struct::representations::LlvmRepresentation>::llvm_type(
            $context,
        )
    };
}

#[macro_export]
macro_rules! make_llvm_value_type {
    ($type:ty) => {
        <$type as $crate::codegen::llvm_struct::representations::LlvmRepresentation<'ctx>>::LlvmValue
    };
}

#[macro_export]
macro_rules! make_function_type {
    ($name:ident, ($($argument_name:ident: $argument:ty),*)) => {
        pub(in $crate::codegen) struct $name<'ctx> {
            value: inkwell::values::FunctionValue<'ctx>
        }

        #[allow(unused_parens)]
        impl<'ctx> $crate::codegen::context::type_maker::Procedure<
            'ctx, ($($crate::make_llvm_value_type!($argument)),*)
        > for $name<'ctx> {
            const NAME: &'static str = stringify!($name);

            fn new(value: inkwell::values::FunctionValue<'ctx>) -> Self {
                Self { value }
            }

            // TODO can CodegenContext implement Context(if it's a trait) maybeee? or maybe we can
            // have some common trait, to just avoid going deep into properties at call sites
            fn llvm_type(context: &'ctx inkwell::context::Context) -> inkwell::types::FunctionType<'ctx> {
                context.void_type().fn_type(&[
                    $(
                        $crate::make_llvm_type_instance!(context, $argument).into()
                    ),*
                ], false)
            }

            fn build_call(
                &self,
                builder: &inkwell::builder::Builder<'ctx>,
                arguments: ($($crate::make_llvm_value_type!($argument)),*)
            ) {
                let ($($argument_name),*) = arguments;

                builder.build_call(
                    self.value,
                    &[
                        $($argument_name.into()),*
                    ],
                    stringify!($name)
                ).unwrap();
            }

            fn as_global_value(&self) -> inkwell::values::GlobalValue<'ctx> {
                self.value.as_global_value()
            }
        }
    };

    ($name:ident, ($($argument_name:ident: $argument:ty),*): $return_type:ty) => {
        pub(in $crate::codegen) struct $name<'ctx> {
            #[allow(unused)]
            value: inkwell::values::FunctionValue<'ctx>,
        }

        #[allow(unused_parens)]
        impl<'ctx> $crate::codegen::context::type_maker::Function<
            'ctx,
            $return_type,
            ($($crate::make_llvm_value_type!($argument)),*)
        > for $name<'ctx> {
            const NAME: &'static str = stringify!($name);

            // TODO can CodegenContext implement Context(if it's a trait) maybeee? or maybe we can
            // have some common trait, to just avoid going deep into properties at call sites
            fn llvm_type(context: &'ctx inkwell::context::Context) -> inkwell::types::FunctionType<'ctx> {
                let return_type = $crate::make_llvm_type_instance!(context, $return_type);

                return_type.fn_type(&[
                    $(
                        $crate::make_llvm_type_instance!(context, $argument).into()
                    ),*
                ], false)
            }

            fn new(value: inkwell::values::FunctionValue<'ctx>) -> Self {
                Self { value }
            }


            fn build_call(
                &self,
                builder: &inkwell::builder::Builder<'ctx>,
                arguments: ($($crate::make_llvm_value_type!($argument)),*)
            ) -> $crate::make_llvm_value_type!($return_type) {
                let ($($argument_name),*) = arguments;

                builder.build_call(
                    self.value,
                    &[
                        $($argument_name.into()),*
                    ],
                    stringify!($name)
                ).unwrap().try_as_basic_value().unwrap_left().try_into().unwrap()
            }
        }
    };
}
