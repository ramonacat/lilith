use inkwell::module::Module;

use crate::codegen::{context::CodegenContext, module::ModuleBuilder};

pub(in crate::codegen) trait ModuleInterface<'ctx, TBuilder> {
    fn register(
        builder: &TBuilder,
        module_builder: &mut ModuleBuilder<'ctx, '_>,
        codegen_context: &CodegenContext<'ctx>,
    ) -> Self;
    fn expose_to(other: &Module<'ctx>, context: &'ctx inkwell::context::Context) -> Self;
}

#[macro_export]
macro_rules! make_module_interface {
    (@builder($builder_name:ty) struct $name:ident {
        $($field_name:ident: $field_type:ty),+
    }) => {
        paste::paste!{
            pub(in $crate::codegen) trait [<$name Builder>]<'ctx> {
                $(
                    fn $field_name(
                        &self,
                        builder: &mut $crate::codegen::module::ModuleBuilder<'ctx, '_>,
                        codegen_context: &$crate::codegen::context::CodegenContext<'ctx>
                    ) -> $field_type;
                )+
            }
        }

        pub(in $crate::codegen) struct $name<'ctx> {
            $(pub $field_name: $field_type),+
        }

        impl<'ctx> $crate::codegen::module::built_module::ModuleInterface<'ctx, $builder_name> for $name<'ctx> {
            fn register(
                builder: &$builder_name,
                module_builder: &mut $crate::codegen::module::ModuleBuilder<'ctx, '_>,
                codegen_context: &$crate::codegen::context::CodegenContext<'ctx>
        ) -> Self {
                Self {
                    $($field_name: builder.$field_name(module_builder, codegen_context)),+
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
