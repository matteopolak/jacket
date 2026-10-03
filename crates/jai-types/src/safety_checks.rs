//! Closed checking decisions shared by constant evaluation and typed operations.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A resolved source check decision, independent of memory or execution validity.
pub enum CheckMode {
    Enabled,
    Disabled,
}

impl CheckMode {
    pub fn enabled(self) -> bool {
        self == Self::Enabled
    }
}
