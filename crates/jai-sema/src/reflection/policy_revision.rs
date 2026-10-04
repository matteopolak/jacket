//! Policy revisions publish fresh descriptors without mutating earlier graphs.
use super::*;
use jai_types::{
    PreparedRecordReflectionTransaction, RecordReflectionCommit, RecordReflectionTransaction,
};

const MAX_DESCRIPTOR_POLICY_REVISIONS: u64 = 65_536;

/// Both exclusive owners remain borrowed through host publication. All checks
/// and allocations have completed; applying the receipt cannot fail or race a
/// source policy update in the canonical arena.
pub(crate) struct PreparedReflectionPolicy<'a> {
    meta: &'a mut MetaContext,
    transaction: PreparedRecordReflectionTransaction<'a>,
}
impl PreparedReflectionPolicy<'_> {
    pub(crate) fn apply(self) -> RecordReflectionCommit {
        let commit = self.transaction.apply();
        if !commit.is_empty() {
            self.meta.revise_reflection_storage();
        }
        commit
    }
}

impl MetaContext {
    #[cfg(test)]
    pub(crate) fn reflection_policy_epoch(&self) -> u64 {
        self.descriptor_policy_epoch
    }

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
        Ok(self
            .prepare_reflection_policy_transaction(types, transaction, location)?
            .apply())
    }

    pub(crate) fn prepare_reflection_policy_transaction<'a>(
        &'a mut self,
        types: &'a mut TypeRegistry,
        transaction: RecordReflectionTransaction,
        location: jai_source::SourceSpan,
    ) -> Result<PreparedReflectionPolicy<'a>, Diagnostic> {
        self.validate_reflection_policy_transaction(types, &transaction, location)?;
        let transaction = transaction
            .prepare(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        Ok(PreparedReflectionPolicy {
            meta: self,
            transaction,
        })
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
