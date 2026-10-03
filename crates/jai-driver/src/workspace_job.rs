//! Keep an immutable compilation unit and its borrowed preparation alive together.
use crate::{CompilationUnit, CompilerSession, EffectReplayCache, Error, ReplayEffects};
use jai_sema::{LibraryPending, LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use jai_source::LocatedDiagnostic;
use std::{
    cell::RefCell,
    future::{Future, poll_fn},
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

pub struct WorkspaceJobResult {
    pub unit: Arc<CompilationUnit>,
    pub library: Box<jai_sema::Library>,
    pub compiler: CompilerSession,
    pub replay: EffectReplayCache,
}

/// Completed source-prefix journals for a genuine input/configuration rebuild.
pub struct WorkspaceSourceRebuild {
    pub unit: Arc<CompilationUnit>,
    pub compiler: CompilerSession,
    pub replay: EffectReplayCache,
}

pub enum WorkspaceJobProgress {
    SourceRebuild(Box<WorkspaceSourceRebuild>),
    Complete(Box<WorkspaceJobResult>),
    Pending(LibraryPending),
    SourceWait(LibraryPending),
    Failed(Error),
}

struct Journal {
    compiler: CompilerSession,
    replay: EffectReplayCache,
}

struct JobState {
    journal: Option<Journal>,
    pending: Option<LibraryPending>,
    cancellation_error: Option<jai_vm::Error>,
    child_canceller: Option<crate::workspace_scheduler::ChildJobCanceller>,
}

enum WorkspaceJobFailure {
    Hard(LocatedDiagnostic),
    Source(LibraryPending),
}
impl From<LocatedDiagnostic> for WorkspaceJobFailure {
    fn from(error: LocatedDiagnostic) -> Self {
        Self::Hard(error)
    }
}
enum WorkspaceJobOutcome {
    Library(Box<jai_sema::Library>),
    SourceRebuild,
}
type JobFuture = Pin<Box<dyn Future<Output = Result<WorkspaceJobOutcome, WorkspaceJobFailure>>>>;
mod source_prefix;

/// Owns a source snapshot, private journals, and a compiler-pinned preparation.
/// Dropping a pending job synchronously cancels its exact VM continuation.
pub struct PreparedWorkspaceJob {
    unit: Arc<CompilationUnit>,
    state: Rc<RefCell<JobState>>,
    future: Option<JobFuture>,
}

impl PreparedWorkspaceJob {
    pub fn new(
        unit: CompilationUnit,
        options: ResolveOptions,
        compiler: CompilerSession,
        replay: EffectReplayCache,
    ) -> Self {
        let baseline = source_prefix::source_configuration(&compiler);
        Self::with_configuration(unit, options, compiler, replay, baseline)
    }

    pub(crate) fn with_source_baseline(
        unit: CompilationUnit,
        options: ResolveOptions,
        compiler: CompilerSession,
        replay: EffectReplayCache,
        baseline: &CompilerSession,
    ) -> Self {
        let baseline = source_prefix::source_configuration(baseline);
        Self::with_configuration(unit, options, compiler, replay, baseline)
    }

    fn with_configuration(
        unit: CompilationUnit,
        options: ResolveOptions,
        compiler: CompilerSession,
        replay: EffectReplayCache,
        baseline: source_prefix::SourceConfiguration,
    ) -> Self {
        let unit = Arc::new(unit);
        let state = Rc::new(RefCell::new(JobState {
            journal: Some(Journal {
                compiler,
                replay,
            }),
            pending: None,
            cancellation_error: None,
            child_canceller: None,
        }));
        let future_unit = Arc::clone(&unit);
        let future_state = Rc::clone(&state);
        let future = Box::pin(async move {
            // Rust's pinned future retains these locals and their borrows across
            // await. The controller never constructs a self-referential struct.
            let mut session = PreparedLibrarySession::new(future_unit.graph(), &options)?;
            let mut guard = CancellationGuard {
                session: &mut session,
                state: future_state,
            };
            if source_prefix::drive(&mut guard, &baseline).await? {
                return Ok(WorkspaceJobOutcome::SourceRebuild);
            }
            poll_fn(|_| {
                let mut state = guard.state.borrow_mut();
                let progress = {
                    let Journal {
                        compiler,
                        replay,
                    } = state
                        .journal
                        .as_mut()
                        .expect("running job retains private journals");
                    guard
                        .session
                        .drive(&mut ReplayEffects::new(compiler, replay))
                };
                match progress {
                    LibraryReadiness::Complete(library) => {
                        state.pending = None;
                        Poll::Ready(Ok(WorkspaceJobOutcome::Library(library)))
                    }
                    LibraryReadiness::Failed(error) => {
                        state.pending = None;
                        Poll::Ready(Err(WorkspaceJobFailure::Hard(error)))
                    }
                    LibraryReadiness::Pending(pending)
                        if crate::discovery_worklists::source_only_wait(&pending) =>
                    {
                        // This immutable graph has no remaining structural
                        // producer. Completed source runs retain their journals
                        // so new committed inputs can start the next graph round.
                        state.pending = None;
                        Poll::Ready(Err(WorkspaceJobFailure::Source(pending)))
                    }
                    LibraryReadiness::Pending(pending) if pending.dependencies.is_empty() => {
                        state.pending = None;
                        Poll::Ready(Err(WorkspaceJobFailure::Hard(pending.diagnostic)))
                    }
                    LibraryReadiness::Pending(pending) => {
                        state.pending = Some(pending);
                        Poll::Pending
                    }
                }
            })
            .await
        });
        Self {
            unit,
            state,
            future: Some(future),
        }
    }

    pub fn unit(&self) -> &CompilationUnit {
        &self.unit
    }

    /// Polls the same pinned source job. Readiness is serviced by its controller.
    pub fn poll(&mut self) -> Result<WorkspaceJobProgress, jai_vm::Error> {
        let parked = self.suspended_job_ids();
        let future = self.future.as_mut().ok_or_else(|| {
            jai_vm::Error::EffectRejected("workspace job has already terminated".into())
        })?;
        let waker = Waker::from(Arc::new(JobWake));
        let result = future.as_mut().poll(&mut Context::from_waker(&waker));
        let retained = self.suspended_job_ids();
        let retired = parked
            .into_iter()
            .filter(|id| !retained.contains(id))
            .collect();
        if let Err(error) = self.cancel_child_ids(retired) {
            self.future = None;
            self.state
                .borrow_mut()
                .cancellation_error
                .get_or_insert(error.clone());
            return Err(error);
        }
        match result {
            Poll::Pending => Ok(WorkspaceJobProgress::Pending(
                self.state
                    .borrow()
                    .pending
                    .as_ref()
                    .expect("suspended future publishes typed readiness")
                    .clone(),
            )),
            Poll::Ready(result) => {
                // Drop the cancellation guard before releasing its journals.
                self.future = None;
                let mut state = self.state.borrow_mut();
                if let Some(error) = state.cancellation_error.take() {
                    return Err(error);
                }
                Ok(match result {
                    Ok(WorkspaceJobOutcome::SourceRebuild) => {
                        let Journal {
                            compiler,
                            replay,
                        } = state
                            .journal
                            .take()
                            .expect("completed source prefix retains its private journals");
                        WorkspaceJobProgress::SourceRebuild(Box::new(WorkspaceSourceRebuild {
                            unit: Arc::clone(&self.unit),
                            compiler,
                            replay,
                        }))
                    }
                    Ok(WorkspaceJobOutcome::Library(library)) => {
                        let Journal {
                            compiler,
                            replay,
                        } = state
                            .journal
                            .take()
                            .expect("terminated job retains private journals");
                        WorkspaceJobProgress::Complete(Box::new(WorkspaceJobResult {
                            unit: Arc::clone(&self.unit),
                            library,
                            compiler,
                            replay,
                        }))
                    }
                    Err(WorkspaceJobFailure::Source(pending)) => {
                        WorkspaceJobProgress::SourceWait(pending)
                    }
                    Err(WorkspaceJobFailure::Hard(error)) => {
                        state.journal = None;
                        WorkspaceJobProgress::Failed(self.unit.located(error))
                    }
                })
            }
        }
    }

    fn cancel_child_ids(&self, ids: Vec<crate::CompilerJobId>) -> Result<(), jai_vm::Error> {
        let canceller = self.state.borrow().child_canceller.clone();
        canceller.map_or(Ok(()), |cancel| cancel(ids))
    }

    fn cancel_children(&self) -> Result<(), jai_vm::Error> {
        self.cancel_child_ids(self.suspended_job_ids())
    }

    pub fn cancel(&mut self) -> Result<(), jai_vm::Error> {
        let child_error = self.cancel_children().err();
        self.future = None;
        let mut state = self.state.borrow_mut();
        state.pending = None;
        state.journal = None;
        state
            .cancellation_error
            .take()
            .or(child_error)
            .map_or(Ok(()), Err)
    }

    pub(crate) fn suspended_job_ids(&self) -> Vec<crate::CompilerJobId> {
        self.state
            .borrow()
            .journal
            .as_ref()
            .map(|journal| {
                journal
                    .replay
                    .suspended_origins()
                    .filter_map(|origin| journal.replay.suspended_job(origin))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn suspended_origins(&self) -> Vec<jai_vm::SourceOrigin> {
        self.state
            .borrow()
            .journal
            .as_ref()
            .map(|journal| journal.replay.suspended_origins().cloned().collect())
            .unwrap_or_default()
    }

    /// Services only the compiler requests issued by this retained source job.
    pub fn service_pending(
        &mut self,
        scheduler: &mut crate::WorkspaceScheduler,
    ) -> Result<bool, crate::SchedulerError> {
        self.state.borrow_mut().child_canceller = Some(scheduler.child_canceller());
        let mut progress = false;
        for origin in self.suspended_origins() {
            progress |= self.with_journals(|compiler, replay| {
                scheduler.advance_suspended(compiler, replay, &origin)
            })??;
        }
        Ok(progress)
    }

    pub(crate) fn with_journals<T>(
        &mut self,
        action: impl FnOnce(&mut CompilerSession, &mut EffectReplayCache) -> T,
    ) -> Result<T, jai_vm::Error> {
        let mut state = self.state.borrow_mut();
        let Journal {
            compiler,
            replay,
        } = state.journal.as_mut().ok_or_else(|| {
            jai_vm::Error::EffectRejected("workspace job has already terminated".into())
        })?;
        Ok(action(compiler, replay))
    }

    pub(crate) fn take_terminal_journals(
        &mut self,
    ) -> Option<(CompilerSession, EffectReplayCache)> {
        if self.future.is_some() {
            return None;
        }
        self.state.borrow_mut().journal.take().map(
            |Journal {
                 compiler,
                 replay,
             }| (compiler, replay),
        )
    }
}

impl Drop for PreparedWorkspaceJob {
    fn drop(&mut self) {
        if let Err(error) = self.cancel_children() {
            self.state
                .borrow_mut()
                .cancellation_error
                .get_or_insert(error);
        }
        self.future = None;
    }
}

struct CancellationGuard<'session, 'graph> {
    session: &'session mut PreparedLibrarySession<'graph>,
    state: Rc<RefCell<JobState>>,
}

impl Drop for CancellationGuard<'_, '_> {
    fn drop(&mut self) {
        let mut state = self.state.borrow_mut();
        if let Some(Journal {
            compiler,
            replay,
        }) = state.journal.as_mut()
            && let Err(error) = self
                .session
                .cancel(&mut ReplayEffects::new(compiler, replay))
        {
            state.cancellation_error.get_or_insert(error);
        }
    }
}

struct JobWake;
impl Wake for JobWake {
    fn wake(self: Arc<Self>) {
    }
}

#[cfg(test)]
#[path = "workspace_job/tests.rs"]
mod tests;
