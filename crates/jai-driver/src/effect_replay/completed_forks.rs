//! Adopt genuine completed child journals without changing their parent lineage.
use super::*;

impl EffectReplayCache {
    pub(crate) fn adopt_completed_fork(&mut self, branch: Self) -> Result<(), jai_vm::Error> {
        if !branch
            .branch_parent
            .as_ref()
            .is_some_and(|parent| Arc::ptr_eq(parent, &self.identity))
        {
            return Err(jai_vm::Error::EffectRejected(
                "completed source replay belongs to another cache".into(),
            ));
        }
        if !self.suspended.is_empty() || !branch.suspended.is_empty() {
            return Err(jai_vm::Error::EffectRejected(
                "completed source replay has unfinished compiler jobs".into(),
            ));
        }
        if self.runs.iter().any(|(origin, requests)| {
            branch
                .runs
                .get(origin)
                .is_none_or(|candidate| candidate.as_ref() != requests.as_ref())
        }) {
            return Err(jai_vm::Error::EffectRejected(
                "completed source replay changed an earlier source trace".into(),
            ));
        }
        if branch.bytes > self.limits.bytes || branch.runs.len() > self.limits.runs {
            return Err(jai_vm::Error::EffectRejected(
                "completed source replay exceeds its parent limits".into(),
            ));
        }
        // The receiver's identity and parent seal still own the active source
        // controller. Only real completed, source-owned traces cross over.
        self.runs = branch.runs;
        self.bytes = branch.bytes;
        Ok(())
    }
}
