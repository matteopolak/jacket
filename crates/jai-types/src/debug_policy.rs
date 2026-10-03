//! Source debug suppression is independent of procedure ABI and diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DebugPolicy {
    #[default]
    Emit,
    Suppress,
}

impl DebugPolicy {
    pub fn emits(self) -> bool {
        self == Self::Emit
    }

    /// Nested expansion cannot reenable locations suppressed by its caller.
    pub fn nested(self, declaration: Self) -> Self {
        if self.emits() && declaration.emits() {
            Self::Emit
        } else {
            Self::Suppress
        }
    }
}
