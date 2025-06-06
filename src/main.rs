#![deny(clippy::all, clippy::pedantic, clippy::nursery, warnings)]
mod types;

use inkwell::context::Context;
use types::ByteCode;

fn main() {
    let context = Context::create();
    let bytecode = ByteCode::new(&context);
    let result = bytecode.execute();

    println!("result: {result}");
}
