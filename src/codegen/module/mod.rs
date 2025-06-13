use std::collections::HashMap;

use inkwell::{
    AddressSpace,
    module::{Linkage, Module},
    types::{BasicType, StructType},
    values::{FunctionValue, GlobalValue, PointerValue},
};

use super::{
    builtins::DebugTypeDefinition,
    context::{
        CodegenContext,
        type_maker::{Function, Procedure},
    },
};
use crate::codegen::{
    context_ergonomics::ContextErgonomics,
    llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
};

make_function_type!(GlobalConstructorFunction, ());

llvm_struct! {
    struct GlobalConstructor {
        priority: u32,
        // TODO this should be probably connected with GlobalConstructionFunction in some way, but
        // uhh that requires some meta-thinking
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

pub(in crate::codegen) struct PublicFunction<'ctx>(
    Box<dyn Fn(&Module<'ctx>) -> FunctionValue<'ctx> + 'ctx>,
);
// TODO the key sould be an Identifier, not a string
pub(in crate::codegen) struct PublicFunctionLinks<'ctx>(HashMap<String, PublicFunction<'ctx>>);

impl<'ctx> PublicFunctionLinks<'ctx> {
    pub(in crate::codegen) fn register(
        &self,
        module: &Module<'ctx>,
    ) -> HashMap<String, FunctionValue<'ctx>> {
        let mut result = HashMap::new();

        for (name, function) in &self.0 {
            let function_value = function.0(module);
            result.insert(name.to_string(), function_value);
        }

        result
    }
}

pub(in crate::codegen) struct ModuleBuilder<'ctx, 'codegen> {
    module: Module<'ctx>,
    global_constructors: Vec<GlobalConstructorOpaque<'ctx>>,
    global_constructor_type: StructType<'ctx>,

    codegen_context: &'codegen CodegenContext<'ctx>,
    public_functions: PublicFunctionLinks<'ctx>,
}

impl<'ctx, 'codegen> ModuleBuilder<'ctx, 'codegen> {
    fn new(
        name: &str,
        codegen_context: &'codegen CodegenContext<'ctx>,
        global_constructor_type: StructType<'ctx>,
    ) -> Self {
        let module = codegen_context.llvm_context().create_module(name);
        // TODO this is a hack, it will link to the function defined in the main module, but we
        // should really invest into some real debug infrastructure
        module.add_function(
            "debug_type_definition",
            DebugTypeDefinition::llvm_type(codegen_context.llvm_context()),
            None,
        );
        Self {
            module,
            global_constructors: vec![],
            global_constructor_type,
            codegen_context,
            public_functions: PublicFunctionLinks(HashMap::new()),
        }
    }

    pub fn add_global_constructor(
        &mut self,
        priority: u32,
        constructor: &GlobalConstructorFunction<'ctx>,
        initialized_value: Option<GlobalValue<'ctx>>,
    ) {
        self.global_constructors.push(GlobalConstructorOpaque {
            priority: self.codegen_context.llvm_context().const_u32(priority),
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

    pub(crate) fn add_global(&self, r#type: impl BasicType<'ctx>, name: &str) -> GlobalValue<'ctx> {
        self.module.add_global(r#type, None, name)
    }

    pub(in crate::codegen) fn build_procedure<
        TArguments,
        TProcedure: Procedure<'ctx, TArguments>,
    >(
        &mut self,
        visibility: FunctionVisibility,
        build: impl Fn(FunctionValue<'ctx>, &CodegenContext, &Module<'ctx>),
    ) -> TProcedure {
        let signature = TProcedure::llvm_type(self.codegen_context.llvm_context());
        let function = self.module.add_function(TProcedure::NAME, signature, None);

        build(function, self.codegen_context, &self.module);

        if visibility == FunctionVisibility::Public {
            self.public_functions.0.insert(
                TProcedure::NAME.to_string(),
                PublicFunction(Box::new(move |module| {
                    module.add_function(TProcedure::NAME, signature, Some(Linkage::External))
                })),
            );
        }

        TProcedure::new(function)
    }

    pub(in crate::codegen) fn build_function<
        TReturn: LlvmRepresentation<'ctx>,
        TArguments,
        TFunction: Function<'ctx, TReturn, TArguments>,
    >(
        &mut self,
        visibility: FunctionVisibility,
        build: impl Fn(FunctionValue<'ctx>, &CodegenContext, &Module<'ctx>),
    ) -> TFunction {
        let signature = TFunction::llvm_type(self.codegen_context.llvm_context());
        let function = self.module.add_function(TFunction::NAME, signature, None);

        build(function, self.codegen_context, &self.module);

        if visibility == FunctionVisibility::Public {
            self.public_functions.0.insert(
                TFunction::NAME.to_string(),
                PublicFunction(Box::new(move |module| {
                    module.add_function(TFunction::NAME, signature, Some(Linkage::External))
                })),
            );
        }

        TFunction::new(function)
    }

    pub fn build(self) -> (Module<'ctx>, PublicFunctionLinks<'ctx>) {
        let Self {
            module,
            global_constructors,
            global_constructor_type,
            codegen_context: _,
            public_functions,
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

        (module, public_functions)
    }
}
