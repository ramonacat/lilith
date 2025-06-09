#![deny(clippy::all, clippy::pedantic, clippy::nursery, warnings)]
mod bytecode;
#[macro_use]
mod codegen;
mod types;
use bytecode::ByteCode;
use codegen::CodeGen;
use inkwell::context::Context;

fn main() {
    let context = Context::create();
    let bytecode = ByteCode::new();
    let codegen = CodeGen::new(&context);
    let result = codegen.execute(bytecode);

    println!("result: {result}");
}
