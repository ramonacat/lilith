pub(in crate::codegen) mod basic_value_enum;
pub(in crate::codegen) mod opaque_struct;
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

        paste::paste!{
            impl $name {
                pub(in $crate::codegen) const fn into_opaque<'ctx>(self) -> [<$name Opaque>]<'ctx> {
                    [<$name Opaque>] {
                        $($field_name: $crate::codegen::llvm_struct::representations::ConstOrValue::Const(self.$field_name)),*
                    }
                }
            }
        }

        impl<'ctx> LlvmRepresentation<'ctx> for $name {
            type LlvmType = inkwell::types::StructType<'ctx>;
            type LlvmValue = inkwell::values::StructValue<'ctx>;

            fn assert_valid(_context: &'ctx inkwell::context::Context, _value: &$crate::codegen::llvm_struct::representations::ConstOrValue<'ctx, Self>) {}

            fn llvm_type(context: &'ctx inkwell::context::Context) -> Self::LlvmType {
                paste::paste! { [<$name Provider>]::new(context).llvm_type() }
            }
        }

        impl<'ctx>
            $crate::codegen::llvm_struct::representations::OperandValue<'ctx>
                for $name {
            // TODO could this somehow work with or replace make_value?
            #[allow(unused)]
            fn build_move_into(
                self,
                context: &'ctx inkwell::context::Context,
                builder: &inkwell::builder::Builder<'ctx>,
                target: PointerValue<'ctx>,
            ) {
                let llvm_type = $name::llvm_type(context);

                paste::paste!{
                    [<$name Provider>]::new(context)
                        .fill_in(target, builder, self.into_opaque());
                }
            }

            fn build_store_into(
                &self,
                _context: &'ctx inkwell::context::Context,
                _builder: &inkwell::builder::Builder<'ctx>,
                _target: PointerValue<'ctx>,
            ) {
                todo!();
            }
        }

        impl<'ctx>
            $crate::codegen::llvm_struct::representations::OperandValue<'ctx>
                // TODO this should be also implemented for $name and [<$name Opaque>]
                for $crate::codegen::llvm_struct::representations::ConstOrValue<'ctx, $name> {
            // TODO could this somehow work with or replace make_value?
            #[allow(unused)]
            fn build_move_into(
                self,
                context: &'ctx inkwell::context::Context,
                builder: &inkwell::builder::Builder<'ctx>,
                target: PointerValue<'ctx>,
            ) {
                let llvm_type = $name::llvm_type(context);

                let mut index = 0;
                $(
                    let struct_value:$crate::codegen::llvm_struct::representations::ConstOrValue<
                        'ctx,
                        $field_type
                    > =
                        match self {
                            $crate::codegen::llvm_struct::representations::ConstOrValue::Const(_value) => {
                                todo!();
                            },
                            $crate::codegen::llvm_struct::representations::ConstOrValue::Value(value) => {
                                // TODO for some reason just using value.get_field(...) returns a
                                // struct. This dance with the stack seems to work fine though.
                                let stack_location = builder.build_alloca(value.get_type(), "").unwrap();
                                builder.build_store(stack_location, value).unwrap();

                                let gep = unsafe { builder.build_gep(
                                    value.get_type(),
                                    stack_location,
                                    &[
                                        context.const_u32(0),
                                        context.const_u32(index)
                                    ],
                                    ""
                                ) }.unwrap();

                                $crate::codegen::llvm_struct::representations::ConstOrValue::Value(
                                    builder.build_load(
                                        value.get_type().get_field_type_at_index(index).unwrap(),
                                        gep,
                                        ""
                                    ).unwrap().into_value()
                                )
                            }
                        };

                    let field_gep = builder.build_struct_gep(
                        llvm_type,
                        target,
                        index,
                        stringify!([<$field_name _gep>])
                    ).unwrap();

                    struct_value.build_store_into(
                        context,
                        builder,
                        field_gep,
                    );

                    #[allow(unused_assignments)]
                    {
                        index += 1;
                    }
                )*
            }

            #[allow(unused)]
            fn build_store_into(
                &self,
                context: &'ctx inkwell::context::Context,
                builder: &inkwell::builder::Builder<'ctx>,
                target: PointerValue<'ctx>,
            ) {
                let llvm_type = $name::llvm_type(context);

                let mut index = 0;
                $(
                    let struct_value:$crate::codegen::llvm_struct::representations::ConstOrValue<
                        'ctx,
                        $field_type
                    > =
                        match self {
                            $crate::codegen::llvm_struct::representations::ConstOrValue::Const(_value) => {
                                todo!();
                            },
                            $crate::codegen::llvm_struct::representations::ConstOrValue::Value(value) => {
                                // TODO for some reason just using value.get_field(...) returns a
                                // struct. This dance with the stack seems to work fine though.
                                let stack_location = builder.build_alloca(value.get_type(), "").unwrap();
                                builder.build_store(stack_location, *value).unwrap();

                                let gep = unsafe { builder.build_gep(
                                    value.get_type(),
                                    stack_location,
                                    &[
                                        context.const_u32(0),
                                        context.const_u32(index)
                                    ],
                                    ""
                                ) }.unwrap();

                                $crate::codegen::llvm_struct::representations::ConstOrValue::Value(
                                    builder.build_load(
                                        value.get_type().get_field_type_at_index(index).unwrap(),
                                        gep,
                                        ""
                                    ).unwrap().into_value()
                                )
                            }
                        };

                    let field_gep = builder.build_struct_gep(
                        llvm_type,
                        target,
                        index,
                        stringify!([<$field_name _gep>])
                    ).unwrap();

                    struct_value.build_store_into(
                        context,
                        builder,
                        field_gep,
                    );

                    #[allow(unused_assignments)]
                    {
                        index += 1;
                    }
                )*
            }

        }

        paste::paste! {
            #[allow(unused)]
            pub(in $crate::codegen) struct [<$name Opaque>]<'ctx> {
                $(
                    pub(in $crate::codegen) $field_name:
                        $crate::codegen::llvm_struct::representations::ConstOrValue<
                            'ctx,
                            $field_type
                        >
                    ),+
            }

            #[derive(Debug, Clone, Copy)]
            pub(in $crate::codegen) struct [<$name OpaquePointer>]<'ctx> {
                pointer: inkwell::values::PointerValue<'ctx>,
                context: &'ctx inkwell::context::Context,
                llvm_type: inkwell::types::StructType<'ctx>,
            }

            impl<'ctx> [<$name OpaquePointer>]<'ctx> {
                #[allow(unused)]
                fn new(
                    pointer: PointerValue<'ctx>,
                    context: &'ctx inkwell::context::Context,
                ) -> Self {
                    Self { pointer, context, llvm_type: [<$name Provider>]::new(context).llvm_type() }
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
            pub(in $crate::codegen) struct [<$name Provider>]<'ctx> {
                llvm_type: inkwell::types::StructType<'ctx>,
                context: &'ctx inkwell::context::Context
            }

            #[allow(unused)]
            impl<'ctx> [<$name Provider>]<'ctx> {
                pub(in $crate::codegen) fn new(context: &'ctx inkwell::context::Context) -> Self {
                    let llvm_type = context.get_struct_type(stringify!($name))
                        .unwrap_or_else(
                        || context.named_struct(stringify!($name), &[
                            $(<$field_type>::llvm_type(&context).into()),+
                        ])
                    );
                    Self { llvm_type, context }
                }

                #[allow(unused)]
                pub(in $crate::codegen) fn opaque_pointer(
                    &self,
                    pointer: PointerValue<'ctx>
                ) -> [<$name OpaquePointer>]<'ctx> {
                    [<$name OpaquePointer>]::new(pointer, self.context)
                }

                pub(in $crate::codegen) fn make_value(
                    &self,
                    builder: &inkwell::builder::Builder<'ctx>,
                    values: [<$name Opaque>]<'ctx>,
                ) -> [<$name OpaquePointer>]<'ctx> {
                    let target = builder.build_malloc(self.llvm_type(), stringify!($name)).unwrap();

                    self.fill_in(
                        target,
                        builder,
                        values,
                    );

                    self.opaque_pointer(target)
                }

                pub(in $crate::codegen) fn fill_in(
                    &self,
                    target: inkwell::values::PointerValue<'ctx>,
                    builder: &inkwell::builder::Builder<'ctx>,
                    values: [<$name Opaque>]<'ctx>
                ) {
                    let mut index:u32 = 0;

                    $({
                        <$field_type as LlvmRepresentation<'ctx>>::assert_valid(&self.context, &values.$field_name);

                        let field_gep = builder
                            .build_struct_gep(
                                self.llvm_type,
                                target,
                                index,
                                stringify!([<$field_name _gep>])
                            )
                            .unwrap();

                        values.$field_name.build_store_into(
                            &self.context,
                            builder,
                            field_gep,
                        );

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
