#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
// The value of 0 means no class
pub(in crate::codegen) struct ClassId(u16);

impl ClassId {
    #[allow(unused)]
    pub(in crate::codegen) const fn as_u16(self) -> u16 {
        self.0
    }

    pub(crate) const fn none() -> Self {
        Self(0)
    }
}
