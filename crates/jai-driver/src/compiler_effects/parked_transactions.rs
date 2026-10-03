//! Park actual compiler transactions while an owned VM continuation waits.
use super::*;
use std::fmt;

/// A process-local job identity. Only a session can allocate one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilerJobId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerTransactionError {
    Inactive,
    Busy,
    WrongSession,
    WrongPreview,
    ChangedSession,
    LostSourceWorkspace,
    UnfinishedPreview,
    JobLimit,
}
impl fmt::Display for CompilerTransactionError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(match self {
            Self::Inactive => "compiler transaction is not active",
            Self::Busy => "another compiler transaction is active",
            Self::WrongSession => "compiler continuation belongs to another session",
            Self::WrongPreview => "compiler continuation preview belongs to another job",
            Self::ChangedSession => "compiler session changed while its continuation was suspended",
            Self::LostSourceWorkspace => "compiler continuation source workspace was retired",
            Self::UnfinishedPreview => {
                "compiler continuation preview has an unfinished transaction"
            }
            Self::JobLimit => "compiler continuation job identity capacity exhausted",
        })
    }
}
impl std::error::Error for CompilerTransactionError {}

/// Owns the uncommitted host half of a suspended VM job. Dropping it cancels
/// staging; it is deliberately neither clonable nor constructible by callers.
pub struct SuspendedCompilerTransaction {
    id: CompilerJobId,
    session: WorkspaceId,
    revision: u64,
    transaction: Transaction,
}
impl SuspendedCompilerTransaction {
    pub fn id(&self) -> CompilerJobId {
        self.id
    }
    pub fn source_workspace(&self) -> WorkspaceId {
        self.transaction.source_workspace
    }
    pub fn intercepted_workspaces(&self) -> impl ExactSizeIterator<Item = WorkspaceId> + '_ {
        self.transaction.interception.workspaces()
    }
    /// The scheduler supplies events only after the corresponding actual job
    /// boundary. Source code cannot publish events through CompilerRequest.
    pub fn publish_event(&mut self, event: CompilerEvent) -> Result<(), CompilerEventError> {
        self.transaction.interception.publish(event)
    }
}

/// A child's actual speculative compiler state remains private until the
/// waiting parent's complete transaction is validated and committed.
#[derive(Clone)]
pub(super) struct PreparedCompilerState {
    pub(super) base_revision: u64,
    pub(super) workspaces: BTreeMap<WorkspaceId, BuildWorkspace>,
    pub(super) destroyed: BTreeSet<WorkspaceId>,
    messages: Vec<CompilerMessage>,
    error: Option<CompilerMessage>,
    outputs: Vec<CompilerOutput>,
    pub(super) output_bytes: usize,
}
impl PreparedCompilerState {
    fn from_preview(preview: CompilerSession, base_revision: u64) -> Self {
        Self {
            base_revision,
            workspaces: preview.workspaces,
            destroyed: preview.destroyed,
            messages: preview.messages,
            error: preview.error,
            outputs: preview.outputs,
            output_bytes: preview.output_bytes,
        }
    }
    pub(super) fn publish(self, session: &mut CompilerSession) {
        session.workspaces = self.workspaces;
        session.destroyed = self.destroyed;
        session.messages = self.messages;
        session.error = self.error;
        session.outputs = self.outputs;
        session.output_bytes = self.output_bytes;
    }
}

impl CompilerSession {
    pub fn suspend_transaction(
        &mut self,
    ) -> Result<SuspendedCompilerTransaction, CompilerTransactionError> {
        let TransactionState::Active(_) = &self.transaction else {
            return Err(CompilerTransactionError::Inactive);
        };
        let TransactionState::Active(mut transaction) = std::mem::take(&mut self.transaction)
        else {
            unreachable!("checked active transaction")
        };
        let id = match transaction.job {
            Some(id) => id,
            None => allocate_job()?,
        };
        transaction.job = Some(id);
        Ok(SuspendedCompilerTransaction {
            id,
            session: self.root,
            revision: self.revision,
            transaction: *transaction,
        })
    }

    pub(crate) fn validate_suspended(
        &self,
        suspended: &SuspendedCompilerTransaction,
    ) -> Result<(), CompilerTransactionError> {
        if suspended.session != self.root {
            return Err(CompilerTransactionError::WrongSession);
        }
        if !matches!(self.transaction, TransactionState::Idle) {
            return Err(CompilerTransactionError::Busy);
        }
        if suspended.revision != self.revision {
            return Err(CompilerTransactionError::ChangedSession);
        }
        if !self.workspaces.contains_key(&suspended.source_workspace()) {
            return Err(CompilerTransactionError::LostSourceWorkspace);
        }
        Ok(())
    }

    /// Materialize staged inputs in an isolated logical-session clone. No
    /// workspace, compiler console byte, or diagnostic is published to `self`.
    pub fn preview_transaction(
        &self,
        suspended: &SuspendedCompilerTransaction,
    ) -> Result<Self, jai_vm::Error> {
        self.validate_suspended(suspended)
            .map_err(|error| jai_vm::Error::EffectRejected(error.to_string()))?;
        let mut preview = self.clone();
        preview.preview_job = Some(suspended.id);
        let mut projection = suspended.transaction.clone();
        projection.interception = Interception::default();
        preview.transaction = TransactionState::Active(Box::new(projection));
        preview.finish(true)?;
        Ok(preview)
    }

    pub fn resume_transaction(
        &mut self,
        suspended: SuspendedCompilerTransaction,
    ) -> Result<(), CompilerTransactionError> {
        self.validate_suspended(&suspended)?;
        self.transaction = TransactionState::Active(Box::new(suspended.transaction));
        Ok(())
    }

    /// Adopt only a completed private preview. Its child mutations remain part
    /// of the parent's active transaction and roll back with a later failure.
    pub fn resume_transaction_with_preview(
        &mut self,
        suspended: SuspendedCompilerTransaction,
        preview: Self,
    ) -> Result<(), CompilerTransactionError> {
        self.validate_suspended(&suspended)?;
        preview.validate_preview_for(&suspended)?;
        let mut transaction = Transaction::new(suspended.source_workspace());
        transaction.job = Some(suspended.id);
        transaction.interception = suspended.transaction.interception;
        transaction.fatal = suspended.transaction.fatal;
        transaction.prepared = Some(PreparedCompilerState::from_preview(preview, self.revision));
        self.transaction = TransactionState::Active(Box::new(transaction));
        Ok(())
    }
    pub(crate) fn validate_preview_for(
        &self,
        suspended: &SuspendedCompilerTransaction,
    ) -> Result<(), CompilerTransactionError> {
        if self.root != suspended.session {
            return Err(CompilerTransactionError::WrongSession);
        }
        if self.preview_job != Some(suspended.id) {
            return Err(CompilerTransactionError::WrongPreview);
        }
        if !matches!(self.transaction, TransactionState::Idle) {
            return Err(CompilerTransactionError::UnfinishedPreview);
        }
        Ok(())
    }
}

pub(super) fn allocate_job() -> Result<CompilerJobId, CompilerTransactionError> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        value.checked_add(1)
    })
    .map(CompilerJobId)
    .map_err(|_| CompilerTransactionError::JobLimit)
}

#[cfg(test)]
#[path = "parked_transactions_tests.rs"]
mod tests;
