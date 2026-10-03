//! Reject active macro-body returns without rejecting returns in inserted caller syntax.
use super::*;

impl Resolver<'_> {
    pub(crate) fn reject_expanded_return(&self, span: Span) -> Result<(), Diagnostic> {
        if self
            .meta
            .codes
            .return_regions
            .last()
            .is_some_and(|&(owner, allowed)| owner == self.procedure && !allowed)
        {
            return Err(Diagnostic::new(
                span,
                "return statements in an expanded procedure require expansion result binding",
            ));
        }
        Ok(())
    }
}
