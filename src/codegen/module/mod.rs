// TODO review if all the levels of abstractions here still make sense and are needed
use crate::codegen::llvm_struct::representations::OperandValue;
pub(in crate::codegen) mod built_module;

use inkwell::{
    AddressSpace,
    context::Context,
    module::{Linkage, Module},
    types::{BasicType, StructType},
    values::{FunctionValue, GlobalValue, PointerValue},
};

use super::{
    builtins::DebugTypeDefinition,
    context::type_maker::{Function, Procedure},
    llvm_struct::representations::ConstOrValue,
};
use crate::codegen::{
    context_ergonomics::ContextErgonomics,
    llvm_struct::{basic_value_enum::IntoValue, representations::LlvmRepresentation},
};

make_function_type!(GlobalConstructorFunction, ());

llvm_struct! {
    struct GlobalConstructor<'ctx> {
        priority: u32,
        target: *const GlobalConstructorFunction<'ctx>,
        // TODO introduce generics so we can actually set the real type here? or is it not worth
        // the drama?
        initialized_value: Option<*const ()>
    }
}

pub fn register(context: &Context) -> ModuleBuilderProvider<'_> {
    ModuleBuilderProvider {
        global_constructors_provider: GlobalConstructorProvider::new(context),
        context,
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
    module: Module<'ctx>,
    global_constructors: Vec<GlobalConstructorOpaque<'ctx>>,
    global_constructor_type: StructType<'ctx>,

    context: &'ctx Context,
}

impl<'ctx> ModuleBuilder<'ctx> {
    fn new(name: &str, context: &'ctx Context, global_constructor_type: StructType<'ctx>) -> Self {
        let module = context.create_module(name);
        // TODO this is a hack, it will link to the function defined in the main module, but we
        // should really invest into some real debug infrastructure
        module.add_function(
            "debug_type_definition",
            DebugTypeDefinition::llvm_type(context),
            None,
        );
        Self {
            module,
            global_constructors: vec![],
            global_constructor_type,
            context,
        }
    }

    pub fn add_global_constructor(
        &mut self,
        priority: u32,
        constructor: &GlobalConstructorFunction<'ctx>,
        initialized_value: Option<GlobalValue<'ctx>>,
    ) {
        // TODO the manual creation of ConstOrValue isn't very pretty, let's look into some
        // automatic coercion, maybe implement into not generially, but for each type, since we
        // have the macros for LlvmRepresentation anyway?
        self.global_constructors.push(GlobalConstructorOpaque {
            priority: ConstOrValue::Const(priority),
            target: ConstOrValue::Value(constructor.as_global_value().as_pointer_value()),
            initialized_value: ConstOrValue::Value(initialized_value.map_or_else(
                || self.context.ptr_type(AddressSpace::default()).const_null(),
                GlobalValue::as_pointer_value,
            )),
        });
    }

    pub(crate) fn add_global(&self, r#type: impl BasicType<'ctx>, name: &str) -> GlobalValue<'ctx> {
        self.module.add_global(r#type, None, name)
    }

    pub(in crate::codegen) fn build_procedure<
        TArguments,
        TProcedure: Procedure<'ctx, TArguments>,
    >(
        &self,
        build: impl Fn(FunctionValue<'ctx>, &'ctx Context, &Module<'ctx>),
    ) -> TProcedure {
        let signature = TProcedure::llvm_type(self.context);
        let function = self.module.add_function(TProcedure::NAME, signature, None);

        build(function, self.context, &self.module);

        TProcedure::new(function)
    }

    pub(in crate::codegen) fn build_function<
        TReturn: LlvmRepresentation<'ctx>,
        TArguments,
        TFunction: Function<'ctx, TReturn, TArguments>,
    >(
        &self,
        build: impl Fn(FunctionValue<'ctx>, &'ctx Context, &Module<'ctx>),
    ) -> TFunction {
        let signature = TFunction::llvm_type(self.context);
        let function = self.module.add_function(TFunction::NAME, signature, None);

        build(function, self.context, &self.module);

        TFunction::new(function)
    }

    pub fn build(self) -> Module<'ctx> {
        let Self {
            module,
            global_constructors,
            global_constructor_type,
            context: _,
        } = self;

        let global_constructors_array_type =
            global_constructor_type.array_type(u32::try_from(global_constructors.len()).unwrap());

        let global_constructors_value =
            module.add_global(global_constructors_array_type, None, "llvm.global_ctors");

        global_constructors_value.set_linkage(Linkage::Appending);
        let constructors: Vec<_> = global_constructors
            .iter()
            .map(|x| {
                // TODO this is very hacky, perhaps create $name Const for structs which are always
                // made from consts?
                let ConstOrValue::Const(priority) = x.priority else {
                    todo!();
                };
                let ConstOrValue::Value(target) = x.target else {
                    todo!();
                };
                let ConstOrValue::Value(initialized_value) = x.initialized_value else {
                    todo!();
                };

                global_constructor_type.const_named_struct(&[
                    self.context.const_u32(priority).into(),
                    target.into(),
                    initialized_value.into(),
                ])
            })
            .collect();
        global_constructors_value
            .set_initializer(&global_constructor_type.const_array(&constructors));

        module
    }
}
