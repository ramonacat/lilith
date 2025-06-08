use crate::{
    codegen::{
        context_ergonomics::ContextErgonomics, llvm_struct::representations::LlvmRepresentation,
    },
    llvm_struct,
};

// TODO both name and type_id should be newtyped into something better representing what they are
llvm_struct! {
    struct FunctionArgument {
        name: u32,
        type_id: u32
    }
}
