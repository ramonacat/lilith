use inkwell::values::PointerValue;

use crate::{
    bytecode::TypeTag,
    codegen::{
        context_ergonomics::ContextErgonomics,
        llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
        types::classes::ClassId,
    },
    llvm_struct,
};

llvm_struct! {
    struct Value {
        tag: TypeTag,
        unused_0: u8,
        class_id: ClassId,
        unused_1: u32,
        raw: u64
    }
}
