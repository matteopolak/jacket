mod compiler_messages;
mod continuation;
pub(crate) use continuation::{EffectJournal, JournalEffects};
mod source;
pub(crate) use source::workspace_argument;
mod source_location;
mod source_options;
use crate::{Dependency, Error, Value};
pub use compiler_messages::{
    CompilerCompletion, CompilerEvent, CompilerMessageSchema, CompilerPhase, InterceptFlags,
};
use jai_types::{FieldId, Integer, IntegerType, TypeId, TypeView};
use std::{fmt, path::PathBuf, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkspaceId(u64);
impl WorkspaceId {
    pub fn from_raw(value: u64) -> Option<Self> {
        (value != 0).then_some(Self(value))
    }
    pub fn get(self) -> u64 {
        self.0
    }
}
/// Trusted source-scheduler identity for replaying one directive transaction.
/// Exact body/specialization bytes participate in equality; hashes are not proofs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceOrigin {
    pub workspace: WorkspaceId,
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
    pub body_hash: u64,
    pub body: Vec<u8>,
    pub specialization: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetTriple(String);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetTripleError;
impl TargetTriple {
    pub fn parse(text: &str) -> Result<Self, TargetTripleError> {
        let count = text.split('-').count();
        if !(3..=5).contains(&count)
            || text.split('-').any(|segment| {
                segment.is_empty()
                    || !segment
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.')
            })
        {
            return Err(TargetTripleError);
        }
        Ok(Self(text.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for TargetTriple {
    type Err = TargetTripleError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}
impl fmt::Display for TargetTripleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("target triple requires 3 to 5 nonempty ASCII identifier segments")
    }
}
impl std::error::Error for TargetTripleError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectKey(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildOption {
    OutputKind(BuildOutputKind),
    RuntimeSupport(RuntimeSupportMode),
    BacktraceOnCrash(BacktraceOnCrash),
    BitcodeOptimization(BitcodeOptimization),
    MachineOptimization(MachineOptimization),
    Optimize(bool),
    OutputPath(PathBuf),
    Target(TargetTriple),
}
pub use jai_types::{BacktraceOnCrash, BuildOutputKind, RuntimeSupportMode};
pub use jai_types::{BitcodeOptimization, MachineOptimization};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub line: u64,
    pub column: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordFieldPath {
    pub outer: FieldId,
    pub inner: Option<FieldId>,
    pub leaf: Option<FieldId>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildOptionsProjection {
    pub output_kind: Option<RecordFieldPath>,
    pub runtime_support: Option<RecordFieldPath>,
    pub backtrace_on_crash: Option<RecordFieldPath>,
    pub output_path: Option<RecordFieldPath>,
    pub target: Option<RecordFieldPath>,
    pub bitcode: Option<RecordFieldPath>,
    pub machine: Option<RecordFieldPath>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkspaceStatus {
    #[default]
    Ok,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportContinuation {
    Stop,
    Continue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerOutputStream {
    StandardOutput,
    StandardError,
}
/// These requests contain compiler data only. The VM never accesses the OS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerRequest {
    BeginIntercept {
        workspace: WorkspaceId,
        flags: InterceptFlags,
    },
    EndIntercept {
        workspace: WorkspaceId,
    },
    WaitForMessage,
    GetWorkspaceName {
        workspace: WorkspaceId,
    },
    DestroyWorkspace {
        workspace: WorkspaceId,
    },
    SetWorkspaceStatus {
        workspace: WorkspaceId,
        status: WorkspaceStatus,
    },
    SetBuildOptionAt {
        workspace: WorkspaceId,
        option: BuildOption,
        location: SourceLocation,
    },
    WriteOutput {
        stream: CompilerOutputStream,
        bytes: Vec<u8>,
    },
    GetBuildOptions {
        workspace: WorkspaceId,
    },
    AddSourceAt {
        workspace: WorkspaceId,
        source: String,
        location: SourceLocation,
    },
    AddSourceFileAt {
        workspace: WorkspaceId,
        path: PathBuf,
        location: SourceLocation,
    },
    Report {
        level: MessageLevel,
        continuation: ReportContinuation,
        location: SourceLocation,
        text: String,
    },
    AddSource {
        workspace: WorkspaceId,
        source: String,
    },
    AddSourceFile {
        workspace: WorkspaceId,
        path: PathBuf,
    },
    CreateWorkspace {
        name: String,
    },
    SetBuildOption {
        workspace: WorkspaceId,
        option: BuildOption,
    },
    Message {
        level: MessageLevel,
        text: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildOptionsSnapshot {
    pub output_kind: BuildOutputKind,
    pub runtime_support: RuntimeSupportMode,
    pub backtrace_on_crash: BacktraceOnCrash,
    pub output_path: Option<PathBuf>,
    pub target: Option<TargetTriple>,
    pub bitcode: BitcodeOptimization,
    pub machine: MachineOptimization,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerResponse {
    Message(CompilerEvent),
    WorkspaceName(String),
    Unit,
    Workspace(WorkspaceId),
    BuildOptions(BuildOptionsSnapshot),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectOutcome {
    Ready(CompilerResponse),
    Pending(EffectKey),
    Rejected(String),
}
/// A driver must stage effects between begin/finish and discard them on rollback.
/// A Ready response may reserve an identity, but must not publish changes early.
pub trait CompilerEffects {
    fn set_source_origin(&mut self, _origin: SourceOrigin) {}
    /// Host requests share this handler's begin/finish transaction.
    fn host_request(
        &mut self,
        _: crate::host_effects::HostRequest,
    ) -> crate::host_effects::HostOutcome {
        crate::host_effects::HostOutcome::Rejected(crate::host_effects::HostError::Unavailable)
    }
    fn host_file_scope(&self) -> Option<crate::host_effects::FilePathScope> {
        None
    }
    fn begin(&mut self);
    /// Park the current transaction without committing or discarding its requests.
    /// A scheduler must restore the same source origin before `resume`.
    fn suspend(&mut self) -> Result<(), Error> {
        Err(Error::EffectRejected(
            "this effect handler cannot suspend transactions".into(),
        ))
    }
    /// Restore the transaction parked by `suspend`; this never calls `begin` again.
    fn resume(&mut self) -> Result<(), Error> {
        Err(Error::EffectRejected(
            "this effect handler cannot resume transactions".into(),
        ))
    }
    /// Advance genuinely runnable dependency jobs while this transaction is
    /// parked. `true` means new readiness; it does not commit the waiting job.
    fn service_pending(&mut self, _dependencies: &[Dependency]) -> Result<bool, Error> {
        Ok(false)
    }
    /// Resolve an already-issued request without advancing the request stream.
    fn poll_request(&mut self, _request: &CompilerRequest, _key: EffectKey) -> EffectOutcome {
        EffectOutcome::Rejected("this effect handler cannot poll pending requests".into())
    }
    fn poll_host_request(
        &mut self,
        _key: crate::host_effects::HostRequestKey,
    ) -> crate::host_effects::HostOutcome {
        crate::host_effects::HostOutcome::Rejected(crate::host_effects::HostError::Unavailable)
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome;
    fn finish(&mut self, commit: bool) -> Result<(), Error>;
}
impl<E: CompilerEffects + ?Sized> CompilerEffects for &mut E {
    fn set_source_origin(&mut self, origin: SourceOrigin) {
        (**self).set_source_origin(origin)
    }
    fn host_request(
        &mut self,
        request: crate::host_effects::HostRequest,
    ) -> crate::host_effects::HostOutcome {
        (**self).host_request(request)
    }
    fn host_file_scope(&self) -> Option<crate::host_effects::FilePathScope> {
        (**self).host_file_scope()
    }
    fn begin(&mut self) {
        (**self).begin();
    }
    fn suspend(&mut self) -> Result<(), Error> {
        (**self).suspend()
    }
    fn resume(&mut self) -> Result<(), Error> {
        (**self).resume()
    }
    fn service_pending(&mut self, dependencies: &[Dependency]) -> Result<bool, Error> {
        (**self).service_pending(dependencies)
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        (**self).poll_request(request, key)
    }
    fn poll_host_request(
        &mut self,
        key: crate::host_effects::HostRequestKey,
    ) -> crate::host_effects::HostOutcome {
        (**self).poll_host_request(key)
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        (**self).request(request)
    }
    fn finish(&mut self, commit: bool) -> Result<(), Error> {
        (**self).finish(commit)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEffects;
impl CompilerEffects for NoEffects {
    fn begin(&mut self) {}
    fn suspend(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn resume(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn request(&mut self, _: CompilerRequest) -> EffectOutcome {
        EffectOutcome::Rejected("compiler effects are unavailable in this execution context".into())
    }
    fn finish(&mut self, _: bool) -> Result<(), Error> {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerIntrinsic {
    SourceBeginIntercept {
        current_workspace: WorkspaceId,
    },
    SourceEndIntercept {
        current_workspace: WorkspaceId,
    },
    SourceWaitForMessage {
        schema: CompilerMessageSchema,
    },
    SourceVersionInfo {
        record: TypeId,
    },
    SourceRuntimeInfo {
        current_workspace: WorkspaceId,
        schema: jai_types::RuntimeInfoSchema,
    },
    SourceGetWorkspaceName {
        current_workspace: WorkspaceId,
    },
    SourceDestroyWorkspace {
        current_workspace: WorkspaceId,
    },
    SourceSetWorkspaceStatus {
        current_workspace: WorkspaceId,
    },
    SourceWriteString,
    SourceWriteStrings,
    SourceDebugBreak,
    SourceGetBuildOptions {
        current_workspace: WorkspaceId,
        projection: BuildOptionsProjection,
        result: TypeId,
    },
    SourceAddStringAt {
        current_workspace: WorkspaceId,
    },
    SourceAddFileAt {
        current_workspace: WorkspaceId,
    },
    SourceReportAt,
    SourceSetBuildOptionsAt {
        current_workspace: WorkspaceId,
        projection: BuildOptionsProjection,
    },
    SourceSetBuildOptions {
        current_workspace: WorkspaceId,
        projection: BuildOptionsProjection,
    },
    SourceAddString {
        current_workspace: WorkspaceId,
    },
    SourceAddFile {
        current_workspace: WorkspaceId,
    },
    SourceCreateWorkspace,
    SourceCurrentWorkspace {
        current_workspace: WorkspaceId,
    },
    SourceReport {
        level: MessageLevel,
    },
    AddSource,
    AddSourceFile,
    CreateWorkspace,
    SetOptimization,
    SetOutputPath,
    SetTarget,
    Message(MessageLevel),
}
#[derive(Clone, Copy, Debug)]
pub struct CompilerProcedure {
    pub signature: TypeId,
    pub intrinsic: CompilerIntrinsic,
}
pub(crate) enum EffectError {
    Pending(Dependency),
    Failed(Error),
}
impl CompilerIntrinsic {
    pub(crate) fn invoke(
        self,
        arguments: &[Value],
        effects: &mut impl CompilerEffects,
        types: &dyn TypeView,
    ) -> Result<Vec<Value>, EffectError> {
        if matches!(
            self,
            Self::SourceAddFileAt { .. }
                | Self::SourceAddStringAt { .. }
                | Self::SourceWriteString
                | Self::SourceWriteStrings
                | Self::SourceVersionInfo { .. }
                | Self::SourceRuntimeInfo { .. }
                | Self::SourceBeginIntercept { .. }
                | Self::SourceEndIntercept { .. }
                | Self::SourceWaitForMessage { .. }
                | Self::SourceDebugBreak
                | Self::SourceReportAt
                | Self::SourceGetBuildOptions { .. }
                | Self::SourceSetBuildOptions { .. }
                | Self::SourceSetBuildOptionsAt { .. }
                | Self::SourceAddString { .. }
                | Self::SourceAddFile { .. }
                | Self::SourceCreateWorkspace
                | Self::SourceCurrentWorkspace { .. }
                | Self::SourceReport { .. }
                | Self::SourceSetWorkspaceStatus { .. }
                | Self::SourceDestroyWorkspace { .. }
                | Self::SourceGetWorkspaceName { .. }
        ) {
            return self.source(arguments, effects, types);
        }
        let fail = |reason| EffectError::Failed(Error::InvalidIr(reason));
        let text = |index| match arguments.get(index) {
            Some(Value::String(text)) => String::from_utf8(text.clone())
                .map_err(|_| fail("compiler intrinsic requires a UTF-8 string")),
            _ => Err(fail("compiler intrinsic requires a string argument")),
        };
        let workspace = |index| match arguments.get(index) {
            Some(Value::Int(value)) if value.ty() == IntegerType::U64 => {
                WorkspaceId::from_raw(value.bits())
                    .ok_or_else(|| fail("workspace identity must be nonzero"))
            }
            _ => Err(fail("compiler intrinsic requires a u64 workspace identity")),
        };
        let expected = match self {
            Self::CreateWorkspace | Self::Message(_) => 1,
            _ => 2,
        };
        if arguments.len() != expected {
            return Err(fail("compiler intrinsic has incorrect argument count"));
        }
        let request = match self {
            Self::AddSource => CompilerRequest::AddSource {
                workspace: workspace(0)?,
                source: text(1)?,
            },
            Self::AddSourceFile => CompilerRequest::AddSourceFile {
                workspace: workspace(0)?,
                path: PathBuf::from(text(1)?),
            },
            Self::CreateWorkspace => CompilerRequest::CreateWorkspace { name: text(0)? },
            Self::SetOptimization => {
                let Some(Value::Bool(enabled)) = arguments.get(1) else {
                    return Err(fail("optimization option requires a boolean"));
                };
                CompilerRequest::SetBuildOption {
                    workspace: workspace(0)?,
                    option: BuildOption::Optimize(*enabled),
                }
            }
            Self::SetOutputPath => CompilerRequest::SetBuildOption {
                workspace: workspace(0)?,
                option: BuildOption::OutputPath(PathBuf::from(text(1)?)),
            },
            Self::SetTarget => CompilerRequest::SetBuildOption {
                workspace: workspace(0)?,
                option: BuildOption::Target(
                    TargetTriple::parse(&text(1)?)
                        .map_err(|_| fail("target triple has invalid structure"))?,
                ),
            },
            Self::SourceAddFileAt { .. }
            | Self::SourceAddStringAt { .. }
            | Self::SourceWriteString
            | Self::SourceWriteStrings
            | Self::SourceVersionInfo { .. }
            | Self::SourceRuntimeInfo { .. }
            | Self::SourceBeginIntercept { .. }
            | Self::SourceEndIntercept { .. }
            | Self::SourceWaitForMessage { .. }
            | Self::SourceDebugBreak
            | Self::SourceReportAt
            | Self::SourceGetBuildOptions { .. }
            | Self::SourceSetBuildOptions { .. }
            | Self::SourceSetBuildOptionsAt { .. }
            | Self::SourceAddString { .. }
            | Self::SourceAddFile { .. }
            | Self::SourceCreateWorkspace
            | Self::SourceCurrentWorkspace { .. }
            | Self::SourceReport { .. }
            | Self::SourceSetWorkspaceStatus { .. }
            | Self::SourceDestroyWorkspace { .. }
            | Self::SourceGetWorkspaceName { .. } => {
                return Err(fail("source intrinsic reached internal adapter"));
            }
            Self::Message(level) => CompilerRequest::Message {
                level,
                text: text(0)?,
            },
        };
        match effects.request(request) {
            EffectOutcome::Ready(CompilerResponse::Workspace(id))
                if self == Self::CreateWorkspace =>
            {
                Ok(vec![Value::Int(Integer::wrapping(
                    IntegerType::U64,
                    i128::from(id.0),
                ))])
            }
            EffectOutcome::Ready(CompilerResponse::Unit) if self != Self::CreateWorkspace => {
                Ok(vec![])
            }
            EffectOutcome::Ready(_) => Err(EffectError::Failed(Error::EffectResponse(
                "response kind does not match intrinsic",
            ))),
            EffectOutcome::Pending(key) => Err(EffectError::Pending(Dependency::Effect(key))),
            EffectOutcome::Rejected(reason) => {
                Err(EffectError::Failed(Error::EffectRejected(reason)))
            }
        }
    }
}
