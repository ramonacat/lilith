#[macro_use]
pub(in crate::codegen) mod type_maker;

use inkwell::context::Context;

pub(in crate::codegen) trait AsLlvmContext<'ctx>: Copy {
    fn llvm_context(self) -> &'ctx Context;
}

impl<'ctx> AsLlvmContext<'ctx> for &CodegenContext<'ctx> {
    fn llvm_context(self) -> &'ctx Context {
        self.llvm_context
    }
}

impl<'ctx> AsLlvmContext<'ctx> for &'ctx Context {
    fn llvm_context(self) -> &'ctx Context {
        self
    }
}

// TODO get rid of it, and use the raw inkwell Context
pub struct CodegenContext<'ctx> {
    llvm_context: &'ctx Context,
}

impl<'ctx> CodegenContext<'ctx> {
    pub(crate) const fn llvm_context(&self) -> &'ctx Context {
        self.llvm_context
    }

    pub(crate) const fn new(context: &'ctx Context) -> Self {
        Self {
            llvm_context: context,
        }
    }
}
