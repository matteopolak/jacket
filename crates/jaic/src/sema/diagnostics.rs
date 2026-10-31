//! What this compile has to report besides its first error: warnings, the reasons deferred items
//! failed, and what each declaration was found to depend on.
use super::body_probe::{Probed, W};
use crate::diagnostics::Diagnostic;
use crate::sema::dependencies;

#[derive(Default)]
pub struct Diagnostics {
    pub warnings: Probed<Vec<Diagnostic>, { W::DiagWarnings as usize }>,
    /// Why each deferred item failed; shown with a later error if it is never retried.
    pub deferred_errors: Probed<Vec<Box<Diagnostic>>, { W::DiagDeferredErrors as usize }>,
    pub(in crate::sema) dependencies:
        Probed<dependencies::Dependencies, { W::DiagDependencies as usize }>,
}
