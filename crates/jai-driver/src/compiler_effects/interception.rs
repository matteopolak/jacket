//! Owned event subscriptions belong to one actual compiler transaction/job.
use super::*;
use std::{collections::VecDeque, fmt};

const MAX_QUEUED_EVENTS: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerEventError {
    UnsubscribedWorkspace,
    QueueLimit,
    InvalidPendingCount,
}
impl fmt::Display for CompilerEventError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(match self {
            Self::UnsubscribedWorkspace => {
                "compiler event workspace has no subscription in this job"
            }
            Self::QueueLimit => "compiler event queue limit exceeded",
            Self::InvalidPendingCount => {
                "compiler event pending count exceeds the source s32 field"
            }
        })
    }
}
impl std::error::Error for CompilerEventError {}

#[derive(Clone, Default)]
pub(super) struct Interception {
    workspaces: BTreeMap<WorkspaceId, InterceptFlags>,
    events: VecDeque<CompilerEvent>,
    pending: Option<(EffectKey, CompilerJobId)>,
}
impl Interception {
    pub(super) fn is_waiting(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn workspaces(&self) -> impl ExactSizeIterator<Item = WorkspaceId> + '_ {
        self.workspaces.keys().copied()
    }
    pub(super) fn publish(&mut self, event: CompilerEvent) -> Result<(), CompilerEventError> {
        if !self.workspaces.contains_key(&event.workspace()) {
            return Err(CompilerEventError::UnsubscribedWorkspace);
        }
        if matches!(event, CompilerEvent::Phase { phase: CompilerPhase::Typechecked { pending_count }, .. } if pending_count > i32::MAX as u32)
        {
            return Err(CompilerEventError::InvalidPendingCount);
        }
        if self.events.len() >= MAX_QUEUED_EVENTS || self.events.try_reserve(1).is_err() {
            return Err(CompilerEventError::QueueLimit);
        }
        self.events.push_back(event);
        Ok(())
    }
}

impl CompilerSession {
    pub(super) fn begin_intercept(
        &mut self,
        workspace: WorkspaceId,
        flags: InterceptFlags,
    ) -> EffectOutcome {
        if flags.requests_ast() {
            return self.reject("compiler AST interception is unavailable; use SKIP_ALL for actual phase and completion events");
        }
        if flags.requests_performance() {
            return self.reject("compiler performance interception is unavailable");
        }
        if !matches!(self.get_name(workspace), EffectOutcome::Ready(_)) {
            return self.reject("compiler interception requires an active workspace");
        }
        let TransactionState::Active(transaction) = &mut self.transaction else {
            unreachable!("checked active workspace")
        };
        transaction.interception.workspaces.insert(workspace, flags);
        EffectOutcome::Ready(CompilerResponse::Unit)
    }
    pub(super) fn end_intercept(&mut self, workspace: WorkspaceId) -> EffectOutcome {
        let TransactionState::Active(transaction) = &mut self.transaction else {
            return self.reject("compiler interception requires an active transaction");
        };
        if transaction
            .interception
            .workspaces
            .remove(&workspace)
            .is_none()
        {
            return self.reject("compiler workspace has no interception subscription in this job");
        }
        transaction
            .interception
            .events
            .retain(|event| event.workspace() != workspace);
        if transaction.interception.workspaces.is_empty() {
            transaction.interception.pending = None;
        }
        EffectOutcome::Ready(CompilerResponse::Unit)
    }
    pub(super) fn wait_for_message(&mut self) -> EffectOutcome {
        let TransactionState::Active(transaction) = &mut self.transaction else {
            return self.reject("compiler wait requires an active transaction");
        };
        if let Some((key, _)) = transaction.interception.pending {
            return EffectOutcome::Pending(key);
        }
        if let Some(event) = transaction.interception.events.pop_front() {
            return EffectOutcome::Ready(CompilerResponse::Message(event));
        }
        if transaction.interception.workspaces.is_empty() {
            return self.reject("compiler wait has no intercepted workspace");
        }
        let job = if let Some(job) = transaction.job {
            job
        } else {
            match parked_transactions::allocate_job() {
                Ok(job) => {
                    transaction.job = Some(job);
                    job
                }
                Err(error) => return self.reject(error.to_string()),
            }
        };
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let key = match NEXT.try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        }) {
            Ok(key) => EffectKey(key),
            Err(_) => return self.reject("compiler wait identity capacity exhausted"),
        };
        transaction.interception.pending = Some((key, job));
        EffectOutcome::Pending(key)
    }
    pub(super) fn poll_message(&mut self, key: EffectKey) -> EffectOutcome {
        let TransactionState::Active(transaction) = &mut self.transaction else {
            return EffectOutcome::Rejected("compiler wait transaction is not active".into());
        };
        let Some((expected, job)) = transaction.interception.pending else {
            return EffectOutcome::Rejected("compiler wait was not issued by this job".into());
        };
        if expected != key || transaction.job != Some(job) {
            return EffectOutcome::Rejected(
                "compiler wait belongs to another job or request".into(),
            );
        }
        if let Some(event) = transaction.interception.events.pop_front() {
            transaction.interception.pending = None;
            EffectOutcome::Ready(CompilerResponse::Message(event))
        } else {
            EffectOutcome::Pending(key)
        }
    }
}

#[cfg(test)]
#[path = "interception_tests.rs"]
mod tests;
