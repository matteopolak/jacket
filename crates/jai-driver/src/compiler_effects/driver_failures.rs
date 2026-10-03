//! Actual driver failures are facts, separate from discarded source effects.
use super::*;

impl CompilerSession {
    pub(crate) fn require_source_idle(&self) -> Result<(), jai_vm::Error> {
        if matches!(self.transaction, TransactionState::Idle) {
            Ok(())
        } else {
            Err(jai_vm::Error::EffectRejected(
                "source resolution requires an idle compiler session".into(),
            ))
        }
    }
    pub(crate) fn source_failure(&self, workspace: WorkspaceId) -> Option<&CompilerMessage> {
        self.workspaces.get(&workspace)?.error.as_ref()
    }

    pub(crate) fn record_source_failure(
        &mut self,
        workspace: WorkspaceId,
        error: &crate::Error,
    ) -> Result<(), jai_vm::Error> {
        if !matches!(self.transaction, TransactionState::Idle) {
            return Err(jai_vm::Error::EffectRejected(
                "cannot publish a driver failure into an active compiler transaction".into(),
            ));
        }
        let message = match error {
            crate::Error::CompilerReport(message) => message.clone(),
            error => CompilerMessage {
                level: MessageLevel::Error,
                // The actual diagnostic is already rendered with its original
                // source coordinates. Do not guess a new message location.
                text: error.to_string(),
                location: None,
            },
        };
        let Some(target) = self.workspaces.get_mut(&workspace) else {
            return Err(jai_vm::Error::EffectRejected(
                "driver failure refers to an unavailable workspace".into(),
            ));
        };
        if target.status == WorkspaceStatus::Failed && target.error.as_ref() == Some(&message) {
            return Ok(());
        }
        target.status = WorkspaceStatus::Failed;
        target.error = Some(message.clone());
        self.messages.push(message);
        self.error = self
            .workspaces
            .values()
            .filter(|workspace| workspace.status == WorkspaceStatus::Failed)
            .find_map(|workspace| workspace.error.clone());
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }
}
