//! Policy revisions publish fresh descriptors without mutating earlier graphs.
use super::*;
use jai_types::{RecordReflectionCommit, RecordReflectionTransaction};

const MAX_DESCRIPTOR_POLICY_REVISIONS: u64 = 65_536;

impl MetaContext {
    pub(crate) fn validate_reflection_policy_transaction(
        &self,
        types: &TypeRegistry,
        transaction: &RecordReflectionTransaction,
        location: jai_source::SourceSpan,
    ) -> Result<(), Diagnostic> {
        transaction
            .validate(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        if !transaction.is_empty()
            && self.descriptor_policy_epoch >= MAX_DESCRIPTOR_POLICY_REVISIONS
        {
            return Err(Diagnostic::at_source(
                location,
                "reflection descriptor policy revision limit exceeded",
            ));
        }
        Ok(())
    }

    /// The caller's compiler publication journal validates this operation before
    /// committing host effects. No registry policy changes occur during staging.
    pub(crate) fn commit_reflection_policy_transaction(
        &mut self,
        types: &mut TypeRegistry,
        transaction: RecordReflectionTransaction,
        location: jai_source::SourceSpan,
    ) -> Result<RecordReflectionCommit, Diagnostic> {
        self.validate_reflection_policy_transaction(types, &transaction, location)?;
        let commit = transaction
            .commit(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        if !commit.is_empty() {
            self.revise_reflection_storage();
        }
        Ok(commit)
    }

    /// Source directives can change a policy before a compiler journal exists.
    /// Detect those genuine registry changes at the descriptor query boundary.
    pub(crate) fn synchronize_reflection_policy(
        &mut self,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let changed =
            self.storage_policies
                .iter()
                .try_fold(false, |changed, (&record, &policy)| {
                    types
                        .record_reflection_policy(record)
                        .map(|current| changed || current != policy)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))
                })?;
        if changed {
            if self.descriptor_policy_epoch >= MAX_DESCRIPTOR_POLICY_REVISIONS {
                return Err(Diagnostic::new(
                    span,
                    "reflection descriptor policy revision limit exceeded",
                ));
            }
            self.revise_reflection_storage();
        }
        Ok(())
    }

    fn revise_reflection_storage(&mut self) {
        self.descriptor_policy_epoch += 1;
        self.storage.clear();
        self.storage_policies.clear();
        self.runtime_info_snapshots.clear();
        // The builder retains immutable objects already published. Their Arc
        // closures and descriptor receipts remain valid, and new reservations
        // cannot alias those objects. Its ordinary cumulative budgets still
        // bound repeated revisions.
    }
}

#[cfg(test)]
mod tests;
