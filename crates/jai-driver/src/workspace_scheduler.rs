//! Rebuild actual module graphs from committed compile-time workspace inputs.
use crate::{
    BuildInput, BuildSettings, CompilationUnit, CompilerSession, EffectReplayCache, Error,
    ReplayLimits,
};
use jai_modules::{BootstrapOptions, GraphOptions, SourceOverlay};
use jai_types::BuildTarget;
use jai_vm::{CompilerEvent, CompilerPhase, Limits, SourceOrigin, TargetTriple, WorkspaceId};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt, fs,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Clone, Copy, Debug)]
pub struct SchedulerLimits {
    pub passes: usize,
    pub workspaces: usize,
    pub inputs: usize,
    pub generated_bytes: usize,
}
impl Default for SchedulerLimits {
    fn default() -> Self {
        Self {
            passes: 128,
            workspaces: 256,
            inputs: 10_000,
            generated_bytes: 64 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug)]
pub struct SchedulerOptions {
    pub target: BuildTarget,
    /// Exact backend triple corresponding to `target`; other triples need a new scheduler.
    pub target_triple: TargetTriple,
    pub compile_time_limits: Limits,
    pub limits: SchedulerLimits,
    pub replay_limits: ReplayLimits,
}
pub enum WorkspaceOutput {
    Checked {
        unit: Box<CompilationUnit>,
        library: Box<jai_sema::Library>,
        settings: BuildSettings,
    },
    AwaitingInputs,
}
pub struct ScheduledWorkspace {
    pub id: WorkspaceId,
    pub name: String,
    pub output: WorkspaceOutput,
}
pub struct ScheduledBuild {
    pub root: WorkspaceId,
    pub workspaces: Vec<ScheduledWorkspace>,
    pub passes: usize,
}
#[derive(Clone, Debug)]
pub struct SchedulerPending {
    pub workspace: WorkspaceId,
    pub source: jai_sema::LibraryPending,
}
pub enum SchedulerReadiness {
    Complete(ScheduledBuild),
    Pending(SchedulerPending),
}
impl ScheduledBuild {
    pub fn workspace(&self, id: WorkspaceId) -> Option<&ScheduledWorkspace> {
        self.workspaces.iter().find(|workspace| workspace.id == id)
    }
}
#[derive(Debug)]
pub enum SchedulerError {
    Driver(Error),
    CompilerEffect(jai_vm::Error),
    Limit(&'static str),
    PhysicalSourceChanged(PathBuf),
    VirtualPathCollision(PathBuf),
    UnsupportedTarget {
        workspace: WorkspaceId,
        requested: TargetTriple,
        selected: TargetTriple,
    },
    DifferentSession,
    NonUtf8Path(PathBuf),
    Pending(Box<SchedulerPending>),
    ChangedPendingSession,
}
impl fmt::Display for SchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Driver(error) => error.fmt(formatter),
            Self::CompilerEffect(error) => error.fmt(formatter),
            Self::Limit(name) => write!(formatter, "workspace scheduler {name} limit exceeded"),
            Self::PhysicalSourceChanged(path) => write!(
                formatter,
                "{} changed during this compiler session; start a fresh scheduler and session",
                path.display()
            ),
            Self::VirtualPathCollision(path) => write!(
                formatter,
                "generated source path already exists: {}",
                path.display()
            ),
            Self::UnsupportedTarget {
                workspace,
                requested,
                selected,
            } => write!(
                formatter,
                "workspace {} requests target {}, but this scheduler is configured for {}",
                workspace.get(),
                requested.as_str(),
                selected.as_str()
            ),
            Self::DifferentSession => {
                formatter.write_str("workspace scheduler belongs to a different compiler session")
            }
            Self::NonUtf8Path(path) => write!(
                formatter,
                "source #load path cannot be represented as UTF-8: {}",
                path.display()
            ),
            Self::Pending(pending) => {
                write!(
                    formatter,
                    "workspace {} is suspended",
                    pending.workspace.get()
                )?;
                if let Some(source) = pending.source.source {
                    write!(
                        formatter,
                        " while preparing source at {:?}",
                        source.location()
                    )?;
                }
                write!(formatter, " on {:?}", pending.source.dependencies)
            }
            Self::ChangedPendingSession => formatter.write_str(
                "the committed compiler session changed while a source job was suspended",
            ),
        }
    }
}
impl std::error::Error for SchedulerError {
}
impl From<Error> for SchedulerError {
    fn from(error: Error) -> Self {
        Self::Driver(error)
    }
}
impl From<jai_vm::Error> for SchedulerError {
    fn from(error: jai_vm::Error) -> Self {
        Self::CompilerEffect(error)
    }
}

mod build_frame;
mod child_jobs;
use build_frame::{BuildFrame, BuildState, PreviewSelection};

pub(crate) type ChildJobCanceller =
    Rc<dyn Fn(Vec<crate::CompilerJobId>) -> Result<(), jai_vm::Error>>;
type ChildFrames = Rc<RefCell<HashMap<crate::CompilerJobId, BuildFrame>>>;

/// Owns source snapshots and effect replay for one compiler session.
/// Checked outputs describe semantic compilation only; no native tool is invoked.
pub struct WorkspaceScheduler {
    root: PathBuf,
    root_text: String,
    graph_options: GraphOptions,
    bootstrap: BootstrapOptions,
    options: SchedulerOptions,
    physical: HashMap<PathBuf, Vec<u8>>,
    replay: EffectReplayCache,
    session: Option<WorkspaceId>,
    build: Option<BuildFrame>,
    children: ChildFrames,
}
struct ActiveWorkspaceJob {
    workspace: Snapshot,
    revision: u64,
    job: ActiveSourceJob,
}
struct WorkspaceSources {
    entry: PathBuf,
    options: crate::SemanticDiscoveryOptions,
    provider: SourceOverlay,
    virtual_paths: Vec<PathBuf>,
}
enum ActiveSourceJob {
    Discovery {
        job: crate::PreparedGraphJob,
        virtual_paths: Vec<PathBuf>,
    },
    Binding(crate::PreparedWorkspaceJob),
}
impl ActiveSourceJob {
    fn suspended_job_ids(&self) -> Vec<crate::CompilerJobId> {
        match self {
            Self::Discovery {
                job, ..
            } => job.suspended_job_ids(),
            Self::Binding(job) => job.suspended_job_ids(),
        }
    }

    #[cfg(test)]
    fn suspended_origins(&self) -> Vec<SourceOrigin> {
        match self {
            Self::Discovery {
                job, ..
            } => job.suspended_origins(),
            Self::Binding(job) => job.suspended_origins(),
        }
    }
    fn cancel(&mut self) -> Result<(), jai_vm::Error> {
        match self {
            Self::Discovery {
                job, ..
            } => job.cancel(),
            Self::Binding(job) => job.cancel(),
        }
    }
    fn service_pending(
        &mut self,
        scheduler: &mut WorkspaceScheduler,
    ) -> Result<bool, SchedulerError> {
        match self {
            Self::Discovery {
                job, ..
            } => job.service_pending(scheduler),
            Self::Binding(job) => job.service_pending(scheduler),
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
struct Snapshot {
    id: WorkspaceId,
    name: String,
    inputs: Vec<BuildInput>,
    settings: BuildSettings,
}
fn snapshots(session: &CompilerSession) -> Vec<Snapshot> {
    session
        .workspaces()
        .map(|workspace| Snapshot {
            id: workspace.id(),
            name: workspace.name().into(),
            inputs: workspace.inputs().into(),
            settings: workspace.settings().clone(),
        })
        .collect()
}
fn io(path: &Path, cause: std::io::Error) -> SchedulerError {
    Error::Io {
        path: path.into(),
        cause,
    }
    .into()
}

impl WorkspaceScheduler {
    pub fn new(
        root: &Path,
        graph_options: GraphOptions,
        options: SchedulerOptions,
    ) -> Result<Self, SchedulerError> {
        Self::new_with_bootstrap(root, graph_options, BootstrapOptions::disabled(), options)
    }
    pub fn new_with_bootstrap(
        root: &Path,
        graph_options: GraphOptions,
        bootstrap: BootstrapOptions,
        options: SchedulerOptions,
    ) -> Result<Self, SchedulerError> {
        let root = fs::canonicalize(root).map_err(|cause| io(root, cause))?;
        let bytes = fs::read(&root).map_err(|cause| io(&root, cause))?;
        let root_text = jai_lexer::decode_source(&bytes)
            .map_err(|diagnostic| Error::Decode {
                path: root.clone(),
                diagnostic,
            })?
            .into_owned();
        let replay = EffectReplayCache::new(options.replay_limits);
        Ok(Self {
            root: root.clone(),
            root_text,
            graph_options,
            bootstrap,
            options,
            physical: HashMap::from([(root, bytes)]),
            replay,
            session: None,
            build: None,
            children: Rc::new(RefCell::new(HashMap::new())),
        })
    }
    pub fn replay_cache(&self) -> &EffectReplayCache {
        &self.replay
    }
    pub fn resolve(
        &mut self,
        session: &mut CompilerSession,
    ) -> Result<ScheduledBuild, SchedulerError> {
        match self.resolve_resumable(session)? {
            SchedulerReadiness::Complete(build) => Ok(build),
            SchedulerReadiness::Pending(pending) => Err(SchedulerError::Pending(Box::new(pending))),
        }
    }

    /// Retains a genuinely suspended source job for the next readiness drive.
    pub fn resolve_resumable(
        &mut self,
        session: &mut CompilerSession,
    ) -> Result<SchedulerReadiness, SchedulerError> {
        if self.session.is_some_and(|id| id != session.root()) {
            return Err(SchedulerError::DifferentSession);
        }
        self.session = Some(session.root());
        let mut frame = self.build.take().unwrap_or_else(|| BuildFrame {
            base_revision: session.committed_revision(),
            compiler: session.clone(),
            replay: self.replay.fork_committed(),
            state: BuildState::new(session, None),
        });
        if frame.base_revision != session.committed_revision()
            || session.require_source_idle().is_err()
        {
            self.cancel_frame(&mut frame)?;
            return Err(SchedulerError::ChangedPendingSession);
        }
        let result = self.drive_build(&mut frame.compiler, &mut frame.replay, &mut frame.state);
        match result {
            Ok(SchedulerReadiness::Pending(pending)) => {
                self.build = Some(frame);
                Ok(SchedulerReadiness::Pending(pending))
            }
            Ok(SchedulerReadiness::Complete(build)) => {
                *session = frame.compiler;
                self.replay = frame.replay;
                Ok(SchedulerReadiness::Complete(build))
            }
            Err(error) => {
                self.cancel_frame(&mut frame)?;
                match error {
                    SchedulerError::Driver(error) => {
                        let workspace = frame
                            .state
                            .failed_workspace
                            .filter(|id| session.workspace(*id).is_some())
                            .unwrap_or_else(|| session.root());
                        Err(SchedulerError::Driver(
                            crate::source_discovery::record_failure(session, workspace, error),
                        ))
                    }
                    error => Err(error),
                }
            }
        }
    }

    pub fn cancel_pending(&mut self) -> Result<(), jai_vm::Error> {
        let mut error = None;
        if let Some(mut frame) = self.build.take()
            && let Err(cause) = self.cancel_frame(&mut frame)
        {
            error.get_or_insert(cause);
        }
        loop {
            let id = self.children.borrow().keys().next().copied();
            let Some(id) = id else {
                break;
            };
            if let Err(cause) = self.retire_child(id) {
                error.get_or_insert(cause);
            }
        }
        error.map_or(Ok(()), Err)
    }

    fn check_limits(&self, workspaces: &[Snapshot]) -> Result<(), SchedulerError> {
        if workspaces.len() > self.options.limits.workspaces {
            return Err(SchedulerError::Limit("workspaces"));
        }
        let inputs = workspaces.iter().try_fold(0usize, |count, workspace| {
            count.checked_add(workspace.inputs.len())
        });
        if inputs.is_none_or(|count| count > self.options.limits.inputs) {
            return Err(SchedulerError::Limit("inputs"));
        }
        let bytes = workspaces
            .iter()
            .flat_map(|workspace| &workspace.inputs)
            .try_fold(0usize, |count, input| {
                count.checked_add(match input {
                    BuildInput::Source(source)
                    | BuildInput::SourceAt {
                        source, ..
                    } => source.len(),
                    _ => 0,
                })
            });
        if bytes.is_none_or(|count| count > self.options.limits.generated_bytes) {
            return Err(SchedulerError::Limit("generated bytes"));
        }
        for workspace in workspaces {
            if let Some(requested) = &workspace.settings.target
                && requested != &self.options.target_triple
            {
                return Err(SchedulerError::UnsupportedTarget {
                    workspace: workspace.id,
                    requested: requested.clone(),
                    selected: self.options.target_triple.clone(),
                });
            }
        }
        Ok(())
    }
    fn check_physical(&self) -> Result<(), SchedulerError> {
        for (path, expected) in &self.physical {
            let actual = fs::read(path).map_err(|cause| io(path, cause))?;
            if actual != *expected {
                return Err(SchedulerError::PhysicalSourceChanged(path.clone()));
            }
        }
        Ok(())
    }
    fn source_snapshot(
        &self,
        workspace: &Snapshot,
        session: &CompilerSession,
    ) -> Result<WorkspaceSources, SchedulerError> {
        let root = session.root();
        let base = self
            .root
            .parent()
            .expect("canonical source has a parent")
            .to_owned();
        let mut overlay = SourceOverlay::new();
        let mut virtual_paths = vec![];
        let entry = if workspace.id == root {
            self.root.clone()
        } else {
            base.join(format!(
                ".jai-workspace-{}-{}.jai",
                root.get(),
                workspace.id.get()
            ))
        };
        if workspace.id != root {
            self.check_virtual(&entry)?;
        }
        let mut source = if workspace.id == root {
            self.root_text.clone()
        } else {
            String::new()
        };
        // A trailing line comment in the physical source must not swallow loads.
        source.push('\n');
        for (index, input) in workspace.inputs.iter().enumerate() {
            let path = match input {
                BuildInput::Source(text)
                | BuildInput::SourceAt {
                    source: text, ..
                } => {
                    let path = base.join(format!(
                        ".jai-generated-{}-{}-{index}.jai",
                        root.get(),
                        workspace.id.get()
                    ));
                    self.check_virtual(&path)?;
                    let path = overlay
                        .insert(&path, text.as_bytes().into())
                        .map_err(|cause| io(&path, cause))?;
                    virtual_paths.push(path.clone());
                    path
                }
                BuildInput::File(path) => {
                    if path.is_absolute() {
                        path.clone()
                    } else {
                        base.join(path)
                    }
                }
                BuildInput::FileAt {
                    path,
                    location,
                } => {
                    if path.is_absolute() {
                        path.clone()
                    } else {
                        let origin = if location.path.is_absolute() {
                            location.path.clone()
                        } else {
                            base.join(&location.path)
                        };
                        origin.parent().unwrap_or(&base).join(path)
                    }
                }
            };
            append_load(&mut source, &path)?;
        }
        let entry = overlay
            .insert(&entry, source.into_bytes())
            .map_err(|cause| io(&entry, cause))?;
        virtual_paths.push(entry.clone());
        let mut bootstrap = self.bootstrap.clone();
        if let Some(runtime) = &mut bootstrap.runtime_support {
            runtime.parameters = workspace.settings.runtime_support_parameters();
        }
        let discovery_options = crate::SemanticDiscoveryOptions {
            graph: self.graph_options.clone(),
            bootstrap,
            target: self.options.target.clone(),
            workspace: workspace.id,
            limits: self.options.compile_time_limits,
            effect_policy: crate::DiscoveryEffectPolicy::CompilerSession,
        };
        Ok(WorkspaceSources {
            entry,
            options: discovery_options,
            provider: overlay,
            virtual_paths,
        })
    }
    fn retain_physical(
        &mut self,
        graph: &jai_modules::ModuleGraph,
        virtual_paths: &[PathBuf],
    ) -> Result<(), SchedulerError> {
        for record in graph.sources().records() {
            if virtual_paths.contains(&record.path().to_owned()) {
                continue;
            }
            let path = record.path();
            let bytes = fs::read(path).map_err(|cause| io(path, cause))?;
            let text = jai_lexer::decode_source(&bytes).map_err(|diagnostic| Error::Decode {
                path: path.into(),
                diagnostic,
            })?;
            if text.as_ref() != record.text()
                || self.physical.get(path).is_some_and(|old| *old != bytes)
            {
                return Err(SchedulerError::PhysicalSourceChanged(path.into()));
            }
            self.physical.entry(path.into()).or_insert(bytes);
        }
        Ok(())
    }
    fn check_virtual(&self, path: &Path) -> Result<(), SchedulerError> {
        if path.try_exists().map_err(|cause| io(path, cause))? {
            Err(SchedulerError::VirtualPathCollision(path.into()))
        } else {
            Ok(())
        }
    }
}
impl Drop for WorkspaceScheduler {
    fn drop(&mut self) {
        let _ = self.cancel_pending();
    }
}

fn append_load(source: &mut String, path: &Path) -> Result<(), SchedulerError> {
    let path = path
        .to_str()
        .ok_or_else(|| SchedulerError::NonUtf8Path(path.into()))?;
    source.push_str("#load \"");
    for character in path.chars() {
        match character {
            '\\' => source.push_str("\\\\"),
            '"' => source.push_str("\\\""),
            '\n' => source.push_str("\\n"),
            '\r' => source.push_str("\\r"),
            '\t' => source.push_str("\\t"),
            other => source.push(other),
        }
    }
    source.push_str("\";\n");
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "workspace_scheduler/continuation_tests.rs"]
mod continuation_tests;

#[cfg(test)]
#[path = "workspace_scheduler/build_frame_tests.rs"]
mod build_frame_tests;
