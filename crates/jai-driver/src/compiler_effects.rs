//! Atomic compiler-input changes requested by compile-time execution.
mod driver_failures;
use jai_types::{BacktraceOnCrash, BuildOutputKind, RuntimeSupportMode};
use jai_vm::{
    BitcodeOptimization, BuildOption, BuildOptionsSnapshot, CompilerEffects, CompilerOutputStream,
    CompilerRequest, CompilerResponse, EffectOutcome, MachineOptimization, MessageLevel,
    ReportContinuation, SourceLocation, SourceOrigin, TargetTriple, WorkspaceId, WorkspaceStatus,
};
use jai_vm::{CompilerEvent, CompilerPhase, EffectKey, InterceptFlags};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn allocate_workspace() -> Option<WorkspaceId> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let mut next = NEXT.load(Ordering::Relaxed);
    loop {
        if next > i64::MAX as u64 {
            return None;
        }
        match NEXT.compare_exchange_weak(next, next + 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(id) => return WorkspaceId::from_raw(id),
            Err(current) => next = current,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildInput {
    Source(String),
    File(PathBuf),
    SourceAt {
        source: String,
        location: SourceLocation,
    },
    FileAt {
        path: PathBuf,
        location: SourceLocation,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildSettings {
    pub bitcode: BitcodeOptimization,
    pub machine: MachineOptimization,
    pub output_path: Option<PathBuf>,
    pub target: Option<TargetTriple>,
    pub output_kind: BuildOutputKind,
    pub runtime_support: RuntimeSupportMode,
    pub backtrace_on_crash: BacktraceOnCrash,
    pub temporary_storage_size: i32,
}
impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            bitcode: BitcodeOptimization::Unset,
            machine: MachineOptimization::Unset,
            output_path: None,
            target: None,
            output_kind: BuildOutputKind::default(),
            runtime_support: RuntimeSupportMode::default(),
            backtrace_on_crash: BacktraceOnCrash::default(),
            temporary_storage_size: 32768,
        }
    }
}
impl BuildSettings {
    fn apply(&mut self, option: BuildOption) {
        match option {
            BuildOption::BitcodeOptimization(value) => self.bitcode = value,
            BuildOption::MachineOptimization(value) => self.machine = value,
            BuildOption::Optimize(value) => {
                self.bitcode = if value {
                    BitcodeOptimization::O2
                } else {
                    BitcodeOptimization::O0
                };
                self.machine = if value {
                    MachineOptimization::Default
                } else {
                    MachineOptimization::None
                };
            }
            BuildOption::OutputPath(value) => self.output_path = Some(value),
            BuildOption::Target(value) => self.target = Some(value),
            BuildOption::OutputKind(value) => self.output_kind = value,
            BuildOption::RuntimeSupport(value) => self.runtime_support = value,
            BuildOption::BacktraceOnCrash(value) => self.backtrace_on_crash = value,
            BuildOption::TemporaryStorageSize(value) => self.temporary_storage_size = value,
        }
    }
}

/// Individual settings whose latest source origin can be queried after commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuildSetting {
    BitcodeOptimization,
    MachineOptimization,
    OutputPath,
    Target,
    OutputKind,
    RuntimeSupport,
    BacktraceOnCrash,
    TemporaryStorageSize,
}
fn setting_keys(option: &BuildOption) -> &'static [BuildSetting] {
    use BuildSetting::*;
    match option {
        BuildOption::Optimize(_) => &[BitcodeOptimization, MachineOptimization],
        BuildOption::BitcodeOptimization(_) => &[BitcodeOptimization],
        BuildOption::MachineOptimization(_) => &[MachineOptimization],
        BuildOption::OutputPath(_) => &[OutputPath],
        BuildOption::Target(_) => &[Target],
        BuildOption::OutputKind(_) => &[OutputKind],
        BuildOption::RuntimeSupport(_) => &[RuntimeSupport],
        BuildOption::BacktraceOnCrash(_) => &[BacktraceOnCrash],
        BuildOption::TemporaryStorageSize(_) => &[TemporaryStorageSize],
    }
}

#[derive(Clone, Debug)]
pub struct BuildWorkspace {
    id: WorkspaceId,
    name: String,
    inputs: Vec<BuildInput>,
    settings: BuildSettings,
    option_origins: BTreeMap<BuildSetting, SourceLocation>,
    status: WorkspaceStatus,
    error: Option<CompilerMessage>,
}
impl BuildWorkspace {
    pub fn id(&self) -> WorkspaceId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn inputs(&self) -> &[BuildInput] {
        &self.inputs
    }
    pub fn option_origin(&self, setting: BuildSetting) -> Option<&SourceLocation> {
        self.option_origins.get(&setting)
    }
    pub fn settings(&self) -> &BuildSettings {
        &self.settings
    }
    pub fn status(&self) -> WorkspaceStatus {
        self.status
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerMessage {
    pub level: MessageLevel,
    pub text: String,
    pub location: Option<SourceLocation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerOutput {
    pub stream: CompilerOutputStream,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
enum Change {
    Destroy(WorkspaceId),
    Input(WorkspaceId, BuildInput),
    Option(WorkspaceId, BuildOption, Option<SourceLocation>),
    Message(WorkspaceId, CompilerMessage),
    Status(WorkspaceId, WorkspaceStatus),
    Output(CompilerOutput),
}

#[cfg(test)]
mod workspace_status_tests {
    use super::*;

    fn status(session: &mut CompilerSession, workspace: WorkspaceId, value: WorkspaceStatus) {
        assert_eq!(
            session.request(CompilerRequest::SetWorkspaceStatus {
                workspace,
                status: value,
            }),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
    }

    fn child(session: &mut CompilerSession) -> WorkspaceId {
        session.begin();
        let EffectOutcome::Ready(CompilerResponse::Workspace(id)) =
            session.request(CompilerRequest::CreateWorkspace {
                name: "child".into(),
            })
        else {
            panic!("workspace creation failed")
        };
        session.finish(true).unwrap();
        id
    }

    fn origin(workspace: WorkspaceId) -> SourceOrigin {
        SourceOrigin {
            workspace,
            path: "/build/main.jai".into(),
            start: 0,
            end: 1,
            body_hash: 1,
            body: vec![0],
            specialization: vec![],
        }
    }

    #[test]
    fn recovery_is_per_workspace_and_survives_message_draining() {
        let mut session = CompilerSession::new();
        let root = session.root();
        let child = child(&mut session);
        session.begin();
        status(&mut session, root, WorkspaceStatus::Failed);
        status(&mut session, child, WorkspaceStatus::Failed);
        session.finish(true).unwrap();
        session.take_messages();
        session.begin();
        status(&mut session, root, WorkspaceStatus::Ok);
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().status(),
            WorkspaceStatus::Ok
        );
        assert_eq!(
            session.workspace(child).unwrap().status(),
            WorkspaceStatus::Failed
        );
        assert!(session.error().is_some());
        session.begin();
        status(&mut session, child, WorkspaceStatus::Ok);
        session.finish(true).unwrap();
        assert!(session.error().is_none());
    }

    #[test]
    fn status_is_atomic_and_request_order_controls_recovery() {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        status(&mut session, root, WorkspaceStatus::Failed);
        assert_eq!(
            session.workspace(root).unwrap().status(),
            WorkspaceStatus::Ok
        );
        session.finish(false).unwrap();
        assert!(session.error().is_none());
        session.begin();
        assert!(matches!(
            session.request(CompilerRequest::Message {
                level: MessageLevel::Error,
                text: "recoverable".into(),
            }),
            EffectOutcome::Ready(_)
        ));
        status(&mut session, root, WorkspaceStatus::Ok);
        session.finish(true).unwrap();
        assert!(session.error().is_none());
        assert_eq!(session.take_messages()[0].text, "recoverable");
        session.begin();
        status(&mut session, root, WorkspaceStatus::Ok);
        assert!(matches!(
            session.request(CompilerRequest::Message {
                level: MessageLevel::Error,
                text: "new error".into(),
            }),
            EffectOutcome::Ready(_)
        ));
        session.finish(true).unwrap();
        assert_eq!(session.error().unwrap().text, "new error");
    }

    #[test]
    fn originated_reports_fail_the_actual_workspace_and_origin_is_consumed() {
        let mut session = CompilerSession::new();
        let root = session.root();
        let child = child(&mut session);
        session.set_source_origin(origin(child));
        session.begin();
        assert!(matches!(
            session.request(CompilerRequest::Report {
                level: MessageLevel::Error,
                continuation: ReportContinuation::Continue,
                location: SourceLocation {
                    path: "/build/main.jai".into(),
                    line: 1,
                    column: 1
                },
                text: "child error".into(),
            }),
            EffectOutcome::Ready(_)
        ));
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().status(),
            WorkspaceStatus::Ok
        );
        assert_eq!(
            session.workspace(child).unwrap().status(),
            WorkspaceStatus::Failed
        );
        session.begin();
        status(&mut session, child, WorkspaceStatus::Ok);
        assert!(matches!(
            session.request(CompilerRequest::Message {
                level: MessageLevel::Error,
                text: "root error".into(),
            }),
            EffectOutcome::Ready(_)
        ));
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().status(),
            WorkspaceStatus::Failed
        );
        assert_eq!(
            session.workspace(child).unwrap().status(),
            WorkspaceStatus::Ok
        );
        assert_eq!(session.error().unwrap().text, "root error");
    }

    #[test]
    fn foreign_status_handles_and_origins_reject_without_publishing() {
        let mut session = CompilerSession::new();
        let foreign = CompilerSession::new().root();
        let root = session.root();
        session.begin();
        status(&mut session, root, WorkspaceStatus::Failed);
        assert!(matches!(
            session.request(CompilerRequest::SetWorkspaceStatus {
                workspace: foreign,
                status: WorkspaceStatus::Ok,
            }),
            EffectOutcome::Rejected(_)
        ));
        assert!(session.finish(true).is_err());
        assert_eq!(
            session.workspace(root).unwrap().status(),
            WorkspaceStatus::Ok
        );
        session.set_source_origin(origin(foreign));
        session.begin();
        assert!(matches!(
            session.request(CompilerRequest::Message {
                level: MessageLevel::Info,
                text: "discard".into(),
            }),
            EffectOutcome::Rejected(_)
        ));
        session.finish(false).unwrap();
        assert!(session.take_messages().is_empty());
    }
}
#[derive(Clone)]
struct FatalReport {
    location: SourceLocation,
    message: String,
}
#[derive(Clone)]
struct Transaction {
    created: BTreeMap<WorkspaceId, BuildWorkspace>,
    destroyed: BTreeSet<WorkspaceId>,
    changes: Vec<Change>,
    fatal: Option<FatalReport>,
    output_bytes: usize,
    source_workspace: WorkspaceId,
    prepared: Option<PreparedCompilerState>,
    job: Option<CompilerJobId>,
    interception: Interception,
}
impl Transaction {
    fn new(source_workspace: WorkspaceId) -> Self {
        Self {
            created: BTreeMap::new(),
            destroyed: BTreeSet::new(),
            changes: vec![],
            fatal: None,
            output_bytes: 0,
            source_workspace,
            prepared: None,
            job: None,
            interception: Interception::default(),
        }
    }
}
#[derive(Clone, Default)]
enum TransactionState {
    #[default]
    Idle,
    Active(Box<Transaction>),
    Rejected,
}

/// Committed inputs are plans for the driver, not completed compilations.
/// Clones preserve this logical session's identity for atomic host commit shadows.
#[derive(Clone)]
pub struct CompilerSession {
    root: WorkspaceId,
    workspaces: BTreeMap<WorkspaceId, BuildWorkspace>,
    destroyed: BTreeSet<WorkspaceId>,
    messages: Vec<CompilerMessage>,
    error: Option<CompilerMessage>,
    transaction: TransactionState,
    outputs: Vec<CompilerOutput>,
    output_bytes: usize,
    output_limit: usize,
    pending_workspace: Option<WorkspaceId>,
    revision: u64,
    preview_job: Option<CompilerJobId>,
}
impl Default for CompilerSession {
    fn default() -> Self {
        Self::new()
    }
}
impl CompilerSession {
    pub(crate) fn begin_from_workspace(&mut self, workspace: WorkspaceId) {
        self.pending_workspace = Some(workspace);
        self.begin();
    }
    pub fn new() -> Self {
        Self::with_output_limit(16 * 1024 * 1024)
    }
    pub fn with_output_limit(output_limit: usize) -> Self {
        let root = allocate_workspace().expect("workspace identity capacity exhausted");
        let mut workspaces = BTreeMap::new();
        workspaces.insert(
            root,
            BuildWorkspace {
                id: root,
                name: String::new(),
                inputs: vec![],
                settings: BuildSettings::default(),
                option_origins: BTreeMap::new(),
                status: WorkspaceStatus::Ok,
                error: None,
            },
        );
        Self {
            root,
            workspaces,
            destroyed: BTreeSet::new(),
            messages: vec![],
            error: None,
            transaction: TransactionState::Idle,
            outputs: vec![],
            output_bytes: 0,
            output_limit,
            pending_workspace: None,
            revision: 0,
            preview_job: None,
        }
    }
    pub fn root(&self) -> WorkspaceId {
        self.root
    }
    pub fn workspace(&self, id: WorkspaceId) -> Option<&BuildWorkspace> {
        self.workspaces.get(&id)
    }
    /// A retired identity remains invalid for the lifetime of this session.
    pub fn is_destroyed(&self, id: WorkspaceId) -> bool {
        self.destroyed.contains(&id)
    }
    pub fn workspaces(&self) -> impl ExactSizeIterator<Item = &BuildWorkspace> {
        self.workspaces.values()
    }
    pub fn take_messages(&mut self) -> Vec<CompilerMessage> {
        if !self.messages.is_empty() {
            self.revision = self.revision.wrapping_add(1);
        }
        std::mem::take(&mut self.messages)
    }
    pub fn take_outputs(&mut self) -> Vec<CompilerOutput> {
        if !self.outputs.is_empty() {
            self.revision = self.revision.wrapping_add(1);
        }
        self.output_bytes = 0;
        std::mem::take(&mut self.outputs)
    }
    pub(crate) fn committed_revision(&self) -> u64 {
        self.revision
    }
    pub fn error(&self) -> Option<&CompilerMessage> {
        self.error.as_ref()
    }
    fn reject(&mut self, message: impl Into<String>) -> EffectOutcome {
        self.transaction = TransactionState::Rejected;
        EffectOutcome::Rejected(message.into())
    }
    fn stage(&mut self, workspace: WorkspaceId, change: Change) -> EffectOutcome {
        let TransactionState::Active(transaction) = &mut self.transaction else {
            return self.reject("compiler effects require an active transaction");
        };
        let (workspaces, destroyed) = transaction
            .prepared
            .as_ref()
            .map_or((&self.workspaces, &self.destroyed), |state| {
                (&state.workspaces, &state.destroyed)
            });
        if destroyed.contains(&workspace) || transaction.destroyed.contains(&workspace) {
            return self.reject("compiler workspace was destroyed");
        }
        if !workspaces.contains_key(&workspace) && !transaction.created.contains_key(&workspace) {
            return self.reject("unknown compiler workspace");
        }
        transaction.changes.push(change);
        EffectOutcome::Ready(CompilerResponse::Unit)
    }
    fn destroy(&mut self, workspace: WorkspaceId) -> EffectOutcome {
        let result = self.stage(workspace, Change::Destroy(workspace));
        if matches!(result, EffectOutcome::Ready(CompilerResponse::Unit)) {
            let TransactionState::Active(transaction) = &mut self.transaction else {
                unreachable!()
            };
            transaction.destroyed.insert(workspace);
        }
        result
    }
    fn get_options(&mut self, id: WorkspaceId) -> EffectOutcome {
        let TransactionState::Active(transaction) = &self.transaction else {
            return self.reject("compiler effects require an active transaction");
        };
        let (workspaces, destroyed) = transaction
            .prepared
            .as_ref()
            .map_or((&self.workspaces, &self.destroyed), |state| {
                (&state.workspaces, &state.destroyed)
            });
        if destroyed.contains(&id) || transaction.destroyed.contains(&id) {
            return self.reject("compiler workspace was destroyed");
        }
        let workspace = transaction.created.get(&id).or_else(|| workspaces.get(&id));
        let Some(workspace) = workspace else {
            return self.reject("unknown compiler workspace");
        };
        let mut settings = workspace.settings.clone();
        for change in &transaction.changes {
            if let Change::Option(workspace, option, _) = change
                && *workspace == id
            {
                settings.apply(option.clone());
            }
        }
        EffectOutcome::Ready(CompilerResponse::BuildOptions(BuildOptionsSnapshot {
            output_path: settings.output_path,
            target: settings.target,
            bitcode: settings.bitcode,
            machine: settings.machine,
            output_kind: settings.output_kind,
            runtime_support: settings.runtime_support,
            backtrace_on_crash: settings.backtrace_on_crash,
            temporary_storage_size: settings.temporary_storage_size,
        }))
    }
    fn get_name(&mut self, id: WorkspaceId) -> EffectOutcome {
        let TransactionState::Active(transaction) = &self.transaction else {
            return self.reject("compiler effects require an active transaction");
        };
        let (workspaces, destroyed) = transaction
            .prepared
            .as_ref()
            .map_or((&self.workspaces, &self.destroyed), |state| {
                (&state.workspaces, &state.destroyed)
            });
        if destroyed.contains(&id) || transaction.destroyed.contains(&id) {
            return self.reject("compiler workspace was destroyed");
        }
        match transaction.created.get(&id).or_else(|| workspaces.get(&id)) {
            Some(workspace) => {
                EffectOutcome::Ready(CompilerResponse::WorkspaceName(workspace.name.clone()))
            }
            None => self.reject("unknown compiler workspace"),
        }
    }
}
impl CompilerEffects for CompilerSession {
    fn set_source_origin(&mut self, origin: SourceOrigin) {
        self.pending_workspace = Some(origin.workspace);
    }
    fn begin(&mut self) {
        let workspace = self.pending_workspace.take().unwrap_or(self.root);
        self.transaction = if self.workspaces.contains_key(&workspace) {
            TransactionState::Active(Box::new(Transaction::new(workspace)))
        } else {
            TransactionState::Rejected
        };
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        if !matches!(self.transaction, TransactionState::Active(_)) {
            return self.reject("compiler effects require an active transaction");
        }
        if let TransactionState::Active(transaction) = &self.transaction
            && transaction.fatal.is_some()
        {
            return EffectOutcome::Rejected(
                "compiler transaction stopped after a fatal report".into(),
            );
        }
        match request {
            CompilerRequest::BeginIntercept {
                workspace,
                flags,
            } => self.begin_intercept(workspace, flags),
            CompilerRequest::EndIntercept {
                workspace,
            } => self.end_intercept(workspace),
            CompilerRequest::WaitForMessage => self.wait_for_message(),
            CompilerRequest::GetWorkspaceName {
                workspace,
            } => self.get_name(workspace),
            CompilerRequest::DestroyWorkspace {
                workspace,
            } => self.destroy(workspace),
            CompilerRequest::SetWorkspaceStatus {
                workspace,
                status,
            } => self.stage(workspace, Change::Status(workspace, status)),
            CompilerRequest::WriteOutput {
                stream,
                bytes,
            } => {
                let TransactionState::Active(transaction) = &mut self.transaction else {
                    unreachable!()
                };
                let Some(staged_bytes) = transaction.output_bytes.checked_add(bytes.len()) else {
                    return self.reject("compiler output byte count overflow");
                };
                let committed_bytes = transaction
                    .prepared
                    .as_ref()
                    .map_or(self.output_bytes, |state| state.output_bytes);
                if committed_bytes
                    .checked_add(staged_bytes)
                    .is_none_or(|total| total > self.output_limit)
                {
                    return self.reject("pending compiler output exceeds the session byte limit");
                }
                transaction.output_bytes = staged_bytes;
                transaction.changes.push(Change::Output(CompilerOutput {
                    stream,
                    bytes,
                }));
                EffectOutcome::Ready(CompilerResponse::Unit)
            }
            CompilerRequest::GetBuildOptions {
                workspace,
            } => self.get_options(workspace),
            CompilerRequest::AddSourceAt {
                workspace,
                source,
                location,
            } => self.stage(
                workspace,
                Change::Input(
                    workspace,
                    BuildInput::SourceAt {
                        source,
                        location,
                    },
                ),
            ),
            CompilerRequest::AddSourceFileAt {
                workspace,
                path,
                location,
            } => {
                if path.as_os_str().is_empty() {
                    return self.reject("compiler source path must not be empty");
                }
                self.stage(
                    workspace,
                    Change::Input(
                        workspace,
                        BuildInput::FileAt {
                            path,
                            location,
                        },
                    ),
                )
            }
            CompilerRequest::Report {
                level,
                continuation,
                location,
                text,
            } => {
                if continuation == ReportContinuation::Stop && level != MessageLevel::Error {
                    return self.reject("only an error report can stop a compiler transaction");
                }
                let TransactionState::Active(transaction) = &mut self.transaction else {
                    unreachable!()
                };
                if continuation == ReportContinuation::Stop {
                    transaction.fatal = Some(FatalReport {
                        location: location.clone(),
                        message: text.clone(),
                    });
                }
                let message = CompilerMessage {
                    level,
                    text,
                    location: Some(location),
                };
                transaction
                    .changes
                    .push(Change::Message(transaction.source_workspace, message));
                EffectOutcome::Ready(CompilerResponse::Unit)
            }
            CompilerRequest::CreateWorkspace {
                name,
            } => {
                let Some(id) = allocate_workspace() else {
                    return self.reject("workspace identities exhausted");
                };
                let TransactionState::Active(transaction) = &mut self.transaction else {
                    unreachable!()
                };
                transaction.created.insert(
                    id,
                    BuildWorkspace {
                        id,
                        name,
                        inputs: vec![],
                        settings: BuildSettings::default(),
                        option_origins: BTreeMap::new(),
                        status: WorkspaceStatus::Ok,
                        error: None,
                    },
                );
                EffectOutcome::Ready(CompilerResponse::Workspace(id))
            }
            CompilerRequest::AddSource {
                workspace,
                source,
            } => self.stage(
                workspace,
                Change::Input(workspace, BuildInput::Source(source)),
            ),
            CompilerRequest::AddSourceFile {
                workspace,
                path,
            } => {
                if path.as_os_str().is_empty() {
                    return self.reject("compiler source path must not be empty");
                }
                self.stage(workspace, Change::Input(workspace, BuildInput::File(path)))
            }
            CompilerRequest::SetBuildOptionAt {
                workspace,
                option,
                location,
            } => {
                if matches!(&option, BuildOption::TemporaryStorageSize(size) if *size < 0) {
                    return self.reject("temporary storage size must be nonnegative");
                }
                if matches!(&option, BuildOption::OutputPath(path) if path.as_os_str().is_empty()) {
                    let reason = format!(
                        "{}:{}:{}: compiler output path must not be empty",
                        location.path.display(),
                        location.line,
                        location.column
                    );
                    return self.reject(&reason);
                }
                self.stage(workspace, Change::Option(workspace, option, Some(location)))
            }
            CompilerRequest::SetBuildOption {
                workspace,
                option,
            } => {
                if matches!(&option, BuildOption::TemporaryStorageSize(size) if *size < 0) {
                    return self.reject("temporary storage size must be nonnegative");
                }
                if matches!(&option, BuildOption::OutputPath(path) if path.as_os_str().is_empty()) {
                    return self.reject("compiler output path must not be empty");
                }
                self.stage(workspace, Change::Option(workspace, option, None))
            }
            CompilerRequest::Message {
                level,
                text,
            } => {
                let TransactionState::Active(transaction) = &mut self.transaction else {
                    unreachable!()
                };
                transaction.changes.push(Change::Message(
                    transaction.source_workspace,
                    CompilerMessage {
                        level,
                        text,
                        location: None,
                    },
                ));
                EffectOutcome::Ready(CompilerResponse::Unit)
            }
        }
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        if *request != CompilerRequest::WaitForMessage {
            return EffectOutcome::Rejected("only an issued compiler wait can be polled".into());
        }
        self.poll_message(key)
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        let state = std::mem::take(&mut self.transaction);
        let mut transaction = match state {
            TransactionState::Active(transaction) => *transaction,
            TransactionState::Idle | TransactionState::Rejected => {
                return if commit {
                    Err(jai_vm::Error::EffectRejected(
                        "compiler transaction is not active".into(),
                    ))
                } else {
                    Ok(())
                };
            }
        };
        if !commit {
            return Ok(());
        }
        if let Some(message) = transaction.fatal {
            return Err(jai_vm::Error::CompilerDiagnostic {
                location: message.location,
                message: message.message,
            });
        }
        if transaction.interception.is_waiting() {
            return Err(jai_vm::Error::EffectRejected(
                "compiler job cannot commit before its pending wait is received".into(),
            ));
        }
        let changed = !transaction.created.is_empty()
            || !transaction.changes.is_empty()
            || transaction.prepared.is_some();
        if let Some(prepared) = transaction.prepared {
            if prepared.base_revision != self.revision {
                return Err(jai_vm::Error::EffectRejected(
                    CompilerTransactionError::ChangedSession.to_string(),
                ));
            }
            prepared.publish(self);
        }
        self.workspaces.append(&mut transaction.created);
        self.output_bytes += transaction.output_bytes;
        for change in transaction.changes {
            match change {
                Change::Destroy(id) => {
                    self.workspaces.remove(&id);
                    self.destroyed.insert(id);
                }
                Change::Input(id, input) => self
                    .workspaces
                    .get_mut(&id)
                    .expect("staged workspace identities were checked")
                    .inputs
                    .push(input),
                Change::Option(id, option, location) => {
                    let workspace = self
                        .workspaces
                        .get_mut(&id)
                        .expect("staged workspace identities were checked");
                    for &key in setting_keys(&option) {
                        if let Some(location) = &location {
                            workspace.option_origins.insert(key, location.clone());
                        } else {
                            workspace.option_origins.remove(&key);
                        }
                    }
                    workspace.settings.apply(option);
                }
                Change::Message(id, message) => {
                    if message.level == MessageLevel::Error {
                        // The running recipe may retire its own workspace. Its
                        // diagnostic remains observable without resurrecting it.
                        if let Some(workspace) = self.workspaces.get_mut(&id) {
                            workspace.status = WorkspaceStatus::Failed;
                            if workspace.error.is_none() {
                                workspace.error = Some(message.clone());
                            }
                        }
                    }
                    self.messages.push(message);
                }
                Change::Status(id, status) => {
                    let workspace = self
                        .workspaces
                        .get_mut(&id)
                        .expect("staged workspace identity was checked");
                    workspace.status = status;
                    match status {
                        WorkspaceStatus::Ok => workspace.error = None,
                        WorkspaceStatus::Failed if workspace.error.is_none() => {
                            workspace.error = Some(CompilerMessage {
                                level: MessageLevel::Error,
                                text: format!("workspace {} was marked as failed", id.get()),
                                location: None,
                            });
                        }
                        WorkspaceStatus::Failed => {}
                    }
                }
                Change::Output(output) => self.outputs.push(output),
            }
        }
        self.error = self
            .workspaces
            .values()
            .filter(|workspace| workspace.status == WorkspaceStatus::Failed)
            .find_map(|workspace| workspace.error.clone());
        if changed {
            self.revision = self.revision.wrapping_add(1);
        }
        Ok(())
    }
}

#[path = "compiler_effects/parked_transactions.rs"]
mod parked_transactions;
use parked_transactions::PreparedCompilerState;

#[path = "compiler_effects/interception.rs"]
mod interception;
pub use interception::CompilerEventError;
use interception::Interception;
pub use parked_transactions::{
    CompilerJobId, CompilerTransactionError, SuspendedCompilerTransaction,
};

#[cfg(test)]
#[path = "compiler_effects/workspace_lifecycle_tests.rs"]
mod workspace_lifecycle_tests;

#[cfg(test)]
mod tests {
    use super::*;
    fn create(session: &mut CompilerSession) -> WorkspaceId {
        match session.request(CompilerRequest::CreateWorkspace {
            name: "target".into(),
        }) {
            EffectOutcome::Ready(CompilerResponse::Workspace(id)) => id,
            other => panic!("{other:?}"),
        }
    }
    fn source(session: &mut CompilerSession, id: WorkspaceId, text: &str) {
        assert_eq!(
            session.request(CompilerRequest::AddSource {
                workspace: id,
                source: text.into()
            }),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
    }
    #[test]
    fn commit_preserves_request_order_and_hides_staged_inputs() {
        let mut session = CompilerSession::new();
        session.begin();
        let id = create(&mut session);
        source(&mut session, id, "A :: 1;");
        source(&mut session, id, "B :: 2;");
        assert!(session.workspace(id).is_none());
        assert_eq!(
            session.request(CompilerRequest::SetBuildOption {
                workspace: id,
                option: BuildOption::Optimize(true),
            }),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
        session.finish(true).unwrap();
        let workspace = session.workspace(id).unwrap();
        assert_eq!(
            workspace.inputs(),
            &[
                BuildInput::Source("A :: 1;".into()),
                BuildInput::Source("B :: 2;".into())
            ]
        );
        assert_eq!(workspace.settings().bitcode, BitcodeOptimization::O2);
    }
    #[test]
    fn retry_does_not_reuse_rolled_back_workspace_handles() {
        let mut session = CompilerSession::new();
        session.begin();
        let stale = create(&mut session);
        source(&mut session, stale, "discard");
        session.finish(false).unwrap();
        session.begin();
        let fresh = create(&mut session);
        assert_ne!(stale, fresh);
        source(&mut session, fresh, "keep");
        session.finish(true).unwrap();
        assert!(session.workspace(stale).is_none());
        assert_eq!(
            session.workspace(fresh).unwrap().inputs(),
            &[BuildInput::Source("keep".into())]
        );
    }
    #[test]
    fn workspaces_cannot_cross_session_boundaries() {
        let mut first = CompilerSession::new();
        let mut second = CompilerSession::new();
        assert_ne!(first.root(), second.root());
        first.begin();
        second.begin();
        let first_child = create(&mut first);
        let second_child = create(&mut second);
        assert_ne!(first_child, second_child);
        first.finish(true).unwrap();
        second.finish(true).unwrap();
        second.begin();
        assert!(matches!(
            second.request(CompilerRequest::AddSource {
                workspace: first_child,
                source: "foreign".into(),
            }),
            EffectOutcome::Rejected(_)
        ));
        assert!(second.finish(true).is_err());
        assert!(second.workspace(second_child).unwrap().inputs().is_empty());
        assert!(second.workspace(first_child).is_none());
    }
    #[test]
    fn rejection_discards_earlier_inputs_options_and_messages() {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        source(&mut session, root, "discard");
        session.request(CompilerRequest::Message {
            level: MessageLevel::Warning,
            text: "discard".into(),
        });
        let missing = WorkspaceId::from_raw(999).unwrap();
        assert!(matches!(
            session.request(CompilerRequest::SetBuildOption {
                workspace: missing,
                option: BuildOption::Optimize(true),
            }),
            EffectOutcome::Rejected(_)
        ));
        assert!(session.finish(true).is_err());
        assert!(session.workspace(root).unwrap().inputs().is_empty());
        assert!(session.take_messages().is_empty());
        session.begin();
        source(&mut session, root, "keep");
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().inputs(),
            &[BuildInput::Source("keep".into())]
        );
    }
    #[test]
    fn starting_a_new_transaction_discards_unfinished_work() {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        source(&mut session, root, "discard");
        session.begin();
        source(&mut session, root, "keep");
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().inputs(),
            &[BuildInput::Source("keep".into())]
        );
        assert!(matches!(
            session.request(CompilerRequest::Message {
                level: MessageLevel::Info,
                text: "outside".into(),
            }),
            EffectOutcome::Rejected(_)
        ));
    }
    #[test]
    fn source_locations_optimization_levels_and_continuable_errors_survive_commit() {
        let mut session = CompilerSession::new();
        let root = session.root();
        let location = SourceLocation {
            path: "build.jai".into(),
            line: 7,
            column: 3,
        };
        session.begin();
        session.request(CompilerRequest::AddSourceAt {
            workspace: root,
            source: "generated :: 5;".into(),
            location: location.clone(),
        });
        session.request(CompilerRequest::SetBuildOption {
            workspace: root,
            option: BuildOption::BitcodeOptimization(BitcodeOptimization::Oz),
        });
        session.request(CompilerRequest::SetBuildOption {
            workspace: root,
            option: BuildOption::MachineOptimization(MachineOptimization::Aggressive),
        });
        session.request(CompilerRequest::Report {
            level: MessageLevel::Error,
            continuation: ReportContinuation::Continue,
            location: location.clone(),
            text: "failed custom check".into(),
        });
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().settings().bitcode,
            BitcodeOptimization::Oz
        );
        assert_eq!(
            session.workspace(root).unwrap().settings().machine,
            MachineOptimization::Aggressive
        );
        assert!(
            matches!(&session.workspace(root).unwrap().inputs()[0], BuildInput::SourceAt { location: origin, .. } if origin == &location)
        );
        assert_eq!(
            session.take_messages()[0].location.as_ref(),
            Some(&location)
        );
        assert_eq!(session.error().unwrap().text, "failed custom check");
    }
    #[test]
    fn get_options_sees_staged_changes_but_rollback_restores_committed_settings() {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        session.request(CompilerRequest::SetBuildOption {
            workspace: root,
            option: BuildOption::BitcodeOptimization(BitcodeOptimization::O3),
        });
        session.request(CompilerRequest::SetBuildOption {
            workspace: root,
            option: BuildOption::BitcodeOptimization(BitcodeOptimization::Os),
        });
        let EffectOutcome::Ready(CompilerResponse::BuildOptions(snapshot)) =
            session.request(CompilerRequest::GetBuildOptions {
                workspace: root,
            })
        else {
            panic!("expected build options");
        };
        assert_eq!(snapshot.bitcode, BitcodeOptimization::Os);
        assert_eq!(
            session.workspace(root).unwrap().settings().bitcode,
            BitcodeOptimization::Unset
        );
        session.finish(false).unwrap();
        session.begin();
        let EffectOutcome::Ready(CompilerResponse::BuildOptions(snapshot)) =
            session.request(CompilerRequest::GetBuildOptions {
                workspace: root,
            })
        else {
            panic!("expected build options");
        };
        assert_eq!(snapshot.bitcode, BitcodeOptimization::Unset);
        session.finish(true).unwrap();
    }

    #[test]
    fn output_and_runtime_settings_are_transactional_and_workspace_specific() {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        let child = create(&mut session);
        for option in [
            BuildOption::OutputKind(BuildOutputKind::Object),
            BuildOption::RuntimeSupport(RuntimeSupportMode::InitializationOnly),
            BuildOption::BacktraceOnCrash(BacktraceOnCrash::Off),
        ] {
            assert_eq!(
                session.request(CompilerRequest::SetBuildOption {
                    workspace: child,
                    option,
                }),
                EffectOutcome::Ready(CompilerResponse::Unit)
            );
        }
        let snapshot = |session: &mut CompilerSession, workspace| {
            let EffectOutcome::Ready(CompilerResponse::BuildOptions(snapshot)) =
                session.request(CompilerRequest::GetBuildOptions {
                    workspace,
                })
            else {
                panic!("expected build options");
            };
            snapshot
        };
        let staged = snapshot(&mut session, child);
        assert_eq!(staged.output_kind, BuildOutputKind::Object);
        assert_eq!(
            staged.runtime_support,
            RuntimeSupportMode::InitializationOnly
        );
        assert_eq!(staged.backtrace_on_crash, BacktraceOnCrash::Off);
        let root_snapshot = snapshot(&mut session, root);
        assert_eq!(root_snapshot.output_kind, BuildOutputKind::Executable);
        assert_eq!(root_snapshot.runtime_support, RuntimeSupportMode::Auto);
        assert_eq!(root_snapshot.backtrace_on_crash, BacktraceOnCrash::On);
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(child).unwrap().settings().output_kind,
            BuildOutputKind::Object
        );

        session.begin();
        session.request(CompilerRequest::SetBuildOption {
            workspace: child,
            option: BuildOption::OutputKind(BuildOutputKind::DynamicLibrary),
        });
        assert_eq!(
            snapshot(&mut session, child).output_kind,
            BuildOutputKind::DynamicLibrary
        );
        session.finish(false).unwrap();
        session.begin();
        assert_eq!(snapshot(&mut session, child), staged);
        session.finish(true).unwrap();
    }

    #[test]
    fn fatal_report_prevents_commit_even_when_the_caller_requests_it() {
        let mut session = CompilerSession::new();
        let root = session.root();
        let location = SourceLocation {
            path: "recipe.jai".into(),
            line: 8,
            column: 4,
        };
        session.begin();
        let child = create(&mut session);
        source(&mut session, root, "discarded :: 1;");
        assert_eq!(
            session.request(CompilerRequest::Report {
                level: MessageLevel::Error,
                continuation: ReportContinuation::Stop,
                location: location.clone(),
                text: "cannot build this workspace".into(),
            }),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
        assert!(matches!(
            session.request(CompilerRequest::AddSource {
                workspace: root,
                source: "later :: 2;".into(),
            }),
            EffectOutcome::Rejected(_)
        ));
        assert_eq!(
            session.finish(true),
            Err(jai_vm::Error::CompilerDiagnostic {
                location,
                message: "cannot build this workspace".into(),
            })
        );
        assert!(session.workspace(child).is_none());
        assert!(session.workspace(root).unwrap().inputs().is_empty());
        assert!(session.take_messages().is_empty());
        assert!(session.error().is_none());
        session.begin();
        source(&mut session, root, "accepted :: 3;");
        session.finish(true).unwrap();
        assert_eq!(
            session.workspace(root).unwrap().inputs(),
            &[BuildInput::Source("accepted :: 3;".into())]
        );
    }

    #[test]
    fn stop_mode_requires_an_error_report_and_rolls_back_earlier_changes() {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        source(&mut session, root, "discarded :: 1;");
        assert!(matches!(
            session.request(CompilerRequest::Report {
                level: MessageLevel::Warning,
                continuation: ReportContinuation::Stop,
                location: SourceLocation {
                    path: "recipe.jai".into(),
                    line: 1,
                    column: 1
                },
                text: "warning".into(),
            }),
            EffectOutcome::Rejected(_)
        ));
        assert!(session.finish(true).is_err());
        assert!(session.workspace(root).unwrap().inputs().is_empty());
    }

    fn output(
        session: &mut CompilerSession,
        stream: CompilerOutputStream,
        bytes: &[u8],
    ) -> EffectOutcome {
        session.request(CompilerRequest::WriteOutput {
            stream,
            bytes: bytes.to_vec(),
        })
    }

    #[test]
    fn output_preserves_stream_order_and_arbitrary_bytes_only_after_commit() {
        let mut session = CompilerSession::new();
        session.begin();
        output(
            &mut session,
            CompilerOutputStream::StandardOutput,
            b"first\0\xff",
        );
        output(&mut session, CompilerOutputStream::StandardError, b"second");
        assert!(session.take_outputs().is_empty());
        session.finish(true).unwrap();
        assert_eq!(
            session.take_outputs(),
            vec![
                CompilerOutput {
                    stream: CompilerOutputStream::StandardOutput,
                    bytes: b"first\0\xff".to_vec()
                },
                CompilerOutput {
                    stream: CompilerOutputStream::StandardError,
                    bytes: b"second".to_vec()
                },
            ]
        );
    }

    #[test]
    fn output_limit_counts_pending_commits_and_draining_releases_the_budget() {
        let mut session = CompilerSession::with_output_limit(4);
        session.begin();
        output(&mut session, CompilerOutputStream::StandardOutput, b"123");
        session.finish(true).unwrap();
        session.begin();
        output(&mut session, CompilerOutputStream::StandardError, b"4");
        assert!(matches!(
            output(&mut session, CompilerOutputStream::StandardOutput, b"5"),
            EffectOutcome::Rejected(_)
        ));
        assert!(session.finish(true).is_err());
        assert_eq!(
            session.take_outputs(),
            vec![CompilerOutput {
                stream: CompilerOutputStream::StandardOutput,
                bytes: b"123".to_vec(),
            }]
        );
        session.begin();
        assert_eq!(
            output(&mut session, CompilerOutputStream::StandardError, b"1234"),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
        session.finish(true).unwrap();
        assert_eq!(session.take_outputs()[0].bytes, b"1234");
    }

    #[test]
    fn rejected_output_discards_source_changes_and_rollback_discards_output() {
        let mut session = CompilerSession::with_output_limit(1);
        let root = session.root();
        session.begin();
        source(&mut session, root, "discarded :: 1;");
        assert!(matches!(
            output(
                &mut session,
                CompilerOutputStream::StandardOutput,
                b"too much"
            ),
            EffectOutcome::Rejected(_)
        ));
        assert!(session.finish(true).is_err());
        assert!(session.workspace(root).unwrap().inputs().is_empty());
        assert!(session.take_outputs().is_empty());
        session.begin();
        output(&mut session, CompilerOutputStream::StandardOutput, b"x");
        session.finish(false).unwrap();
        assert!(session.take_outputs().is_empty());
    }
}

#[cfg(test)]
mod option_location_tests {
    use super::*;
    #[test]
    fn option_origin_commits_rolls_back_and_tracks_latest_setting() {
        let mut session = CompilerSession::new();
        let root = session.root();
        let location = SourceLocation {
            path: "recipe.jai".into(),
            line: 12,
            column: 9,
        };
        let request = CompilerRequest::SetBuildOptionAt {
            workspace: root,
            option: BuildOption::OutputKind(BuildOutputKind::Object),
            location: location.clone(),
        };
        session.begin();
        assert_eq!(
            session.request(request.clone()),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
        assert!(
            session
                .workspace(root)
                .unwrap()
                .option_origin(BuildSetting::OutputKind)
                .is_none()
        );
        session.finish(false).unwrap();
        assert!(
            session
                .workspace(root)
                .unwrap()
                .option_origin(BuildSetting::OutputKind)
                .is_none()
        );
        session.begin();
        assert_eq!(
            session.request(request),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
        session.finish(true).unwrap();
        assert_eq!(
            session
                .workspace(root)
                .unwrap()
                .option_origin(BuildSetting::OutputKind),
            Some(&location)
        );
        assert_eq!(
            session.workspace(root).unwrap().settings().output_kind,
            BuildOutputKind::Object
        );
        session.begin();
        assert_eq!(
            session.request(CompilerRequest::SetBuildOption {
                workspace: root,
                option: BuildOption::OutputKind(BuildOutputKind::None)
            }),
            EffectOutcome::Ready(CompilerResponse::Unit)
        );
        session.finish(true).unwrap();
        assert!(
            session
                .workspace(root)
                .unwrap()
                .option_origin(BuildSetting::OutputKind)
                .is_none()
        );
    }
}
