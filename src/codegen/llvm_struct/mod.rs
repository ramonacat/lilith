pub(super) mod representations;

#[macro_export]
macro_rules! llvm_struct {
    (struct $name:ident { $($field_name:ident: $field_type:ty),+ }) => {
        #[repr(C)]
        pub(in $crate::codegen) struct $name {
            $(pub(in $crate::codegen) $field_name: $field_type),+
        }

        paste::paste! {
            pub(in $crate::codegen) struct [<$name Provider>]<'ctx> {
                llvm_type: inkwell::types::StructType<'ctx>,
            }

            impl<'ctx> [<$name Provider>]<'ctx> {
                pub(in $crate::codegen) fn register(context: &'ctx inkwell::context::Context) -> Self {
                    let llvm_type = context.named_struct(stringify!($name), &[
                        $(<$field_type>::llvm_type(context)),+
                    ]);
                    Self { llvm_type }
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
