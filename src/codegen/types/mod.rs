pub(in crate::codegen) mod functions;
//TODO rename -> values
pub(in crate::codegen) mod value;

// TODO this should go into its own mod for classes, once we have classes...
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// The value of 0 means no class
pub(super) struct ClassId(u16);

impl ClassId {}
