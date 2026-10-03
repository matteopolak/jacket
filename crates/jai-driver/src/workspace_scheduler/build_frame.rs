//! Keep every source round and retained job inside one private build transaction.
use super::*;

pub(super) struct BuildFrame {
    pub(super) base_revision: u64,
    pub(super) compiler: CompilerSession,
    pub(super) replay: EffectReplayCache,
    pub(super) state: BuildState,
}
pub(super) struct PreviewSelection {
    pub(super) parent: SourceOrigin,
    pub(super) receivers: Vec<WorkspaceId>,
}
pub(super) struct SourceWaitFact {
    pub(super) workspace: WorkspaceId,
    pub(super) source_parsed: bool,
    pub(super) error: Error,
    pub(super) pending: crate::SourceDiscoveryPending,
}
pub(super) struct BuildState {
    pub(super) pass: usize,
    pub(super) before: Vec<Snapshot>,
    pub(super) next: usize,
    pub(super) results: BTreeMap<u64, ScheduledWorkspace>,
    pub(super) failure: Option<(WorkspaceId, Error)>,
    pub(super) active: Option<ActiveWorkspaceJob>,
    pub(super) selection: Option<PreviewSelection>,
    pub(super) waits: BTreeMap<u64, SourceWaitFact>,
    pub(super) failed: BTreeMap<u64, (WorkspaceId, bool)>,
    pub(super) failed_workspace: Option<WorkspaceId>,
}

impl BuildState {
    pub(super) fn new(session: &CompilerSession, selection: Option<PreviewSelection>) -> Self {
        Self {
            pass: 1,
            before: snapshots(session),
            next: 0,
            results: BTreeMap::new(),
            failure: None,
            active: None,
            selection,
            waits: BTreeMap::new(),
            failed: BTreeMap::new(),
            failed_workspace: None,
        }
    }
}

impl WorkspaceScheduler {
    pub(crate) fn child_canceller(&self) -> ChildJobCanceller {
        let children = Rc::downgrade(&self.children);
        Rc::new(move |ids| {
            let Some(children) = children.upgrade() else {
                return Ok(());
            };
            retire_children(&children, ids)
        })
    }

    pub(super) fn cancel_frame(&mut self, frame: &mut BuildFrame) -> Result<(), jai_vm::Error> {
        cancel_frame(&self.children, frame)
    }

    pub(super) fn retire_children(
        &mut self,
        ids: Vec<crate::CompilerJobId>,
    ) -> Result<(), jai_vm::Error> {
        retire_children(&self.children, ids)
    }

    pub(super) fn retire_child(&mut self, id: crate::CompilerJobId) -> Result<(), jai_vm::Error> {
        retire_child(&self.children, id)
    }

    pub(super) fn drive_build(
        &mut self,
        session: &mut CompilerSession,
        replay: &mut EffectReplayCache,
        state: &mut BuildState,
    ) -> Result<SchedulerReadiness, SchedulerError> {
        'rounds: loop {
            if state.pass > self.options.limits.passes {
                return Err(SchedulerError::Limit("passes"));
            }
            self.check_physical()?;
            self.check_limits(&state.before)?;
            while state.next < state.before.len() || state.active.is_some() {
                let mut active = if let Some(active) = state.active.take() {
                    if session.committed_revision() != active.revision {
                        state.active = Some(active);
                        return Err(SchedulerError::ChangedPendingSession);
                    }
                    active
                } else {
                    let workspace = state.before[state.next].clone();
                    state.next += 1;
                    if state.selection.as_ref().is_some_and(|selection| {
                        workspace.id == selection.parent.workspace
                            || !selection.receivers.contains(&workspace.id)
                    }) {
                        continue;
                    }
                    if session.workspace(workspace.id).is_none() {
                        continue;
                    }
                    if workspace.id != session.root() && workspace.inputs.is_empty() {
                        state.results.insert(
                            workspace.id.get(),
                            ScheduledWorkspace {
                                id: workspace.id,
                                name: workspace.name,
                                output: WorkspaceOutput::AwaitingInputs,
                            },
                        );
                        continue;
                    }
                    state.failed_workspace = Some(workspace.id);
                    let WorkspaceSources {
                        entry,
                        options,
                        provider,
                        virtual_paths,
                    } = self.source_snapshot(&workspace, session)?;
                    ActiveWorkspaceJob {
                        workspace,
                        revision: session.committed_revision(),
                        job: ActiveSourceJob::Discovery {
                            job: crate::PreparedGraphJob::new(
                                entry,
                                options,
                                provider,
                                session.clone(),
                                replay.fork_committed(),
                            ),
                            virtual_paths,
                        },
                    }
                };
                loop {
                    let parked = active.job.suspended_job_ids();
                    if let ActiveSourceJob::Discovery {
                        job,
                        virtual_paths,
                    } = &mut active.job
                    {
                        let progress = match job.poll() {
                            Ok(progress) => progress,
                            Err(error) => {
                                let retired = self.retire_children(parked);
                                state.active = Some(active);
                                retired?;
                                return Err(error.into());
                            }
                        };
                        let retained = if matches!(&progress, crate::GraphJobProgress::Pending(_)) {
                            job.suspended_job_ids()
                        } else {
                            vec![]
                        };
                        let retired = parked
                            .into_iter()
                            .filter(|id| !retained.contains(id))
                            .collect();
                        if let Err(error) = self.retire_children(retired) {
                            state.active = Some(active);
                            return Err(error.into());
                        }
                        match progress {
                            crate::GraphJobProgress::Pending(source) => {
                                match active.job.service_pending(self) {
                                    Ok(true) => continue,
                                    Ok(false) => {}
                                    Err(error) => {
                                        state.active = Some(active);
                                        return Err(error);
                                    }
                                }
                                let pending = SchedulerPending {
                                    workspace: active.workspace.id,
                                    source,
                                };
                                state.active = Some(active);
                                return Ok(SchedulerReadiness::Pending(pending));
                            }
                            crate::GraphJobProgress::SourceWait {
                                pending,
                                error,
                            } => {
                                // Only an actual source dependency may retain
                                // completed input changes for another graph round.
                                if let Some((compiler, traces)) = job.take_terminal_journals()
                                    && snapshots(&compiler) != snapshots(session)
                                {
                                    replay.adopt_completed_fork(traces)?;
                                    *session = compiler;
                                    restart_source_round(state, session);
                                    continue 'rounds;
                                }
                                state.waits.insert(
                                    active.workspace.id.get(),
                                    SourceWaitFact {
                                        workspace: active.workspace.id,
                                        source_parsed: false,
                                        error,
                                        pending,
                                    },
                                );
                                break;
                            }
                            crate::GraphJobProgress::Failed(error) => {
                                session.record_source_failure(active.workspace.id, &error)?;
                                state.failed.insert(
                                    active.workspace.id.get(),
                                    (active.workspace.id, false),
                                );
                                state.failure.get_or_insert((active.workspace.id, error));
                                break;
                            }
                            crate::GraphJobProgress::Complete(result) => {
                                let crate::GraphJobResult {
                                    unit,
                                    compiler,
                                    replay: traces,
                                } = *result;
                                self.retain_physical(unit.graph(), virtual_paths)?;
                                let options = jai_sema::ResolveOptions {
                                    target: Some(self.options.target.clone()),
                                    compile_time_limits: self.options.compile_time_limits,
                                    compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                                        unit.graph(),
                                        &self.graph_options.import_dirs,
                                        active.workspace.id,
                                    )),
                                    file_abi: jai_sema::FileAbiBindingContext::allocator_from_graph(
                                        unit.graph(),
                                        &self.graph_options.import_dirs,
                                        self.options.target.clone(),
                                    ),
                                    ..Default::default()
                                };
                                active.job = ActiveSourceJob::Binding(
                                    crate::PreparedWorkspaceJob::with_source_baseline(
                                        unit, options, compiler, traces, session,
                                    ),
                                );
                                continue;
                            }
                        }
                    }
                    let ActiveSourceJob::Binding(job) = &mut active.job else {
                        unreachable!()
                    };
                    let progress = match job.poll() {
                        Ok(progress) => progress,
                        Err(error) => {
                            let retired = self.retire_children(parked);
                            state.active = Some(active);
                            retired?;
                            return Err(error.into());
                        }
                    };
                    let retained = if matches!(&progress, crate::WorkspaceJobProgress::Pending(_)) {
                        job.suspended_job_ids()
                    } else {
                        vec![]
                    };
                    let retired = parked
                        .into_iter()
                        .filter(|id| !retained.contains(id))
                        .collect();
                    if let Err(error) = self.retire_children(retired) {
                        state.active = Some(active);
                        return Err(error.into());
                    }
                    match progress {
                        crate::WorkspaceJobProgress::SourceRebuild(result) => {
                            let crate::WorkspaceSourceRebuild {
                                unit: _,
                                compiler,
                                replay: traces,
                            } = *result;
                            if snapshots(&compiler) == snapshots(session) {
                                return Err(jai_vm::Error::InvalidIr(
                                    "completed source prefix has no source configuration change",
                                )
                                .into());
                            }
                            replay.adopt_completed_fork(traces)?;
                            *session = compiler;
                            restart_source_round(state, session);
                            continue 'rounds;
                        }
                        crate::WorkspaceJobProgress::Pending(source) => {
                            match active.job.service_pending(self) {
                                Ok(true) => continue,
                                Ok(false) => {}
                                Err(error) => {
                                    state.active = Some(active);
                                    return Err(error);
                                }
                            }
                            let pending = SchedulerPending {
                                workspace: active.workspace.id,
                                source,
                            };
                            state.active = Some(active);
                            return Ok(SchedulerReadiness::Pending(pending));
                        }
                        crate::WorkspaceJobProgress::SourceWait(source) => {
                            let pending = crate::SourceDiscoveryPending {
                                request: crate::SourceDiscoveryRequest::LibraryPreparation(
                                    source.source.expect(
                                        "source-only binding wait retains its actual cause",
                                    ),
                                ),
                                diagnostic: source.diagnostic,
                            };
                            let error = job.unit().located(pending.diagnostic.clone());
                            if let Some((compiler, traces)) = job.take_terminal_journals()
                                && snapshots(&compiler) != snapshots(session)
                            {
                                replay.adopt_completed_fork(traces)?;
                                *session = compiler;
                                restart_source_round(state, session);
                                continue 'rounds;
                            }
                            state.waits.insert(
                                active.workspace.id.get(),
                                SourceWaitFact {
                                    workspace: active.workspace.id,
                                    source_parsed: true,
                                    error,
                                    pending,
                                },
                            );
                            break;
                        }
                        crate::WorkspaceJobProgress::Failed(error) => {
                            session.record_source_failure(active.workspace.id, &error)?;
                            state
                                .failed
                                .insert(active.workspace.id.get(), (active.workspace.id, true));
                            state.failure.get_or_insert((active.workspace.id, error));
                            break;
                        }
                        crate::WorkspaceJobProgress::Complete(result) => {
                            let crate::WorkspaceJobResult {
                                unit,
                                library,
                                compiler,
                                replay: traces,
                            } = *result;
                            let workspace = active.workspace;
                            drop(active.job);
                            let unit = std::sync::Arc::try_unwrap(unit).map_err(|_| {
                                jai_vm::Error::InvalidIr(
                                    "completed workspace source still has an active owner",
                                )
                            })?;
                            let source_changed = snapshots(&compiler) != snapshots(session);
                            replay.adopt_completed_fork(traces)?;
                            *session = compiler;
                            if source_changed {
                                restart_source_round(state, session);
                                continue 'rounds;
                            }
                            state.results.insert(
                                workspace.id.get(),
                                ScheduledWorkspace {
                                    id: workspace.id,
                                    name: workspace.name,
                                    output: WorkspaceOutput::Checked {
                                        unit: Box::new(unit),
                                        library,
                                        settings: workspace.settings,
                                    },
                                },
                            );
                            break;
                        }
                    }
                }
            }
            self.check_physical()?;
            let after = snapshots(session);
            self.check_limits(&after)?;
            if state.before == after {
                for (_, wait) in std::mem::take(&mut state.waits) {
                    // The ticket retains the actual request and source span;
                    // every subscribed producer has now reached a fixed point.
                    debug_assert_eq!(
                        wait.pending.diagnostic.location,
                        match &wait.error {
                            Error::Located {
                                source,
                                diagnostic,
                                ..
                            } => jai_source::SourceSpan {
                                source: *source,
                                span: diagnostic.span,
                            },
                            _ => wait.pending.diagnostic.location,
                        }
                    );
                    session.record_source_failure(wait.workspace, &wait.error)?;
                    state
                        .failed
                        .insert(wait.workspace.get(), (wait.workspace, wait.source_parsed));
                    state.failure.get_or_insert((wait.workspace, wait.error));
                }
                // An actual source reset or destroy may deliberately retire a
                // failure. Preserve that existing recovery policy.
                state.failed.retain(|_, (workspace, _)| {
                    session.workspace(*workspace).is_some_and(|workspace| {
                        workspace.status() == jai_vm::WorkspaceStatus::Failed
                    })
                });
                if state
                    .failure
                    .as_ref()
                    .is_some_and(|(workspace, _)| !state.failed.contains_key(&workspace.get()))
                {
                    state.failure = None;
                }
                if state.selection.is_none() {
                    if let Some((workspace, error)) = state.failure.take() {
                        state.failed_workspace = Some(workspace);
                        return Err(error.into());
                    }
                    if let Some(error) = session.error() {
                        state.failed_workspace = session
                            .workspaces()
                            .find(|workspace| workspace.status() == jai_vm::WorkspaceStatus::Failed)
                            .map(|workspace| workspace.id());
                        return Err(Error::CompilerReport(error.clone()).into());
                    }
                }
                return Ok(SchedulerReadiness::Complete(ScheduledBuild {
                    root: session.root(),
                    workspaces: std::mem::take(&mut state.results).into_values().collect(),
                    passes: state.pass,
                }));
            }
            restart_source_round(state, session);
        }
    }
}

fn cancel_frame(children: &ChildFrames, frame: &mut BuildFrame) -> Result<(), jai_vm::Error> {
    let mut error = None;
    if let Some(active) = &mut frame.state.active {
        if let Err(cause) = retire_children(children, active.job.suspended_job_ids()) {
            error.get_or_insert(cause);
        }
        if let Err(cause) = active.job.cancel() {
            error.get_or_insert(cause);
        }
    }
    error.map_or(Ok(()), Err)
}
fn retire_children(
    children: &ChildFrames,
    ids: Vec<crate::CompilerJobId>,
) -> Result<(), jai_vm::Error> {
    let mut error = None;
    for id in ids {
        if let Err(cause) = retire_child(children, id) {
            error.get_or_insert(cause);
        }
    }
    error.map_or(Ok(()), Err)
}
fn retire_child(children: &ChildFrames, id: crate::CompilerJobId) -> Result<(), jai_vm::Error> {
    let child = children.borrow_mut().remove(&id);
    if let Some(mut child) = child {
        cancel_frame(children, &mut child)?;
    }
    Ok(())
}

fn restart_source_round(state: &mut BuildState, session: &CompilerSession) {
    state.pass += 1;
    state.before = snapshots(session);
    state.next = 0;
    state.results.clear();
    state.waits.clear();
}
