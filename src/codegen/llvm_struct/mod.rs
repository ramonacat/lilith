pub(in crate::codegen) mod basic_value_enum;
pub(in crate::codegen) mod representations;

#[macro_export]
macro_rules! get_field_inner {
    ($index:expr, $field_name:ident: $field_type:ty) => {
        paste::paste! {
            #[allow(unused)]
            pub fn [<get_ $field_name _ptr>](
                &self,
                builder: &inkwell::builder::Builder<'ctx>
            ) -> inkwell::values::PointerValue<'ctx> {
                builder.build_struct_gep(
                    self.llvm_type,
                    self.pointer,
                    $index,
                    stringify!([<$field_name _gep>])
                ).unwrap()
            }

            #[allow(unused)]
            pub fn [<get_ $field_name>](
                self,
                builder: &inkwell::builder::Builder<'ctx>
            ) -> <$field_type as LlvmRepresentation<'ctx>>::LlvmValue {
                let struct_gep = self.[<get_ $field_name _ptr>](builder);

                (
                    builder.build_load(
                        <$field_type>::llvm_type(&self.context),
                        struct_gep,
                        stringify!($field_name),
                    )
                    .unwrap()
                ).into_value()
            }
        }
    };
}

#[macro_export]
macro_rules! get_field {
    ($field_name_first:ident: $field_type_first:ty, $($field_name:ident: $field_type:ty),*) => {
        get_field_inner!(0u32, $field_name_first: $field_type_first);

        get_field!(1u32, $($field_name: $field_type),*);
    };
    ($count:expr, $field_name_first:ident: $field_type_first:ty, $($field_name:ident: $field_type:ty),*) => {
        get_field_inner!($count, $field_name_first: $field_type_first);

        get_field!(1u32+$count, $($field_name: $field_type),*);
    };
    ($count:expr, $field_name:ident: $field_type:ty) => {
        get_field_inner!($count, $field_name: $field_type);
    };
}

#[macro_export]
macro_rules! llvm_struct {
    (struct $name:ident { $($field_name:ident: $field_type:ty),+ }) => {
        #[repr(C)]
        #[allow(unused)]
        pub(in $crate::codegen) struct $name {
            $(pub(in $crate::codegen) $field_name: $field_type),+
        }

        impl<'ctx> LlvmRepresentation<'ctx> for $name {
            type LlvmType = inkwell::types::StructType<'ctx>;
            type LlvmValue = inkwell::values::StructValue<'ctx>;

            fn assert_valid(_context: &impl $crate::codegen::context::AsLlvmContext<'ctx>, _value: Self::LlvmValue) {}

            fn llvm_type(context: &impl $crate::codegen::context::AsLlvmContext<'ctx>) -> Self::LlvmType {
                context.llvm_context().get_struct_type(stringify!($name)).unwrap()
            }
        }

        paste::paste! {
            #[allow(unused)]
            pub(in $crate::codegen) struct [<$name Opaque>]<'ctx> {
                $(pub(in $crate::codegen) $field_name: <$field_type as LlvmRepresentation<'ctx>>::LlvmValue),+
            }

            #[derive(Debug, Clone, Copy)]
            pub(in $crate::codegen) struct [<$name OpaquePointer>]<'ctx, TContext: $crate::codegen::context::AsLlvmContext<'ctx>> {
                pointer: inkwell::values::PointerValue<'ctx>,
                context: TContext,
                llvm_type: inkwell::types::StructType<'ctx>,
            }

            impl<'ctx, TContext: $crate::codegen::context::AsLlvmContext<'ctx>> [<$name OpaquePointer>]<'ctx, TContext> {
                #[allow(unused)]
                const fn new(
                    pointer: PointerValue<'ctx>,
                    context: TContext,
                    llvm_type: inkwell::types::StructType<'ctx>,
                ) -> Self {
                    Self { pointer, context, llvm_type }
                }

                // TODO get rid of this method, this is a hack around the bad typestore impl
                #[allow(unused)]
                pub(in $crate::codegen) const fn ptr(&self) -> inkwell::values::PointerValue<'ctx> {
                    self.pointer
                }

                $crate::get_field! { $($field_name: $field_type),* }
            }

            #[derive(Debug, Clone, Copy)]
            #[allow(unused)]
            pub(in $crate::codegen) struct [<$name Provider>]<'ctx, TContext: $crate::codegen::context::AsLlvmContext<'ctx>> {
                llvm_type: inkwell::types::StructType<'ctx>,
                context: TContext
            }

            #[allow(unused)]
            impl<'ctx, TContext: $crate::codegen::context::AsLlvmContext<'ctx>> [<$name Provider>]<'ctx, TContext> {
                pub(in $crate::codegen) fn register(context: TContext) -> Self {
                    let llvm_context = context.llvm_context();
                    let llvm_type = llvm_context.named_struct(stringify!($name), &[
                        $(<$field_type>::llvm_type(&context).into()),+
                    ]);
                    Self { llvm_type, context }
                }

                #[allow(unused)]
                pub(in $crate::codegen) const fn opaque_pointer(
                    &self,
                    pointer: PointerValue<'ctx>
                ) -> [<$name OpaquePointer>]<'ctx, TContext> {
                    [<$name OpaquePointer>]::new(pointer, self.context, self.llvm_type)
                }

                #[allow(unused)]
                pub(in $crate::codegen) fn opaque_to_value(
                    &self,
                    opaque: [<$name Opaque>]<'ctx>
                ) -> inkwell::values::StructValue<'ctx> {
                    self.llvm_type.const_named_struct(&[
                        $(opaque.$field_name.into()),+
                    ])
                }

                #[allow(clippy::too_many_arguments)]
                pub(in $crate::codegen) fn fill_in(
                    &self,
                    target: inkwell::values::PointerValue<'ctx>,
                    builder: &inkwell::builder::Builder<'ctx>,
                    $($field_name: <$field_type as LlvmRepresentation<'ctx>>::LlvmValue),+
                ) {
                    let mut index:u32 = 0;

                    $({
                        <$field_type as LlvmRepresentation<'ctx>>::assert_valid(&self.context, $field_name);

                        let field_gep = builder
                            .build_struct_gep(
                                self.llvm_type,
                                target,
                                index,
                                stringify!([<$field_name _gep>])
                            )
                            .unwrap();

                        builder.build_store(field_gep, $field_name).unwrap();

                        // TODO should we make this comptime by using macro recursion tricks?
                        index += 1;
                    })+
                }

                pub(in $crate::codegen) const fn llvm_type(&self) -> inkwell::types::StructType<'ctx> {
                    self.llvm_type
                }
            }
        }
    };
}
