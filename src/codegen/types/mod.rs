pub(in crate::codegen) mod functions;
pub(in crate::codegen) mod primitive;
#[allow(clippy::module_inception)]
pub(in crate::codegen) mod types;
pub(in crate::codegen) mod value;

use functions::FunctionTypes;
use inkwell::{builder::Builder, context::Context, types::StructType, values::GlobalValue};
use primitive::PrimitiveTypes;
use types::TypesTypes;
use value::ValueProvider;

use super::ContextErgonomics as _;
use crate::bytecode::TypeTag;

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
    // TODO this is fucky hacky for the old bad typestore
    pub(crate) const fn llvm_type(&self) -> StructType<'ctx> {
        self.value_type.llvm_type()
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
                &[self.context.const_u32(TypeTag::U64 as u32)],
                "gep_u64",
            )
        }
        .unwrap();

        let value = self.value.make_value(
            self.value.make_tag(TypeTag::Primitive),
            self.value.make_class_id(ClassId::none()),
            self.context.const_u64(TypeTag::U64 as u64),
            builder,
        );

        // TODO the alignments are pulled from my ass and idk if they're right, but it doesn't
        // matter, bevause this will be replaced with codegen::type_store, and this code is fucky
        // hacky
        builder
            .build_memmove(
                gep,
                8,
                value.ptr(),
                8,
                self.value.value_type.llvm_type().size_of().unwrap(),
            )
            .unwrap();
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
