use inkwell::values::PointerValue;

use super::{TypeStoreProvider, TypeValueProvider};
use crate::codegen::{
    ContextErgonomics as _,
    module::{self, GlobalConstructorFunction},
};

pub(super) fn make_type_store_initializer<'ctx>(
    module_builder: &module::ModuleBuilder<'ctx>,
    type_store: PointerValue<'ctx>,
) -> GlobalConstructorFunction<'ctx> {
    module_builder.build_procedure::<_, GlobalConstructorFunction>(|function, context, _module| {
        let entry = context.append_basic_block(function, "entry");
        let builder = context.create_builder();
        builder.position_at_end(entry);

        let capacity = context.const_u32(1);
        let types = builder
            .build_array_malloc(
                TypeValueProvider::register(context).llvm_type(),
                capacity,
                "types",
            )
            .unwrap();

        TypeStoreProvider::register(context).fill_in(
            type_store,
            &builder,
            super::TypeStoreOpaque {
                types,
                length: context.const_u32(0),
                capacity,
            },
        );
        builder.build_return(None).unwrap();
    })
}
