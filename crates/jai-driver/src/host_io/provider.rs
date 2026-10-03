//! Opt-in host provider; no supplied compiler/library is loaded or invoked.
use jai_vm::SourceOrigin;
use jai_vm::host_effects::*;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
pub struct HostLimits {
    pub bytes: usize,
    pub requests: usize,
    pub process_output_bytes: usize,
    pub process_timeout: Duration,
}
impl Default for HostLimits {
    fn default() -> Self {
        Self {
            bytes: 16 * 1024 * 1024,
            requests: 1024,
            process_output_bytes: 1024 * 1024,
            process_timeout: Duration::from_secs(5),
        }
    }
}
#[derive(Debug)]
struct Root {
    path: PathBuf,
    writable: bool,
}
#[derive(Debug)]
struct Program {
    path: PathBuf,
    fingerprint: [u8; 32],
    argument_zero: std::ffi::OsString,
    arguments: ProcessArguments,
    root: FileRootId,
}
enum ProgramObservation {
    Completed(ProcessOutput),
    LaunchFailed(ProcessLaunchFailure),
}
#[derive(Clone, Debug)]
struct Observation {
    request: HostRequest,
    outcome: HostOutcome,
}
#[derive(Debug)]
struct Transaction {
    origin: Arc<SourceOrigin>,
    cursor: usize,
    writes: Vec<(HostPath, Vec<u8>)>,
    failed: bool,
}
static NEXT_PROVIDER: AtomicU64 = AtomicU64::new(1);
/// Owns unpublished host staging while a VM continuation waits. It cannot be
/// cloned or fabricated; dropping it discards its overlay without publication.
#[derive(Debug)]
pub struct SuspendedHostTransaction {
    provider: u64,
    transaction: Transaction,
    lifetime: Arc<()>,
}
impl SuspendedHostTransaction {
    pub fn origin(&self) -> &SourceOrigin {
        &self.transaction.origin
    }
}
/// Constructed by the driver, never by source strings. An empty provider denies every capability.
#[derive(Debug)]
pub struct HostIo {
    identity: u64,
    roots: HashMap<FileRootId, Root>,
    protected_inputs: Vec<PathBuf>,
    original_native_fingerprints: HashSet<[u8; 32]>,
    programs: HashMap<ProgramId, Program>,
    observations: HashMap<Arc<SourceOrigin>, Vec<Observation>>,
    pending: HashMap<HostRequestKey, (Arc<SourceOrigin>, usize)>,
    tickets: HashMap<HostRequestKey, (Arc<SourceOrigin>, usize)>,
    parked: HashMap<Arc<SourceOrigin>, Weak<()>>,
    committed: HashMap<Arc<SourceOrigin>, usize>,
    transaction: Option<Transaction>,
    limits: HostLimits,
    retained_bytes: usize,
    registry_bytes: usize,
}
impl Default for HostIo {
    fn default() -> Self {
        Self::new(HostLimits::default())
    }
}
fn io(error: std::io::Error) -> HostError {
    HostError::Io(error.to_string())
}
impl HostIo {
    pub fn new(limits: HostLimits) -> Self {
        Self {
            identity: NEXT_PROVIDER
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("host provider identities exhausted"),
            roots: HashMap::new(),
            protected_inputs: Vec::new(),
            original_native_fingerprints: HashSet::new(),
            programs: HashMap::new(),
            observations: HashMap::new(),
            pending: HashMap::new(),
            tickets: HashMap::new(),
            parked: HashMap::new(),
            committed: HashMap::new(),
            transaction: None,
            limits,
            retained_bytes: 0,
            registry_bytes: 0,
        }
    }
    pub fn file_scope(
        &self,
        root: FileRootId,
        working_relative: &Path,
    ) -> Result<FilePathScope, HostError> {
        let path = &self
            .roots
            .get(&root)
            .ok_or(HostError::UnknownCapability)?
            .path;
        FilePathScope::new(root, path, working_relative)
    }
    /// Register a driver-owned root. Callers must prevent concurrent filesystem mutation.
    pub fn register_root(&mut self, path: &Path, writable: bool) -> Result<FileRootId, HostError> {
        let path = fs::canonicalize(path).map_err(io)?;
        if !path.is_dir() {
            return Err(HostError::InvalidPath);
        }
        let bytes = path.as_os_str().as_encoded_bytes().len();
        self.reserve_registration(bytes)?;
        let id = FileRootId::allocate();
        self.roots.insert(id, Root { path, writable });
        Ok(id)
    }
    /// Trusted inventory protection for original inputs. Granted static reads
    /// remain available, while writes and executable registration are denied.
    pub fn protect_original_inputs(&mut self, path: &Path) -> Result<(), HostError> {
        if self.transaction.is_some()
            || !self.observations.is_empty()
            || self
                .parked
                .values()
                .any(|lifetime| lifetime.strong_count() != 0)
        {
            return Err(HostError::Transaction(
                "input protection must precede source observations",
            ));
        }
        let path = fs::canonicalize(path).map_err(io)?;
        if self
            .protected_inputs
            .iter()
            .any(|original| path.starts_with(original))
        {
            return Ok(());
        }
        if self
            .programs
            .values()
            .any(|program| program.path.starts_with(&path))
        {
            return Err(HostError::Denied(
                "registered executable overlaps original inputs",
            ));
        }
        self.reserve_registration(path.as_os_str().as_encoded_bytes().len())?;
        self.protected_inputs.push(path);
        Ok(())
    }
    fn is_original_input(&self, path: &Path) -> bool {
        self.protected_inputs
            .iter()
            .any(|original| path.starts_with(original))
    }
    pub(super) fn apply_original_protection(
        &mut self,
        roots: &[PathBuf],
        fingerprints: &[[u8; 32]],
    ) -> Result<(), HostError> {
        if self.transaction.is_some()
            || !self.observations.is_empty()
            || self
                .parked
                .values()
                .any(|lifetime| lifetime.strong_count() != 0)
        {
            return Err(HostError::Transaction(
                "input policy must precede source observations",
            ));
        }
        if self.programs.values().any(|program| {
            roots.iter().any(|root| program.path.starts_with(root))
                || fingerprints.contains(&program.fingerprint)
        }) {
            return Err(HostError::Denied(
                "registered executable overlaps original input policy",
            ));
        }
        let mut new_roots = Vec::new();
        for root in roots {
            if !self.is_original_input(root) && !new_roots.contains(root) {
                new_roots.push(root.clone());
            }
        }
        let new_fingerprints: HashSet<_> = fingerprints
            .iter()
            .copied()
            .filter(|hash| !self.original_native_fingerprints.contains(hash))
            .collect();
        let count = self
            .roots
            .len()
            .checked_add(self.programs.len())
            .and_then(|n| n.checked_add(self.protected_inputs.len()))
            .and_then(|n| n.checked_add(self.original_native_fingerprints.len()))
            .and_then(|n| n.checked_add(new_roots.len()))
            .and_then(|n| n.checked_add(new_fingerprints.len()))
            .ok_or(HostError::Budget("capability count"))?;
        if count > self.limits.requests {
            return Err(HostError::Budget("capability count"));
        }
        let bytes = new_roots
            .iter()
            .try_fold(self.registry_bytes, |total, path| {
                total.checked_add(path.as_os_str().as_encoded_bytes().len())
            })
            .and_then(|total| {
                new_fingerprints
                    .len()
                    .checked_mul(32)
                    .and_then(|n| total.checked_add(n))
            })
            .ok_or(HostError::Budget("registry bytes"))?;
        if bytes > self.limits.bytes {
            return Err(HostError::Budget("registry bytes"));
        }
        self.registry_bytes = bytes;
        self.protected_inputs.extend(new_roots);
        self.original_native_fingerprints.extend(new_fingerprints);
        Ok(())
    }
    /// Trusted native-input inventory digest. Copies outside protected roots
    /// remain prohibited; this denylist grants no read or launch capability.
    pub fn protect_original_native_fingerprint(
        &mut self,
        fingerprint: [u8; 32],
    ) -> Result<(), HostError> {
        if self.transaction.is_some()
            || !self.observations.is_empty()
            || self
                .parked
                .values()
                .any(|lifetime| lifetime.strong_count() != 0)
        {
            return Err(HostError::Transaction(
                "native protection must precede source observations",
            ));
        }
        if self.original_native_fingerprints.contains(&fingerprint) {
            return Ok(());
        }
        if self
            .programs
            .values()
            .any(|program| program.fingerprint == fingerprint)
        {
            return Err(HostError::Denied(
                "registered executable matches original native input",
            ));
        }
        self.reserve_registration(fingerprint.len())?;
        self.original_native_fingerprints.insert(fingerprint);
        Ok(())
    }
    /// Trusted driver registration for one independently installed, reviewed, read-only invocation.
    /// This is not a sandbox: never register shell interpreters, supplied tools or commands that
    /// write externally, use networks, spawn descendants, or depend on inherited credentials.
    pub fn register_read_only_program(
        &mut self,
        executable: &Path,
        arguments: ProcessArguments,
        working_root: FileRootId,
    ) -> Result<ProgramId, HostError> {
        self.register_program(executable, None, arguments, working_root)
    }
    /// The reviewed argv[0] is part of executable behavior (for example applet
    /// selection). It must be reviewed together with the actual executable.
    #[cfg(unix)]
    pub fn register_read_only_program_with_argument_zero(
        &mut self,
        executable: &Path,
        argument_zero: std::ffi::OsString,
        arguments: ProcessArguments,
        working_root: FileRootId,
    ) -> Result<ProgramId, HostError> {
        self.register_program(executable, Some(argument_zero), arguments, working_root)
    }
    fn register_program(
        &mut self,
        executable: &Path,
        argument_zero: Option<std::ffi::OsString>,
        arguments: ProcessArguments,
        working_root: FileRootId,
    ) -> Result<ProgramId, HostError> {
        if !self.roots.contains_key(&working_root) {
            return Err(HostError::UnknownCapability);
        }
        if !executable.is_absolute() {
            return Err(HostError::InvalidPath);
        }
        let path = fs::canonicalize(executable).map_err(io)?;
        if self.is_original_input(&path) {
            return Err(HostError::Denied(
                "original input cannot be a reviewed executable",
            ));
        }
        if !path.is_file() {
            return Err(HostError::InvalidPath);
        }
        let argument_zero = argument_zero.unwrap_or_else(|| path.as_os_str().to_owned());
        if argument_zero.as_encoded_bytes().contains(&0) {
            return Err(HostError::InvalidArguments);
        }
        if arguments
            .as_slice()
            .iter()
            .try_fold(0usize, |n, arg| n.checked_add(arg.as_encoded_bytes().len()))
            .is_none_or(|n| n > self.limits.bytes)
        {
            return Err(HostError::Budget("arguments"));
        }
        let argument_bytes = arguments
            .as_slice()
            .iter()
            .try_fold(0usize, |n, arg| n.checked_add(arg.as_encoded_bytes().len()))
            .ok_or(HostError::Budget("arguments"))?;
        let bytes = path
            .as_os_str()
            .as_encoded_bytes()
            .len()
            .checked_add(argument_bytes)
            .and_then(|bytes| bytes.checked_add(argument_zero.as_encoded_bytes().len()))
            .ok_or(HostError::Budget("registry bytes"))?;
        let fingerprint = self.program_fingerprint(&path)?;
        if self.original_native_fingerprints.contains(&fingerprint) {
            return Err(HostError::Denied(
                "original native bytes cannot be a reviewed executable",
            ));
        }
        self.reserve_registration(bytes)?;
        let id = ProgramId::allocate();
        self.programs.insert(
            id,
            Program {
                path,
                fingerprint,
                argument_zero,
                arguments,
                root: working_root,
            },
        );
        Ok(id)
    }
    fn program_fingerprint(&self, path: &Path) -> Result<[u8; 32], HostError> {
        let metadata = fs::symlink_metadata(path).map_err(io)?;
        if !metadata.is_file() || self.is_original_input(path) {
            return Err(HostError::Denied("reviewed executable path changed"));
        }
        if metadata.len() > self.limits.bytes as u64 {
            return Err(HostError::Budget("executable fingerprint bytes"));
        }
        let mut file = fs::File::open(path).map_err(io)?;
        let mut digest = Sha256::new();
        let mut bytes = 0usize;
        let mut buffer = [0u8; 8192];
        loop {
            let count = file.read(&mut buffer).map_err(io)?;
            if count == 0 {
                break;
            }
            bytes = bytes
                .checked_add(count)
                .ok_or(HostError::Budget("executable fingerprint bytes"))?;
            if bytes > self.limits.bytes {
                return Err(HostError::Budget("executable fingerprint bytes"));
            }
            digest.update(&buffer[..count]);
        }
        Ok(digest.finalize().into())
    }
    /// Bind a source exec spelling to an existing exact provider registration.
    /// This neither resolves PATH nor inspects/executes another candidate.
    pub fn reviewed_program_grant(
        &self,
        program: ProgramId,
        source_executable: std::ffi::OsString,
    ) -> Result<ReviewedProgramGrant, HostError> {
        let registered = self
            .programs
            .get(&program)
            .ok_or(HostError::UnknownCapability)?;
        let root = self
            .roots
            .get(&registered.root)
            .ok_or(HostError::UnknownCapability)?;
        ReviewedProgramGrant::from_registered(
            source_executable,
            registered.argument_zero.clone(),
            ProgramInvocation {
                program,
                arguments: registered.arguments.clone(),
                working_root: registered.root,
            },
            root.path.clone(),
        )
    }
    fn reserve_registration(&mut self, bytes: usize) -> Result<(), HostError> {
        if self
            .roots
            .len()
            .checked_add(self.programs.len())
            .and_then(|count| count.checked_add(self.protected_inputs.len()))
            .and_then(|count| count.checked_add(self.original_native_fingerprints.len()))
            .is_none_or(|n| n >= self.limits.requests)
        {
            return Err(HostError::Budget("capability count"));
        }
        let total = self
            .registry_bytes
            .checked_add(bytes)
            .ok_or(HostError::Budget("registry bytes"))?;
        if total > self.limits.bytes {
            return Err(HostError::Budget("registry bytes"));
        }
        self.registry_bytes = total;
        Ok(())
    }
    fn path(&self, path: &HostPath, write: bool) -> Result<PathBuf, HostError> {
        self.resolve_path(path, write, write)
    }
    fn resolve_path(
        &self,
        path: &HostPath,
        write: bool,
        allow_missing: bool,
    ) -> Result<PathBuf, HostError> {
        let root = self
            .roots
            .get(&path.root())
            .ok_or(HostError::UnknownCapability)?;
        if write && !root.writable {
            return Err(HostError::Denied("root is read-only"));
        }
        let mut result = root.path.clone();
        for component in path.relative().components() {
            result.push(component);
            match fs::symlink_metadata(&result) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(HostError::Denied("symlink path"));
                }
                Ok(_) => {}
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        && allow_missing
                        && (result == root.path.join(path.relative()) || !write) => {}
                Err(error) => return Err(io(error)),
            }
        }
        if write && self.is_original_input(&result) {
            return Err(HostError::Denied("original inputs are immutable"));
        }
        Ok(result)
    }
    fn bounded_read(&self, path: &Path) -> Result<Vec<u8>, HostError> {
        let mut file = fs::File::open(path).map_err(io)?;
        if !file.metadata().map_err(io)?.is_file() {
            return Err(HostError::Denied("only regular files can be read"));
        }
        let mut bytes = vec![];
        (&mut file)
            .take(self.limits.bytes.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > self.limits.bytes {
            return Err(HostError::Budget("file bytes"));
        }
        Ok(bytes)
    }
    fn read_for_open(&self, path: &HostPath) -> Result<FileOpenObservation, HostError> {
        let path = self.resolve_path(path, false, true)?;
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) => {
                return Ok(FileOpenObservation::Failed(match error.kind() {
                    std::io::ErrorKind::NotFound => FileOpenFailure::NotFound,
                    std::io::ErrorKind::PermissionDenied => FileOpenFailure::PermissionDenied,
                    _ => FileOpenFailure::Other,
                }));
            }
        };
        if !file.metadata().map_err(io)?.is_file() {
            return Ok(FileOpenObservation::Failed(FileOpenFailure::NotRegular));
        }
        let mut bytes = vec![];
        file.take(self.limits.bytes.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > self.limits.bytes {
            return Err(HostError::Budget("file bytes"));
        }
        Ok(FileOpenObservation::Bytes(bytes))
    }
    fn reject(&mut self, error: HostError) -> HostOutcome {
        if let Some(transaction) = self.transaction.as_mut() {
            transaction.failed = true;
        }
        HostOutcome::Rejected(error)
    }
    /// Execute one ready process outside VM evaluation. Repeated service calls never rerun it.
    pub fn service(&mut self, key: HostRequestKey) -> Result<(), HostError> {
        if self.pending.get(&key).is_some_and(|(origin, _)| {
            self.parked
                .get(origin)
                .is_some_and(|lifetime| lifetime.strong_count() != 0)
        }) {
            return Err(HostError::Denied(
                "parked process service requires its sealed transaction",
            ));
        }
        self.service_inner(key)
    }
    fn service_inner(&mut self, key: HostRequestKey) -> Result<(), HostError> {
        if self.transaction.is_some() {
            return Err(HostError::Transaction(
                "process service during VM transaction",
            ));
        }
        let Some((origin, index)) = self.pending.get(&key).cloned() else {
            return Err(HostError::Transaction(
                "unknown or already serviced process request",
            ));
        };
        let request = self
            .observations
            .get(&origin)
            .and_then(|records| records.get(index))
            .ok_or(HostError::Transaction("missing process observation"))?
            .request
            .clone();
        let HostRequest::RunProgram(invocation) = request else {
            return Err(HostError::Transaction("pending request is not a process"));
        };
        let output = self.run(&invocation);
        let outcome = match output {
            Ok(ProgramObservation::Completed(output)) => {
                let bytes = output
                    .stdout
                    .len()
                    .checked_add(output.stderr.len())
                    .ok_or(HostError::Budget("process bytes"))?;
                if self
                    .retained_bytes
                    .checked_add(bytes)
                    .is_none_or(|n| n > self.limits.bytes)
                {
                    HostOutcome::Rejected(HostError::Budget("retained bytes"))
                } else {
                    self.retained_bytes += bytes;
                    HostOutcome::Ready(HostResponse::Process(output))
                }
            }
            Ok(ProgramObservation::LaunchFailed(failure)) => {
                HostOutcome::Ready(HostResponse::ProcessLaunchFailed(failure))
            }
            Err(error) => HostOutcome::Rejected(error),
        };
        self.observations.get_mut(&origin).unwrap()[index].outcome = outcome;
        self.pending.remove(&key);
        Ok(())
    }
    /// Reject a pending request without starting a program, releasing its readiness ticket.
    pub fn cancel(&mut self, key: HostRequestKey) -> Result<(), HostError> {
        let (origin, index) = self
            .pending
            .remove(&key)
            .ok_or(HostError::Transaction("unknown pending process request"))?;
        self.observations.get_mut(&origin).unwrap()[index].outcome =
            HostOutcome::Rejected(HostError::Denied("process cancelled by driver"));
        Ok(())
    }
    /// Observe the same readiness ticket without consuming another source request slot.
    /// A resumed continuation must supply the original full source identity.
    pub fn completion(
        &self,
        origin: &SourceOrigin,
        key: HostRequestKey,
    ) -> Result<HostOutcome, HostError> {
        let (owner, index) = self.tickets.get(&key).ok_or(HostError::Transaction(
            "unknown or retired readiness ticket",
        ))?;
        if owner.as_ref() != origin {
            return Err(HostError::Denied(
                "readiness ticket belongs to another source",
            ));
        }
        self.observations
            .get(owner)
            .and_then(|records| records.get(*index))
            .map(|record| record.outcome.clone())
            .ok_or(HostError::Transaction("missing readiness observation"))
    }
    fn retain_origin(&mut self, origin: &Arc<SourceOrigin>) -> Result<(), HostError> {
        if self.observations.contains_key(origin) {
            return Ok(());
        }
        if self.observations.len() >= self.limits.requests {
            return Err(HostError::Budget("source identity count"));
        }
        let total = self
            .retained_bytes
            .checked_add(origin_bytes(origin)?)
            .ok_or(HostError::Budget("source identity bytes"))?;
        if total > self.limits.bytes {
            return Err(HostError::Budget("source identity bytes"));
        }
        self.retained_bytes = total;
        self.observations.insert(Arc::clone(origin), vec![]);
        Ok(())
    }
    pub fn suspend_transaction(&mut self) -> Result<SuspendedHostTransaction, HostError> {
        self.parked
            .retain(|_, lifetime| lifetime.strong_count() != 0);
        let transaction = self
            .transaction
            .as_ref()
            .ok_or(HostError::Transaction("suspend outside transaction"))?;
        if transaction.failed {
            return Err(HostError::Transaction("failed transaction cannot suspend"));
        }
        let origin = Arc::clone(&transaction.origin);
        if self.parked.contains_key(&origin) {
            return Err(HostError::Transaction(
                "source transaction is already suspended",
            ));
        }
        self.retain_origin(&origin)?;
        let lifetime = Arc::new(());
        self.parked.insert(origin, Arc::downgrade(&lifetime));
        Ok(SuspendedHostTransaction {
            provider: self.identity,
            transaction: self.transaction.take().expect("validated transaction"),
            lifetime,
        })
    }
    fn validate_suspended(&self, suspended: &SuspendedHostTransaction) -> Result<(), HostError> {
        if suspended.provider != self.identity {
            return Err(HostError::Denied(
                "host transaction belongs to another provider",
            ));
        }
        if !self
            .parked
            .get(&suspended.transaction.origin)
            .is_some_and(|lifetime| lifetime.ptr_eq(&Arc::downgrade(&suspended.lifetime)))
        {
            return Err(HostError::Transaction("unknown suspended host transaction"));
        }
        Ok(())
    }
    pub fn resume_transaction(
        &mut self,
        suspended: SuspendedHostTransaction,
    ) -> Result<(), HostError> {
        self.validate_suspended(&suspended)?;
        if self.transaction.is_some() {
            return Err(HostError::Transaction("another host transaction is active"));
        }
        self.parked.remove(&suspended.transaction.origin);
        self.transaction = Some(suspended.transaction);
        Ok(())
    }
    /// Cancel a parked journal without resuming or publishing it. Pending
    /// programs become rejected observations and can never be serviced later.
    pub fn cancel_suspended(
        &mut self,
        suspended: SuspendedHostTransaction,
    ) -> Result<(), HostError> {
        self.validate_suspended(&suspended)?;
        let keys: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(key, (origin, _))| {
                (origin == &suspended.transaction.origin).then_some(*key)
            })
            .collect();
        for key in keys {
            self.cancel(key)?;
        }
        self.parked.remove(&suspended.transaction.origin);
        Ok(())
    }
    /// Service only the exact parked origin, retaining its cursor and overlay.
    pub fn service_suspended(
        &mut self,
        suspended: &SuspendedHostTransaction,
        key: HostRequestKey,
    ) -> Result<(), HostError> {
        self.validate_suspended(suspended)?;
        if !self
            .pending
            .get(&key)
            .is_some_and(|(owner, _)| owner == &suspended.transaction.origin)
        {
            return Err(HostError::Denied(
                "readiness ticket differs from parked source",
            ));
        }
        if !suspended.transaction.writes.is_empty() {
            return Err(HostError::Denied(
                "process cannot observe uncommitted file writes",
            ));
        }
        self.service_inner(key)
    }
    fn run(&self, invocation: &ProgramInvocation) -> Result<ProgramObservation, HostError> {
        let program = self
            .programs
            .get(&invocation.program)
            .ok_or(HostError::UnknownCapability)?;
        if program.arguments != invocation.arguments || program.root != invocation.working_root {
            return Err(HostError::Denied(
                "invocation differs from reviewed arguments or working root",
            ));
        }
        if self.program_fingerprint(&program.path)? != program.fingerprint {
            return Err(HostError::Denied("reviewed executable bytes changed"));
        }
        let root = &self
            .roots
            .get(&program.root)
            .ok_or(HostError::UnknownCapability)?
            .path;
        let mut command = Command::new(&program.path);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.arg0(&program.argument_zero);
        }
        let child = command
            .args(invocation.arguments.as_slice())
            .current_dir(root)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(error) => {
                let Some(errno) = error.raw_os_error() else {
                    return Err(io(error));
                };
                let platform = if cfg!(target_os = "macos") {
                    ProcessHostPlatform::MacOS
                } else if cfg!(target_os = "linux") {
                    ProcessHostPlatform::Linux
                } else if cfg!(target_os = "windows") {
                    ProcessHostPlatform::Windows
                } else {
                    ProcessHostPlatform::Other
                };
                return Ok(ProgramObservation::LaunchFailed(
                    ProcessLaunchFailure::from_os_error(platform, errno)?,
                ));
            }
        };
        let exceeded = Arc::new(AtomicBool::new(false));
        let stdout = capture(
            child.stdout.take().unwrap(),
            self.limits.process_output_bytes,
            Arc::clone(&exceeded),
        );
        let stderr = capture(
            child.stderr.take().unwrap(),
            self.limits.process_output_bytes,
            Arc::clone(&exceeded),
        );
        let started = Instant::now();
        let mut timeout = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(io(error));
                }
            }
            if exceeded.load(Ordering::Relaxed) || started.elapsed() >= self.limits.process_timeout
            {
                timeout = !exceeded.load(Ordering::Relaxed);
                let _ = child.kill();
                break child.wait().map_err(io)?;
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        let stdout = stdout
            .join()
            .map_err(|_| HostError::Transaction("stdout reader failed"))??;
        let stderr = stderr
            .join()
            .map_err(|_| HostError::Transaction("stderr reader failed"))??;
        if exceeded.load(Ordering::Relaxed)
            || stdout
                .len()
                .checked_add(stderr.len())
                .is_none_or(|n| n > self.limits.process_output_bytes)
        {
            return Err(HostError::Budget("process output"));
        }
        let termination = if timeout {
            ProcessTermination::TimedOut
        } else {
            status
                .code()
                .map_or(ProcessTermination::Signaled, ProcessTermination::Exited)
        };
        Ok(ProgramObservation::Completed(ProcessOutput {
            termination,
            stdout,
            stderr,
        }))
    }
    /// Release observations only after the scheduler has retired their exact source identity.
    /// Retiring active/pending identities is forbidden, so retries cannot rerun an in-flight process.
    pub fn retire(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        self.parked
            .retain(|_, lifetime| lifetime.strong_count() != 0);
        if self
            .transaction
            .as_ref()
            .is_some_and(|t| t.origin.as_ref() == origin)
            || self.pending.values().any(|(o, _)| o.as_ref() == origin)
            || self.parked.contains_key(origin)
        {
            return Err(HostError::Transaction("source identity is still live"));
        }
        if let Some(records) = self.observations.remove(origin) {
            self.retained_bytes -= origin_bytes(origin)?;
            for record in records {
                self.retained_bytes -= retained(&record);
            }
        }
        self.committed.remove(origin);
        self.tickets
            .retain(|_, (owner, _)| owner.as_ref() != origin);
        Ok(())
    }
}
fn capture<R: Read + Send + 'static>(
    mut pipe: R,
    limit: usize,
    exceeded: Arc<AtomicBool>,
) -> std::thread::JoinHandle<Result<Vec<u8>, HostError>> {
    std::thread::spawn(move || {
        let mut bytes = vec![];
        (&mut pipe)
            .take(limit.saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > limit {
            exceeded.store(true, Ordering::Relaxed);
        }
        Ok(bytes)
    })
}
fn retained(record: &Observation) -> usize {
    let request = match &record.request {
        HostRequest::WriteEntireFile { path, bytes } => {
            path.relative().as_os_str().as_encoded_bytes().len() + bytes.len()
        }
        HostRequest::ReadEntireFile(path) | HostRequest::ReadFileForOpen(path) => {
            path.relative().as_os_str().as_encoded_bytes().len()
        }
        HostRequest::RunProgram(invocation) => invocation
            .arguments
            .as_slice()
            .iter()
            .map(|a| a.as_encoded_bytes().len())
            .sum(),
    };
    request
        + match &record.outcome {
            HostOutcome::Ready(HostResponse::FileBytes(bytes))
            | HostOutcome::Ready(HostResponse::FileOpen(FileOpenObservation::Bytes(bytes))) => {
                bytes.len()
            }
            HostOutcome::Ready(HostResponse::Process(output)) => {
                output.stdout.len() + output.stderr.len()
            }
            _ => 0,
        }
}
fn origin_bytes(origin: &SourceOrigin) -> Result<usize, HostError> {
    origin
        .body
        .len()
        .checked_add(origin.specialization.len())
        .and_then(|n| n.checked_add(origin.path.as_os_str().as_encoded_bytes().len()))
        .ok_or(HostError::Budget("source identity bytes"))
}
impl HostEffects for HostIo {
    fn begin(&mut self, origin: SourceOrigin) -> Result<(), HostError> {
        if self.transaction.is_some() {
            return Err(HostError::Transaction("nested transaction"));
        }
        self.parked
            .retain(|_, lifetime| lifetime.strong_count() != 0);
        if self.parked.contains_key(&origin) {
            return Err(HostError::Transaction("source transaction is suspended"));
        }
        if origin_bytes(&origin)? > self.limits.bytes {
            return Err(HostError::Budget("source identity bytes"));
        }
        let origin = self
            .observations
            .get_key_value(&origin)
            .map(|(o, _)| Arc::clone(o))
            .or_else(|| {
                self.committed
                    .get_key_value(&origin)
                    .map(|(o, _)| Arc::clone(o))
            })
            .unwrap_or_else(|| Arc::new(origin));
        self.transaction = Some(Transaction {
            origin,
            cursor: 0,
            writes: vec![],
            failed: false,
        });
        Ok(())
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        let Some(transaction) = self.transaction.as_mut() else {
            return self.reject(HostError::Transaction("request outside transaction"));
        };
        if transaction.failed {
            return HostOutcome::Rejected(HostError::Transaction("transaction has already failed"));
        }
        let origin = transaction.origin.clone();
        let index = transaction.cursor;
        transaction.cursor += 1;
        if index >= self.limits.requests {
            return self.reject(HostError::Budget("request count"));
        }
        if let Some(record) = self
            .observations
            .get(&origin)
            .and_then(|records| records.get(index))
        {
            if record.request != request {
                return self.reject(HostError::Transaction(
                    "retry diverged from source observation",
                ));
            }
            let outcome = record.outcome.clone();
            if let HostRequest::WriteEntireFile { path, bytes } = request {
                self.transaction
                    .as_mut()
                    .unwrap()
                    .writes
                    .push((path, bytes));
            }
            if matches!(outcome, HostOutcome::Rejected(_)) {
                self.transaction.as_mut().unwrap().failed = true;
            }
            return outcome;
        }
        if self.committed.contains_key(&origin) {
            return self.reject(HostError::Transaction(
                "committed replay has additional requests",
            ));
        }
        let outcome = match &request {
            HostRequest::ReadFileForOpen(path) => {
                let staged = self
                    .transaction
                    .as_ref()
                    .unwrap()
                    .writes
                    .iter()
                    .rev()
                    .find(|(p, _)| p == path)
                    .map(|(_, bytes)| bytes.clone());
                let opened = match staged {
                    Some(bytes) => Ok(FileOpenObservation::Bytes(bytes)),
                    None => self.read_for_open(path),
                };
                match opened {
                    Ok(opened) => HostOutcome::Ready(HostResponse::FileOpen(opened)),
                    Err(error) => return self.reject(error),
                }
            }
            HostRequest::ReadEntireFile(path) => {
                let staged = self
                    .transaction
                    .as_ref()
                    .unwrap()
                    .writes
                    .iter()
                    .rev()
                    .find(|(p, _)| p == path)
                    .map(|(_, bytes)| bytes.clone());
                match staged.map(Ok).unwrap_or_else(|| {
                    self.path(path, false)
                        .and_then(|path| self.bounded_read(&path))
                }) {
                    Ok(bytes) => HostOutcome::Ready(HostResponse::FileBytes(bytes)),
                    Err(error) => return self.reject(error),
                }
            }
            HostRequest::WriteEntireFile { path, bytes } => {
                if let Err(error) = self.path(path, true) {
                    return self.reject(error);
                }
                if bytes.len() > self.limits.bytes {
                    return self.reject(HostError::Budget("write bytes"));
                }
                self.transaction
                    .as_mut()
                    .unwrap()
                    .writes
                    .push((path.clone(), bytes.clone()));
                HostOutcome::Ready(HostResponse::WriteStaged)
            }
            HostRequest::RunProgram(invocation) => {
                let Some(program) = self.programs.get(&invocation.program) else {
                    return self.reject(HostError::UnknownCapability);
                };
                if program.arguments != invocation.arguments
                    || program.root != invocation.working_root
                {
                    return self.reject(HostError::Denied("unreviewed invocation"));
                }
                if !self.transaction.as_ref().unwrap().writes.is_empty() {
                    return self.reject(HostError::Denied(
                        "process cannot observe uncommitted file writes",
                    ));
                }
                HostOutcome::Pending(HostRequestKey::allocate())
            }
        };
        if self.observations.values().map(Vec::len).sum::<usize>() >= self.limits.requests {
            return self.reject(HostError::Budget("retained observation count"));
        }
        if !self.observations.contains_key(&origin)
            && self.observations.len() >= self.limits.requests
        {
            return self.reject(HostError::Budget("source identity count"));
        }
        let record = Observation {
            request,
            outcome: outcome.clone(),
        };
        let identity_bytes = if self.observations.contains_key(&origin) {
            0
        } else {
            match origin_bytes(&origin) {
                Ok(bytes) => bytes,
                Err(error) => return self.reject(error),
            }
        };
        let Some(bytes) = retained(&record).checked_add(identity_bytes) else {
            return self.reject(HostError::Budget("retained bytes"));
        };
        if self
            .retained_bytes
            .checked_add(bytes)
            .is_none_or(|n| n > self.limits.bytes)
        {
            return self.reject(HostError::Budget("retained bytes"));
        }
        if let HostOutcome::Pending(key) = outcome {
            self.pending.insert(key, (origin.clone(), index));
            self.tickets.insert(key, (origin.clone(), index));
        }
        self.retained_bytes += bytes;
        self.observations.entry(origin).or_default().push(record);
        outcome
    }
    fn finish(&mut self, commit: bool) -> Result<(), HostError> {
        let transaction = self
            .transaction
            .take()
            .ok_or(HostError::Transaction("finish outside transaction"))?;
        if !commit {
            return Ok(());
        }
        if transaction.failed {
            return Err(HostError::Transaction("failed transaction cannot commit"));
        }
        if transaction.cursor == 0 && !self.observations.contains_key(&transaction.origin) {
            if self.observations.len() >= self.limits.requests {
                return Err(HostError::Budget("source identity count"));
            }
            let bytes = origin_bytes(&transaction.origin)?;
            let total = self
                .retained_bytes
                .checked_add(bytes)
                .ok_or(HostError::Budget("source identity bytes"))?;
            if total > self.limits.bytes {
                return Err(HostError::Budget("source identity bytes"));
            }
            self.retained_bytes = total;
            self.observations
                .insert(Arc::clone(&transaction.origin), vec![]);
        }
        let records = self.observations.get(&transaction.origin);
        if records.is_some_and(|r| {
            r.len() != transaction.cursor
                || r.iter()
                    .any(|r| !matches!(r.outcome, HostOutcome::Ready(_)))
        }) {
            return Err(HostError::Transaction(
                "transaction has pending or unconsumed observations",
            ));
        }
        if let Some(count) = self.committed.get(&transaction.origin) {
            return if *count == transaction.cursor {
                Ok(())
            } else {
                Err(HostError::Transaction(
                    "committed replay has fewer requests",
                ))
            };
        }
        // Restrict one distinct target per transaction so publication is one atomic rename.
        // A multi-file transaction needs a driver-owned journal, not a claim of OS-wide atomicity.
        if transaction.writes.iter().any(|(path, _)| {
            transaction
                .writes
                .first()
                .is_some_and(|(first, _)| first != path)
        }) {
            return Err(HostError::Denied(
                "multi-file atomic publication is not implemented",
            ));
        }
        if let Some((path, bytes)) = transaction.writes.last() {
            let target = self.path(path, true)?;
            let parent = target.parent().ok_or(HostError::InvalidPath)?;
            let temporary =
                parent.join(format!(".jai-host-stage-{:?}", HostRequestKey::allocate()));
            let result = (|| {
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&temporary)
                    .map_err(io)?;
                file.write_all(bytes).map_err(io)?;
                file.sync_all().map_err(io)?;
                fs::rename(&temporary, target).map_err(io)
            })();
            if result.is_err() {
                let _ = fs::remove_file(&temporary);
            }
            result?;
        }
        self.committed
            .insert(transaction.origin, transaction.cursor);
        Ok(())
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
