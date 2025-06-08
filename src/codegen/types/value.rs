use super::{ClassId, TypeTag};
use crate::{
    codegen::{
        context_ergonomics::ContextErgonomics, llvm_struct::representations::LlvmRepresentation,
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
