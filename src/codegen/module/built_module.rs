use inkwell::{context::Context, module::Module};

use crate::codegen::module::ModuleBuilder;

pub(in crate::codegen) trait ModuleInterface<'ctx, 'codegen, TBuilder> {
    fn register(
        builder: &TBuilder,
        module_builder: &mut ModuleBuilder<'ctx>,
        context: &'ctx Context,
    ) -> Self;
    fn expose_to(other: &Module<'ctx>, context: &'ctx Context) -> Self;
}

#[macro_export]
macro_rules! make_module_interface {
    (@builder($builder_name:ty) struct $name:ident {
        $($field_name:ident: $field_type:ty),+
    }) => {
        paste::paste!{
            pub(in $crate::codegen) trait [<$name Builder>]<'ctx, 'codegen> {
                $(
                    fn $field_name(
                        &self,
                        builder: &mut $crate::codegen::module::ModuleBuilder<'ctx>,
                        context: &'ctx inkwell::context::Context
                    ) -> $field_type;
                )+
            }
        }

        pub(in $crate::codegen) struct $name<'ctx> {
            $(pub $field_name: $field_type),+
        }

        impl<'ctx, 'codegen> $crate::codegen::module::built_module::ModuleInterface<'ctx, 'codegen, $builder_name> for $name<'ctx> {
            fn register(
                builder: &$builder_name,
                module_builder: &mut $crate::codegen::module::ModuleBuilder<'ctx>,
                context: &'ctx inkwell::context::Context
        ) -> Self {
                Self {
                    $($field_name: builder.$field_name(module_builder, context)),+
                }
            }

            fn expose_to(
                other: &inkwell::module::Module<'ctx>,
                context: &'ctx inkwell::context::Context
            ) -> Self {
                Self {
                    $(
                        $field_name: <$field_type>::new(
                            other.add_function(
                                <$field_type>::NAME,
                                <$field_type>::llvm_type(context),
                                None
                            )
                        ),
                    )*
                }
            }
        }
    };
}
