pub(in crate::codegen) mod functions;
pub(in crate::codegen) mod value;

use functions::{FunctionArgumentProvider, FunctionSignatureProvider};
use inkwell::{
    builder::Builder,
    context::Context,
    values::{GlobalValue, IntValue, PointerValue},
};
use value::ValueProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// The value of 0 means no class
pub(super) struct ClassId(u16);

impl ClassId {
    const fn none() -> Self {
        Self(0)
    }
}

#[repr(u8)]
#[derive(Debug)]
pub(super) enum TypeTag {
    Primitive = 0,

    U64 = 16,

    FunctionSignature = 128,
}

impl TypeTag {
    pub const fn from_value(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Primitive),
            16 => Some(Self::U64),
            128 => Some(Self::FunctionSignature),
            _ => None,
        }
    }
}

pub(super) fn register(context: &Context) -> Types {
    Types {
        context,
        value_type: ValueProvider::register(context),
        function_argument_type: FunctionArgumentProvider::register(context),
        function_signature_type: FunctionSignatureProvider::register(context),
    }
}

pub(super) struct Types<'ctx> {
    context: &'ctx Context,
    value_type: ValueProvider<'ctx>,
    function_argument_type: FunctionArgumentProvider<'ctx>,
    function_signature_type: FunctionSignatureProvider<'ctx>,
}

pub(super) struct ArgumentType<'ctx> {
    name: IntValue<'ctx>,
    type_id: IntValue<'ctx>,
}

impl<'ctx> ArgumentType<'ctx> {
    pub(crate) const fn new(name: IntValue<'ctx>, type_id: IntValue<'ctx>) -> Self {
        Self { name, type_id }
    }
}

// TODO instead of return straight up StructValues, should we do some newtyping to avoid type
// confusion?
impl<'ctx> Types<'ctx> {
    pub(super) const fn value(&self) -> &ValueProvider<'ctx> {
        &self.value_type
    }

    pub(super) fn register_predefined(&self, builder: &Builder<'ctx>, types: GlobalValue<'ctx>) {
        let gep = unsafe {
            builder.build_gep(
                self.value_type.llvm_type(),
                types.as_pointer_value(),
                &[self
                    .context
                    .i32_type()
                    .const_int(TypeTag::U64 as u64, false)],
                "gep_u64",
            )
        }
        .unwrap();

        self.make_value(
            self.make_tag(TypeTag::Primitive),
            self.make_class_id(ClassId::none()),
            self.context
                .i64_type()
                .const_int(TypeTag::U64 as u64, false),
            builder,
            gep,
        );
    }

    pub(super) fn make_function_argument(
        &self,
        target: PointerValue<'ctx>,
        builder: &Builder<'ctx>,
        argument: &ArgumentType<'ctx>,
    ) {
        self.function_argument_type.fill_in(
            target,
            &[argument.name.into(), argument.type_id.into()],
            builder,
        );
    }

    pub(super) fn make_function_arguments(
        &self,
        builder: &Builder<'ctx>,
        arguments: &[ArgumentType<'ctx>],
    ) -> PointerValue<'ctx> {
        let arguments_allocation = builder
            .build_array_malloc(
                self.function_argument_type.llvm_type(),
                self.context
                    .i32_type()
                    .const_int(arguments.len() as u64 + 1, false),
                "arguments",
            )
            .unwrap();

        for (index, argument) in arguments
            .iter()
            .chain(&[ArgumentType::new(
                self.context.i32_type().const_int(0, false),
                self.context.i32_type().const_int(0, false),
            )])
            .enumerate()
        {
            let argument_pointer = unsafe {
                builder.build_gep(
                    self.function_argument_type.llvm_type(),
                    arguments_allocation,
                    &[self.context.i32_type().const_int(index as u64, false)],
                    "arguments",
                )
            }
            .unwrap();

            self.make_function_argument(argument_pointer, builder, argument);
        }

        arguments_allocation
    }

    pub(super) fn make_function_signature(
        &self,
        builder: &Builder<'ctx>,
        class_id: IntValue<'ctx>,
        return_type: IntValue<'ctx>,
        arguments: PointerValue<'ctx>,
        target: PointerValue<'ctx>,
    ) {
        let signature_ptr = builder
            .build_malloc(self.function_signature_type.llvm_type(), "signature_ptr")
            .unwrap();

        self.function_signature_type.fill_in(
            signature_ptr,
            &[
                class_id.into(),
                self.context.i16_type().const_zero().into(),
                return_type.into(),
                arguments.into(),
            ],
            builder,
        );

        let signature_u64 = builder
            .build_ptr_to_int(signature_ptr, self.context.i64_type(), "signature_u64")
            .unwrap();

        self.make_value(
            self.make_tag(TypeTag::FunctionSignature),
            self.make_class_id(ClassId::none()),
            signature_u64,
            builder,
            target,
        );
    }

    fn make_tag(&self, tag: TypeTag) -> IntValue<'ctx> {
        self.context.i8_type().const_int(tag as u64, false)
    }

    fn make_class_id(&self, id: ClassId) -> IntValue<'ctx> {
        self.context.i16_type().const_int(u64::from(id.0), false)
    }

    fn make_value(
        &self,
        type_tag: IntValue<'ctx>,
        class_id: IntValue<'ctx>,
        value: IntValue<'ctx>,
        builder: &Builder<'ctx>,
        target: PointerValue<'ctx>,
    ) {
        self.value_type.fill_in(
            target,
            &[
                type_tag.into(),
                self.context.i8_type().const_zero().into(),
                class_id.into(),
                self.context.i32_type().const_zero().into(),
                value.into(),
            ],
            builder,
        );
    }
}
