use inkwell::{
    context::Context,
    module::{Linkage, Module},
    types::StructType,
    values::{IntValue, PointerValue},
};

use super::context::CodegenContext;
use crate::codegen::{
    context_ergonomics::ContextErgonomics,
    llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
};

llvm_struct! {
    struct GlobalConstructor {
        priority: u32,
        target: *const fn(),
        initialized_value: Option<*const ()>
    }
}

pub fn register<'ctx>(codegen_context: &CodegenContext<'ctx>) -> ModuleBuilderProvider<'ctx> {
    ModuleBuilderProvider {
        global_constructors_provider: GlobalConstructorProvider::register(
            codegen_context.llvm_context(),
        ),
        context: codegen_context.llvm_context(),
    }
}

pub(in crate::codegen) struct ModuleBuilderProvider<'ctx> {
    global_constructors_provider: GlobalConstructorProvider<'ctx>,
    context: &'ctx Context,
}

impl<'ctx> ModuleBuilderProvider<'ctx> {
    pub fn make_builder(&self, name: &str) -> ModuleBuilder<'ctx> {
        ModuleBuilder::new(
            name,
            self.context,
            self.global_constructors_provider.llvm_type(),
        )
    }
}

pub(in crate::codegen) struct ModuleBuilder<'ctx> {
    // TODO instead of making this pub, shall we provide some cute API for adding globals and
    // functions
    pub(in crate::codegen) module: Module<'ctx>,
    global_constructors: Vec<GlobalConstructorType<'ctx>>,
    global_constructor_type: StructType<'ctx>,
}

// TODO this is an abomination, but we need to have automagic generation of structs for this in
// llvm_struct
type GlobalConstructorType<'ctx> = (IntValue<'ctx>, PointerValue<'ctx>, PointerValue<'ctx>);

impl<'ctx> ModuleBuilder<'ctx> {
    fn new(name: &str, context: &'ctx Context, global_constructor_type: StructType<'ctx>) -> Self {
        Self {
            module: context.create_module(name),
            global_constructors: vec![],
            global_constructor_type,
        }
    }

    pub fn add_global_constructor(&mut self, constructor: GlobalConstructorType<'ctx>) {
        self.global_constructors.push(constructor);
    }

    pub fn build(self) -> Module<'ctx> {
        let Self {
            module,
            global_constructors,
            global_constructor_type,
        } = self;

        let global_constructors_array_type =
            global_constructor_type.array_type(u32::try_from(global_constructors.len()).unwrap());

        let global_constructors_value =
            module.add_global(global_constructors_array_type, None, "llvm.global_ctors");

        global_constructors_value.set_linkage(Linkage::Appending);
        let constructors: Vec<_> = global_constructors
            .iter()
            .map(|x| {
                global_constructor_type.const_named_struct(&[x.0.into(), x.1.into(), x.2.into()])
            })
            .collect();
        global_constructors_value
            .set_initializer(&global_constructor_type.const_array(&constructors));

        module
    }
}
