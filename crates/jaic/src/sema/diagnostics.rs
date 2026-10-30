//! What this compile has to report besides its first error: warnings, the reasons deferred items
//! failed, and what each declaration was found to depend on.
use crate::diagnostics::Diagnostic;
use crate::sema::dependencies;

#[derive(Default)]
pub struct Diagnostics {
    pub warnings: Vec<Diagnostic>,
    /// Why each deferred item failed; shown with a later error if it is never retried.
    pub deferred_errors: Vec<Box<Diagnostic>>,
    pub(in crate::sema) dependencies: dependencies::Dependencies,
}
