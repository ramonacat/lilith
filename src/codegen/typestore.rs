use inkwell::{
    builder::Builder, context::Context, module::Module, types::StructType, values::GlobalValue,
};

use super::types::Types;

const PREDEFINED_TYPES_COUNT: u32 = 256;

pub(super) fn register<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    types: &Types<'ctx>,
) -> TypeStore<'ctx> {
    let types_global_type = types
        .value()
        // TODO this really should be dynamic, but for prototyping 256 predefined + 256 complex
        // types should be fine
        .array_type(PREDEFINED_TYPES_COUNT + 256);
    let types_global = module.add_global(
        // The first 256 are predefined types (primitives, etc.)
        types_global_type,
        None,
        "types",
    );
    types_global.set_initializer(&types_global_type.const_zero());

    types.register_predefined(builder, types_global);

    TypeStore {
        store: types_global,
        context,
        value_type: types.value(),
    }
}
pub(super) struct TypeStore<'ctx> {
    context: &'ctx Context,
    store: GlobalValue<'ctx>,
    value_type: StructType<'ctx>,
}

impl<'ctx> TypeStore<'ctx> {
    // TODO this method should not exist, we should just have "next_slot" for adding types and "slot_by_id" for retreiving them
    pub(crate) fn get_slot(
        &self,
        arg: u64,
        builder: &Builder<'ctx>,
    ) -> inkwell::values::PointerValue<'_> {
        self.get_type_at_index(arg, builder)
    }

    fn get_type_at_index(
        &self,
        index: u64,
        builder: &Builder<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        unsafe {
            builder.build_gep(
                self.value_type,
                self.store.as_pointer_value(),
                &[self.context.i32_type().const_int(index, false)],
                "type",
            )
        }
        .unwrap()
    }
}
