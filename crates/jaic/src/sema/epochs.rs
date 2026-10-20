//! Counters and flags that say how far checking has got: what may change a cached failure
//! (`lower_epoch`), and which phase of deferred expansion is running.

#[derive(Default)]
pub struct Epochs {
    /// Bumped whenever a name, import, `using` or expanded item becomes visible in a scope
    /// that is not a body's own (`Scope::namespace_changed`): part of `lower_epoch`.
    pub(in crate::sema) namespace_generation: u64,
    /// Lookups that reached an undefined `#placeholder`: a pending top-level item whose
    /// expansion did so waits (a metaprogram may define it later) until `finish_program`.
    pub placeholder_misses: u64,
    /// Lookups that reached something still being computed (a signature, entity or layout in
    /// progress): a failure that did so may succeed once that finishes, so it is not memoized.
    pub in_progress_misses: u64,
    /// Entities being resolved right now (nested): a conditional item whose condition names one
    /// of them fails as circular only because the outer resolution has not finished.
    pub resolving_depth: u32,
    /// Set while `expand_all` retries deferred items: failures are then final.
    pub retrying_pending: bool,
    /// Set while the plain `#if` pass expands an item: lookups then leave other pending items
    /// alone, so evaluating `OS` cannot run a sibling `using x: T` or `#insert` early.
    pub lookup_without_expansion: bool,
    /// Set by `finish_program`: pending items no longer wait for placeholders.
    pub placeholders_final: bool,
}
