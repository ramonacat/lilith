pub(in crate::codegen) mod functions;
pub(in crate::codegen) mod primitive;
#[allow(clippy::module_inception)]
pub(in crate::codegen) mod types;
pub(in crate::codegen) mod value;

use functions::FunctionTypes;
use inkwell::{builder::Builder, context::Context, types::StructType, values::GlobalValue};
use primitive::PrimitiveTypes;
use types::TypesTypes;
use value::{TypeTag, ValueOpaquePointer, ValueProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// The value of 0 means no class
pub(super) struct ClassId(u16);

impl ClassId {
    const fn none() -> Self {
        Self(0)
    }
}

pub(super) fn register(context: &Context) -> Types {
    let value_type = ValueProvider::register(context);
    let value_types = ValueTypes {
        value_type,
        context,
    };

    Types {
        context,
        value: value_types,
        function: FunctionTypes::new(context, value_types),
        primitive: PrimitiveTypes::new(value_types),
        types: TypesTypes::new(context),
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ValueTypes<'ctx> {
    value_type: ValueProvider<'ctx>,
    context: &'ctx Context,
}
impl<'ctx> ValueTypes<'ctx> {
    // TODO we should just get rid of this, and handle the cases where it's needed internally
    pub(crate) const fn llvm_type(&self) -> StructType<'ctx> {
        self.value_type.llvm_type()
    }

    pub(crate) const fn opaque_pointer(
        &self,
        pointer: inkwell::values::PointerValue<'ctx>,
    ) -> ValueOpaquePointer<'ctx> {
        self.value_type.opaque_pointer(pointer)
    }
}

pub(super) struct Types<'ctx> {
    context: &'ctx Context,

    value: ValueTypes<'ctx>,
    function: FunctionTypes<'ctx>,
    primitive: PrimitiveTypes<'ctx>,
    #[allow(clippy::struct_field_names)]
    types: TypesTypes<'ctx>,
}

impl<'ctx> Types<'ctx> {
    pub(super) const fn value(&self) -> &ValueTypes<'ctx> {
        &self.value
    }

    pub(super) fn register_predefined(&self, builder: &Builder<'ctx>, types: GlobalValue<'ctx>) {
        let gep = unsafe {
            builder.build_gep(
                self.value.value_type.llvm_type(),
                types.as_pointer_value(),
                &[self
                    .context
                    .i32_type()
                    .const_int(TypeTag::U64 as u64, false)],
                "gep_u64",
            )
        }
        .unwrap();

        self.value.make_value(
            self.value.make_tag(TypeTag::Primitive),
            self.value.make_class_id(ClassId::none()),
            self.context
                .i64_type()
                .const_int(TypeTag::U64 as u64, false),
            builder,
            gep,
        );
    }

    pub(crate) const fn function(&self) -> &FunctionTypes<'ctx> {
        &self.function
    }

    pub(in crate::codegen) const fn primitive(&self) -> &PrimitiveTypes<'ctx> {
        &self.primitive
    }

    pub(crate) const fn types(&self) -> &TypesTypes<'ctx> {
        &self.types
    }
}
