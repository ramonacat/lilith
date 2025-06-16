use inkwell::values::PointerValue;

use super::classes::ClassId;
use crate::{
    bytecode::{Identifier, TypeId},
    codegen::{
        context_ergonomics::ContextErgonomics,
        llvm_struct::{
            basic_value_enum::IntoValue,
            representations::{LlvmRepresentation, OperandValue},
        },
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
