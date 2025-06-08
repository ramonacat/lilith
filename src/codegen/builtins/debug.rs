use std::fmt::Write;

use crate::codegen::{
    TypeTag,
    types::{functions::FunctionSignature, value::Value},
};

impl std::fmt::Debug for crate::codegen::types::value::Value {
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
