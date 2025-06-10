use inkwell::{
    builder::Builder,
    context::Context,
    types::StructType,
    values::{IntValue, PointerValue},
};

use super::{ValueTypes, value::ValueOpaquePointer};
use crate::{
    bytecode::{Identifier, TypeId, TypeTag},
    codegen::{
        context_ergonomics::ContextErgonomics,
        llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
        types::ClassId,
    },
    llvm_struct,
};

llvm_struct! {
    struct FunctionArgument {
        name: Identifier,
        type_id: TypeId
    }
}

llvm_struct! {
    struct FunctionSignature {
        class_id: ClassId,
        argument_count: u16,
        return_type_id: TypeId,
        arguments: *const FunctionArgument
    }
}

pub(in crate::codegen) struct FunctionTypes<'ctx> {
    basic: ValueTypes<'ctx>,
    function_argument_type: FunctionArgumentProvider<'ctx>,
    function_signature_type: FunctionSignatureProvider<'ctx>,
    context: &'ctx Context,
}

pub(in crate::codegen) struct ArgumentType<'ctx> {
    name: IntValue<'ctx>,
    type_id: IntValue<'ctx>,
}

impl<'ctx> ArgumentType<'ctx> {
    pub(crate) const fn new(name: IntValue<'ctx>, type_id: IntValue<'ctx>) -> Self {
        Self { name, type_id }
    }
}

impl<'ctx> FunctionTypes<'ctx> {
    pub fn new(context: &'ctx Context, basic: ValueTypes<'ctx>) -> Self {
        Self {
            function_argument_type: FunctionArgumentProvider::register(context),
            function_signature_type: FunctionSignatureProvider::register(context),
            context,
            basic,
        }
    }

    pub(super) fn make_function_argument(
        &self,
        target: PointerValue<'ctx>,
        builder: &Builder<'ctx>,
        argument: &ArgumentType<'ctx>,
    ) {
        self.function_argument_type
            .fill_in(target, builder, argument.name, argument.type_id);
    }

    pub(in crate::codegen) fn make_function_arguments(
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

    pub(in crate::codegen) fn make_function_signature(
        &self,
        builder: &Builder<'ctx>,
        class_id: IntValue<'ctx>,
        return_type: IntValue<'ctx>,
        arguments: PointerValue<'ctx>,
    ) -> ValueOpaquePointer<'ctx> {
        let signature_ptr = builder
            .build_malloc(self.function_signature_type.llvm_type(), "signature_ptr")
            .unwrap();

        self.function_signature_type.fill_in(
            signature_ptr,
            builder,
            class_id,
            self.context.i16_type().const_zero(),
            return_type,
            arguments,
        );

        let signature_u64 = builder
            .build_ptr_to_int(signature_ptr, self.context.i64_type(), "signature_u64")
            .unwrap();

        self.basic.make_value(
            self.basic.make_tag(TypeTag::FunctionSignature),
            self.basic.make_class_id(ClassId::none()),
            signature_u64,
            builder,
        )
    }

    // TODO this method is fucky hacky, get rid of it once the typestore situation is cleaned up
    pub(crate) const fn signature_llvm_type(&self) -> StructType<'ctx> {
        self.function_signature_type.llvm_type()
    }
}
