//! Rebuild only after the actual top-level source prefix has completed.
use super::*;

type WorkspaceSourceState = (
    jai_vm::WorkspaceId,
    String,
    Vec<crate::BuildInput>,
    crate::BuildSettings,
);
pub(super) type SourceConfiguration = Vec<WorkspaceSourceState>;

pub(super) fn source_configuration(compiler: &CompilerSession) -> SourceConfiguration {
    compiler
        .workspaces()
        .map(|workspace| {
            (
                workspace.id(),
                workspace.name().to_owned(),
                workspace.inputs().to_vec(),
                workspace.settings().clone(),
            )
        })
        .collect()
}

/// A real prefix checkpoint can release this graph only after its source jobs
/// complete. Output alone cannot request another source graph or skip Full.
pub(super) async fn drive(
    guard: &mut CancellationGuard<'_, '_>,
    baseline: &SourceConfiguration,
) -> Result<bool, WorkspaceJobFailure> {
    poll_fn(|_| {
        let mut state = guard.state.borrow_mut();
        loop {
            let readiness = {
                let Journal {
                    compiler,
                    replay,
                } = state
                    .journal
                    .as_mut()
                    .expect("running prefix retains actual compiler and replay owners");
                guard
                    .session
                    .drive_source_prefix(&mut ReplayEffects::new(compiler, replay))
            };
            match readiness {
                jai_sema::SourcePrefixReadiness::Ready => {
                    state.pending = None;
                    let compiler = &state
                        .journal
                        .as_ref()
                        .expect("completed prefix retains its journals")
                        .compiler;
                    if source_configuration(compiler) != *baseline {
                        return Poll::Ready(Ok(true));
                    }
                    continue;
                }
                jai_sema::SourcePrefixReadiness::Complete => {
                    state.pending = None;
                    let compiler = &state
                        .journal
                        .as_ref()
                        .expect("completed prefix retains its journals")
                        .compiler;
                    return Poll::Ready(Ok(source_configuration(compiler) != *baseline));
                }
                jai_sema::SourcePrefixReadiness::Failed(error) => {
                    state.pending = None;
                    return Poll::Ready(Err(WorkspaceJobFailure::Hard(error)));
                }
                jai_sema::SourcePrefixReadiness::Pending(pending)
                    if crate::discovery_worklists::source_only_wait(&pending) =>
                {
                    state.pending = None;
                    return Poll::Ready(Err(WorkspaceJobFailure::Source(pending)));
                }
                jai_sema::SourcePrefixReadiness::Pending(pending)
                    if pending.dependencies.is_empty() =>
                {
                    state.pending = None;
                    return Poll::Ready(Err(WorkspaceJobFailure::Hard(pending.diagnostic)));
                }
                jai_sema::SourcePrefixReadiness::Pending(pending) => {
                    state.pending = Some(pending);
                    return Poll::Pending;
                }
            }
        }
    })
    .await
}
