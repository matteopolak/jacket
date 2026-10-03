//! Explicit host data protocol. Constructing a request never accesses the OS.
mod capabilities;
mod programs;
mod virtual_paths;
use crate::SourceOrigin;
pub use capabilities::HostCapability;
pub use programs::{ProgramScopeLimits, ReviewedProgramGrant, ReviewedPrograms};
use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
fn identity() -> u64 {
    NEXT_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .expect("host capability identities exhausted")
}
/// Capability allocated by a trusted provider, never reconstructed from Jai integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileRootId(u64);
impl FileRootId {
    pub fn allocate() -> Self {
        Self(identity())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgramId(u64);
impl ProgramId {
    pub fn allocate() -> Self {
        Self(identity())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HostPath {
    root: FileRootId,
    relative: PathBuf,
}
impl HostPath {
    pub fn new(root: FileRootId, relative: impl AsRef<Path>) -> Result<Self, HostError> {
        let relative = relative.as_ref();
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || relative.as_os_str().as_encoded_bytes().contains(&0)
        {
            return Err(HostError::InvalidPath);
        }
        Ok(Self {
            root,
            relative: relative.into(),
        })
    }
    pub fn root(&self) -> FileRootId {
        self.root
    }
    pub fn relative(&self) -> &Path {
        &self.relative
    }
    /// Actual encoded path backing, inspected without traversing components.
    pub(crate) fn retained_path_capacity(&self) -> usize {
        self.relative.capacity()
    }
}
/// Argument boundaries are preserved; there is no shell command string.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProcessArguments(Vec<OsString>);
impl ProcessArguments {
    pub fn new(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, HostError> {
        let arguments: Vec<_> = arguments.into_iter().collect();
        if arguments.iter().any(|a| a.as_encoded_bytes().contains(&0)) {
            return Err(HostError::InvalidArguments);
        }
        Ok(Self(arguments))
    }
    pub fn as_slice(&self) -> &[OsString] {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramInvocation {
    pub program: ProgramId,
    pub arguments: ProcessArguments,
    pub working_root: FileRootId,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HostRequest {
    ReadEntireFile(HostPath),
    /// OS open failures are source-observable data; policy rejection remains fatal.
    ReadFileForOpen(HostPath),
    WriteEntireFile {
        path: HostPath,
        bytes: Vec<u8>,
    },
    RunProgram(ProgramInvocation),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessTermination {
    Exited(i32),
    Signaled,
    TimedOut,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessOutput {
    pub termination: ProcessTermination,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessHostPlatform {
    MacOS,
    Linux,
    Windows,
    Other,
}
/// An observed OS launch failure, not a denied capability. The source adapter
/// must match its selected platform before writing this actual errno value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessLaunchFailure {
    platform: ProcessHostPlatform,
    errno: i32,
}
impl ProcessLaunchFailure {
    pub fn from_os_error(platform: ProcessHostPlatform, errno: i32) -> Result<Self, HostError> {
        if errno <= 0 {
            return Err(HostError::InvalidArguments);
        }
        Ok(Self {
            platform,
            errno,
        })
    }
    pub fn platform(self) -> ProcessHostPlatform {
        self.platform
    }
    pub fn errno(self) -> i32 {
        self.errno
    }
    pub fn matches_platform(self, target: &jai_types::OperatingSystem) -> bool {
        matches!(
            (self.platform, target),
            (
                ProcessHostPlatform::MacOS,
                jai_types::OperatingSystem::MacOS
            ) | (
                ProcessHostPlatform::Linux,
                jai_types::OperatingSystem::Linux
            ) | (
                ProcessHostPlatform::Windows,
                jai_types::OperatingSystem::Windows
            )
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOpenFailure {
    NotFound,
    PermissionDenied,
    NotRegular,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileOpenObservation {
    Bytes(Vec<u8>),
    Failed(FileOpenFailure),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostResponse {
    FileBytes(Vec<u8>),
    FileOpen(FileOpenObservation),
    WriteStaged,
    Process(ProcessOutput),
    ProcessLaunchFailed(ProcessLaunchFailure),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HostRequestKey(u64);
impl HostRequestKey {
    pub fn allocate() -> Self {
        Self(identity())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostError {
    Unavailable,
    InvalidPath,
    InvalidArguments,
    UnknownCapability,
    UnsupportedCapability(HostCapability),
    Denied(&'static str),
    Budget(&'static str),
    Transaction(&'static str),
    Io(String),
}
impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "host effect: {self:?}")
    }
}
impl std::error::Error for HostError {
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostOutcome {
    Ready(HostResponse),
    Pending(HostRequestKey),
    Rejected(HostError),
}
/// Reads are observations, writes are staged, processes require driver readiness.
/// Providers must bound all retained bytes and reject unsupported capabilities.
pub trait HostEffects {
    fn begin(&mut self, origin: SourceOrigin) -> Result<(), HostError>;
    fn request(&mut self, request: HostRequest) -> HostOutcome;
    fn finish(&mut self, commit: bool) -> Result<(), HostError>;
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NoHostEffects;
impl HostEffects for NoHostEffects {
    fn begin(&mut self, _: SourceOrigin) -> Result<(), HostError> {
        Ok(())
    }
    fn request(&mut self, _: HostRequest) -> HostOutcome {
        HostOutcome::Rejected(HostError::Unavailable)
    }
    fn finish(&mut self, _: bool) -> Result<(), HostError> {
        Ok(())
    }
}

/// Driver-owned lexical scope for source C filenames. It grants one registered
/// root, never follows symlinks or consults the host filesystem inside the VM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePathScope {
    root: FileRootId,
    canonical_root: PathBuf,
    working_relative: PathBuf,
    virtual_namespace: bool,
}
impl FilePathScope {
    pub fn new(
        root: FileRootId,
        canonical_root: impl AsRef<Path>,
        working_relative: impl AsRef<Path>,
    ) -> Result<Self, HostError> {
        let canonical_root = canonical_root.as_ref();
        if !canonical_root.is_absolute()
            || canonical_root
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        {
            return Err(HostError::InvalidPath);
        }
        let working_relative = normalize_relative(working_relative.as_ref())?;
        Ok(Self {
            root,
            canonical_root: canonical_root.into(),
            working_relative,
            virtual_namespace: false,
        })
    }
    pub fn root(&self) -> FileRootId {
        self.root
    }
    pub fn canonical_root(&self) -> &Path {
        &self.canonical_root
    }
    pub fn resolve(&self, source: &Path) -> Result<HostPath, HostError> {
        if self.virtual_namespace {
            return self.resolve_virtual(source);
        }
        if source.as_os_str().is_empty() || source.as_os_str().as_encoded_bytes().contains(&0) {
            return Err(HostError::InvalidPath);
        }
        let relative = if source.is_absolute() {
            // Normalize parent components before containment checking.
            let mut prefix = PathBuf::new();
            let mut normals = PathBuf::new();
            for component in source.components() {
                match component {
                    Component::Prefix(_) | Component::RootDir => prefix.push(component),
                    Component::Normal(value) => normals.push(value),
                    Component::CurDir => {}
                    Component::ParentDir if normals.pop() => {}
                    Component::ParentDir => {
                        return Err(HostError::Denied("filename escapes filesystem root"));
                    }
                }
            }
            prefix.push(normals);
            prefix
                .strip_prefix(&self.canonical_root)
                .map_err(|_| HostError::Denied("filename is outside its granted root"))?
                .to_path_buf()
        } else {
            normalize_relative(&self.working_relative.join(source))?
        };
        HostPath::new(self.root, relative)
    }
}
fn normalize_relative(path: &Path) -> Result<PathBuf, HostError> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => result.push(value),
            Component::CurDir => {}
            Component::ParentDir if result.pop() => {}
            Component::ParentDir => return Err(HostError::Denied("filename escapes granted root")),
            Component::Prefix(_) | Component::RootDir => return Err(HostError::InvalidPath),
        }
    }
    Ok(result)
}
