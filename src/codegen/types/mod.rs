pub(in crate::codegen) mod functions;

use functions::FunctionArgumentProvider;
use inkwell::{
    builder::Builder,
    context::Context,
    types::StructType,
    values::{GlobalValue, IntValue, PointerValue},
};

use super::context_ergonomics::ContextErgonomics as _;

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
    let value_type = context.named_struct(
        "Value",
        &[
            // tag
            context.i8(),
            // unused
            context.i8(),
            // classid
            context.i16(),
            // unused
            context.i32(),
            // value
            context.i64(),
        ],
    );

    let function_signature_type = context.named_struct(
        "FunctionSignature",
        &[
            // classid
            context.i16(),
            // unused (probably flags, like is_static, etc.)
            context.i16(),
            // return_type (typeid)
            context.i32(),
            // arguments (heap allocated array)
            context.ptr(),
        ],
    );

    Types {
        context,
        value_type,
        function_argument_type: FunctionArgumentProvider::register(context),
        function_signature_type,
    }
}

pub(super) struct Types<'ctx> {
    context: &'ctx Context,
    value_type: StructType<'ctx>,
    function_argument_type: FunctionArgumentProvider<'ctx>,
    function_signature_type: StructType<'ctx>,
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
    pub(super) const fn value(&self) -> StructType<'ctx> {
        self.value_type
    }

    pub(super) fn register_predefined(&self, builder: &Builder<'ctx>, types: GlobalValue<'ctx>) {
        let gep = unsafe {
            builder.build_gep(
                self.value_type,
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
        assert!(class_id.get_type().get_bit_width() == 16);
        assert!(return_type.get_type().get_bit_width() == 32);

        // TODO by god we need some cleaner API to build structs...
        let signature_ptr = builder
            .build_malloc(self.function_signature_type, "signature_ptr")
            .unwrap();

        let class_id_gep = builder
            .build_struct_gep(self.function_signature_type, signature_ptr, 0, "class_id")
            .unwrap();
        builder.build_store(class_id_gep, class_id).unwrap();

        let unused_gep = builder
            .build_struct_gep(self.function_signature_type, signature_ptr, 1, "unused")
            .unwrap();

        builder
            .build_store(unused_gep, self.context.i16_type().const_int(0, false))
            .unwrap();

        let return_type_gep = builder
            .build_struct_gep(
                self.function_signature_type,
                signature_ptr,
                2,
                "return_type",
            )
            .unwrap();
        builder.build_store(return_type_gep, return_type).unwrap();

        let arguments_gep = builder
            .build_struct_gep(self.function_signature_type, signature_ptr, 3, "arguments")
            .unwrap();
        builder.build_store(arguments_gep, arguments).unwrap();

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
        assert!(type_tag.get_type().get_bit_width() == 8);
        assert!(class_id.get_type().get_bit_width() == 16);
        assert!(value.get_type().get_bit_width() == 64);

        let type_tag_gep = builder
            .build_struct_gep(self.value_type, target, 0, "type_tag")
            .unwrap();
        builder.build_store(type_tag_gep, type_tag).unwrap();

        let unused_0_gep = builder
            .build_struct_gep(self.value_type, target, 1, "unused_0")
            .unwrap();
        builder
            .build_store(unused_0_gep, self.context.i8_type().const_zero())
            .unwrap();

        let class_id_gep = builder
            .build_struct_gep(self.value_type, target, 2, "class_id")
            .unwrap();
        builder.build_store(class_id_gep, class_id).unwrap();

        let unused_1_gep = builder
            .build_struct_gep(self.value_type, target, 3, "unused_1")
            .unwrap();
        builder
            .build_store(unused_1_gep, self.context.i32_type().const_zero())
            .unwrap();

        let value_gep = builder
            .build_struct_gep(self.value_type, target, 4, "value")
            .unwrap();
        builder.build_store(value_gep, value).unwrap();
    }
}
