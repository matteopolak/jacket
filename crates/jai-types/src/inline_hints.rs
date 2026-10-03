//! Source definition or call inlining policy, independent of the canonical callable type.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InlineHint {
    /// Leave the decision to the backend's optimization policy.
    #[default]
    Automatic,
    /// The source explicitly requires inlining this definition or call.
    Always,
    /// The source explicitly prohibits inlining this definition or call.
    Never,
}
