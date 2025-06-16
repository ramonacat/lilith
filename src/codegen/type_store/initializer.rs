use inkwell::values::PointerValue;

use super::{TypeStoreProvider, TypeValueProvider};
use crate::codegen::{
    ContextErgonomics as _,
    llvm_struct::representations::ConstOrValue,
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

        // TODO we should be using make_value from TypeValueProvider here
        let types = builder
            .build_array_malloc(
                TypeValueProvider::register(context).llvm_type(),
                context.const_u32(1),
                "types",
            )
            .unwrap();

        TypeStoreProvider::register(context).fill_in(
            type_store,
            &builder,
            super::TypeStoreOpaque {
                types: ConstOrValue::Value(types),
                length: ConstOrValue::Const(0),
                capacity: ConstOrValue::Const(1),
            },
        );
        builder.build_return(None).unwrap();
    })
}
