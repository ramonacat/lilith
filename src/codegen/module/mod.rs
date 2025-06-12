use inkwell::{
    AddressSpace,
    module::{Linkage, Module},
    types::{BasicType, FunctionType, StructType},
    values::{FunctionValue, GlobalValue, PointerValue},
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

pub fn register<'ctx, 'codegen>(
    codegen_context: &'codegen CodegenContext<'ctx>,
) -> ModuleBuilderProvider<'ctx, 'codegen> {
    ModuleBuilderProvider {
        global_constructors_provider: GlobalConstructorProvider::register(
            codegen_context.llvm_context(),
        ),
        codegen_context,
    }
}

pub(in crate::codegen) enum FunctionVisibility {
    Private,
    Public,
}

pub(in crate::codegen) struct ModuleBuilderProvider<'ctx, 'codegen> {
    global_constructors_provider: GlobalConstructorProvider<'ctx>,
    codegen_context: &'codegen CodegenContext<'ctx>,
}

impl<'ctx, 'codegen> ModuleBuilderProvider<'ctx, 'codegen> {
    pub fn make_builder(&self, name: &str) -> ModuleBuilder<'ctx, 'codegen> {
        ModuleBuilder::new(
            name,
            self.codegen_context,
            self.global_constructors_provider.llvm_type(),
        )
    }
}

pub(in crate::codegen) struct ModuleBuilder<'ctx, 'codegen> {
    module: Module<'ctx>,
    global_constructors: Vec<GlobalConstructorOpaque<'ctx>>,
    global_constructor_type: StructType<'ctx>,

    codegen_context: &'codegen CodegenContext<'ctx>,
}

impl<'ctx, 'codegen> ModuleBuilder<'ctx, 'codegen> {
    fn new(
        name: &str,
        codegen_context: &'codegen CodegenContext<'ctx>,
        global_constructor_type: StructType<'ctx>,
    ) -> Self {
        Self {
            module: codegen_context.llvm_context().create_module(name),
            global_constructors: vec![],
            global_constructor_type,
            codegen_context,
        }
    }

    pub fn add_global_constructor(
        &mut self,
        priority: u32,
        constructor: FunctionValue<'ctx>,
        initialized_value: Option<GlobalValue<'ctx>>,
    ) {
        self.global_constructors.push(GlobalConstructorOpaque {
            priority: self
                .codegen_context
                .llvm_context()
                .i32_type()
                .const_int(u64::from(priority), false),
            target: constructor.as_global_value().as_pointer_value(),
            initialized_value: initialized_value.map_or_else(
                || {
                    self.codegen_context
                        .llvm_context()
                        .ptr_type(AddressSpace::default())
                        .const_null()
                },
                GlobalValue::as_pointer_value,
            ),
        });
    }

    pub fn build(self) -> Module<'ctx> {
        let Self {
            module,
            global_constructors,
            global_constructor_type,
            codegen_context: _,
        } = self;

        let global_constructors_array_type =
            global_constructor_type.array_type(u32::try_from(global_constructors.len()).unwrap());

        let global_constructors_value =
            module.add_global(global_constructors_array_type, None, "llvm.global_ctors");

        global_constructors_value.set_linkage(Linkage::Appending);
        let constructors: Vec<_> = global_constructors
            .iter()
            .map(|x| {
                global_constructor_type.const_named_struct(&[
                    x.priority.into(),
                    x.target.into(),
                    x.initialized_value.into(),
                ])
            })
            .collect();
        global_constructors_value
            .set_initializer(&global_constructor_type.const_array(&constructors));

        module
    }

    pub(crate) fn add_global(&self, r#type: impl BasicType<'ctx>, name: &str) -> GlobalValue<'ctx> {
        self.module.add_global(r#type, None, name)
    }

    pub(in crate::codegen) fn build_function(
        &self,
        name: &str,
        // TODO this will be needed so we can generate imports for other modules
        _visibility: FunctionVisibility,
        signature: FunctionType<'ctx>,
        build: impl Fn(FunctionValue<'ctx>, &CodegenContext),
    ) -> FunctionValue<'ctx> {
        let function = self.module.add_function(name, signature, None);

        build(function, self.codegen_context);

        function
    }
}
