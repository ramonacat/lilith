pub(super) mod representations;

// TODO the amount of copy-paste here is unholy, fix it
#[macro_export]
macro_rules! get_field {
    ($field_name_first:ident: $field_type_first:ty, $($field_name:ident: $field_type:ty),*) => {
        // TODO can we do some dark magic to return a more precise type here?
        paste::paste! {
            #[allow(unused)]
            pub fn [<get_ $field_name_first>](&self, builder: &inkwell::builder::Builder<'ctx>) -> inkwell::values::BasicValueEnum<'ctx> {
                let struct_gep = builder.build_struct_gep(
                    self.llvm_type,
                    self.pointer,
                    0,
                    "field_gep"
                ).unwrap();

                builder.build_load(<$field_type_first>::llvm_type(self.context), struct_gep, "field").unwrap().into()
            }
        }

        get_field!(1, $($field_name: $field_type),*);
    };
    ($count:expr, $field_name_first:ident: $field_type_first:ty, $($field_name:ident: $field_type:ty),*) => {
        // TODO can we do some dark magic to return a more precise type here?
        paste::paste! {
            #[allow(unused)]
            pub fn [<get_ $field_name_first>](&self, builder: &inkwell::builder::Builder<'ctx>) -> inkwell::values::BasicValueEnum<'ctx> {
                let struct_gep = builder.build_struct_gep(
                    self.llvm_type,
                    self.pointer,
                    $count as u32,
                    "field_gep"
                ).unwrap();

                builder.build_load(<$field_type_first>::llvm_type(self.context), struct_gep, "field").unwrap().into()
            }
        }

        get_field!(1+$count, $($field_name: $field_type),*);
    };
    ($count:expr, $field_name:ident: $field_type:ty) => {
        // TODO can we do some dark magic to return a more precise type here?
        paste::paste! {
            #[allow(unused)]
            pub fn [<get_ $field_name>](&self, builder: &inkwell::builder::Builder<'ctx>) -> inkwell::values::BasicValueEnum<'ctx> {
                let struct_gep = builder.build_struct_gep(
                    self.llvm_type,
                    self.pointer,
                    $count as u32,
                    "field_gep"
                ).unwrap();

                builder.build_load(<$field_type>::llvm_type(self.context), struct_gep, "field").unwrap().into()
            }
        }
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
            fn llvm_type(context: &'ctx inkwell::context::Context) -> inkwell::types::BasicTypeEnum<'ctx> {
                context.get_struct_type(stringify!($name)).unwrap().into()
            }
        }

        paste::paste! {
            #[derive(Debug, Clone, Copy)]
            pub(in $crate::codegen) struct [<$name Opaque>]<'ctx> {
                pointer: inkwell::values::PointerValue<'ctx>,
                context: &'ctx inkwell::context::Context,
                llvm_type: inkwell::types::StructType<'ctx>,
            }

            impl<'ctx> [<$name Opaque>]<'ctx> {
                #[allow(unused)]
                const fn new(
                    pointer: PointerValue<'ctx>,
                    context: &'ctx inkwell::context::Context,
                    llvm_type: inkwell::types::StructType<'ctx>,
                ) -> Self {
                        Self { pointer, context, llvm_type }
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
            #[allow(unused)]
            impl<'ctx> [<$name Provider>]<'ctx> {
                pub(in $crate::codegen) fn register(context: &'ctx inkwell::context::Context) -> Self {
                    let llvm_type = context.named_struct(stringify!($name), &[
                        $(<$field_type>::llvm_type(context)),+
                    ]);
                    Self { llvm_type, context }
                }

                #[allow(unused)]
                pub(in $crate::codegen) const fn opaque(&self, pointer: PointerValue<'ctx>) -> [<$name Opaque>]<'ctx> {
                    [<$name Opaque>]::new(pointer, self.context, self.llvm_type)
                }

                pub(in $crate::codegen) fn fill_in(
                    &self,
                    target: inkwell::values::PointerValue<'ctx>,
                    fields: &[inkwell::values::BasicValueEnum<'ctx>],
                    builder: &inkwell::builder::Builder<'ctx>,
                ) {
                    assert!(fields.len() == self.llvm_type.count_fields() as usize);

                    for (index, value) in fields.iter().enumerate() {
                        let field_gep = builder
                            .build_struct_gep(
                                self.llvm_type,
                                target,
                                u32::try_from(index).unwrap(),
                                "field_gep"
                            )
                            .unwrap();

                        builder.build_store(field_gep, *value).unwrap();
                    }
                }

                pub(in $crate::codegen) const fn llvm_type(&self) -> inkwell::types::StructType<'ctx> {
                    self.llvm_type
                }
            }
        }
    };
}
