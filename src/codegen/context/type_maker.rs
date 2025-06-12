use inkwell::{
    AddressSpace,
    context::Context,
    types::{BasicType as _, BasicTypeEnum, FunctionType},
};

#[derive(Clone, Copy)]
pub(in crate::codegen) enum TypeDeclaration {
    // TODO add U*
    // TODO add S* (signed types - these will require some type-level indirection, as those are the
    // same as unsigned, just with different instructions used on them)
    // TODO add F*

    // TODO should we separate DataPointer and FunctionPointer?
    Pointer,
}

#[derive(Debug)]
pub(in crate::codegen) struct TypeMaker<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> TypeMaker<'ctx> {
    pub const fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    // TODO we might add another layer of type indirection here, so that e.g. structs can be
    // represented
    pub fn make(&self, declaration: TypeDeclaration) -> BasicTypeEnum<'ctx> {
        match declaration {
            TypeDeclaration::Pointer => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    pub fn make_function(
        &self,
        return_type: Option<TypeDeclaration>,
        arguments: &[TypeDeclaration],
    ) -> FunctionType<'ctx> {
        let arguments = arguments
            .iter()
            .map(|x| self.make(*x).into())
            .collect::<Vec<_>>();

        return_type.map_or_else(
            || self.context.void_type().fn_type(&arguments, false),
            |return_type| self.make(return_type).fn_type(&arguments, false),
        )
    }

    // TODO add a make_struct function that will replace the named_struct from context_ergonomics
}
