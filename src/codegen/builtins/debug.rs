use std::fmt::Write;

use crate::codegen::{TypeTag, types::ClassId};

#[repr(C)]
// TODO this should really be in crate::codegen::types, ideally somehow automagically synced with
// the inkwell definition
pub(super) struct Value {
    tag: TypeTag,
    unused_0: u8,
    class_id: ClassId,
    unused_1: u32,
    raw: u64,
}

#[repr(C)]
pub(super) struct FunctionSignature {
    class_id: u16,
    unused: u16,
    return_type_id: u32, // TODO Should be newtyped into TypeId or something
    arguments: *const FunctionArgument,
}

// TODO this should really be in crate::codegen::types, ideally somehow automagically synced with
// the inkwell definition
#[repr(C)]
pub(super) struct FunctionArgument {
    name: u32, // TODO this is an interned id of the name, we need actual support for interning tho
    type_id: u32, // TODO we should've a newtype for that
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            TypeTag::Primitive => write!(
                f,
                "Value({:?}, {:?})",
                self.tag,
                TypeTag::from_value(u8::try_from(self.raw).unwrap())
            ),
            TypeTag::U64 => todo!(),
            TypeTag::FunctionSignature => {
                // TODO resolve the return type to the actual type
                // TODO resolve the interned names
                // TODO resolve the names of predefined types
                let mut formatted_arguments = String::new();

                let signature = self.raw as *const FunctionSignature;
                let mut arguments = unsafe { &*signature }.arguments;
                loop {
                    let argument = unsafe { &*arguments };

                    if argument.name == 0 && argument.type_id == 0 {
                        break;
                    }

                    write!(
                        formatted_arguments,
                        "({}, {}), ",
                        argument.name, argument.type_id
                    )?;

                    unsafe {
                        arguments = arguments.add(1);
                    }
                }
                let return_type = unsafe { &*signature }.return_type_id;

                write!(f, "fn({formatted_arguments}): {return_type}")
            }
        }
    }
}

// TODO actually print useful information instead of raw data here
pub(super) extern "C" fn debug_type_definition_impl(value: *const Value) {
    println!("{:?}", unsafe { &*value });
}
