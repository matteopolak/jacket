//! Closed in-memory file capabilities with replayable, origin-bound journals.
use crate::{
    ConsoleOutput, HostCapability, HostServices, Platform, PlatformError, SharedVfs, VfsSnapshot,
};
use jai_vm::{CompilerOutputStream, SourceOrigin, host_effects::*};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Copy, Debug)]
pub struct VfsHostLimits {
    pub journal_bytes: usize,
    pub requests_per_origin: usize,
    pub origins: usize,
    pub console_bytes: usize,
}
impl Default for VfsHostLimits {
    fn default() -> Self {
        Self {
            journal_bytes: 8 * 1024 * 1024,
            requests_per_origin: 4096,
            origins: 1024,
            console_bytes: 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Operation {
    Request(HostRequest),
    Console(ConsoleOutput),
}
#[derive(Clone)]
struct Observation {
    operation: Operation,
    response: Option<HostResponse>,
}
struct Journal {
    snapshot: VfsSnapshot,
    observations: Vec<Observation>,
    committed: bool,
}
struct Transaction {
    origin: SourceOrigin,
    cursor: usize,
    writes: BTreeMap<PathBuf, Arc<[u8]>>,
    console: Vec<ConsoleOutput>,
    console_bytes: usize,
    failed: bool,
}
struct Publication {
    bytes: usize,
    revision: u64,
    output_bytes: usize,
}

pub struct VfsHost {
    vfs: SharedVfs,
    root: FileRootId,
    scope: FilePathScope,
    capabilities: Vec<HostCapability>,
    limits: VfsHostLimits,
    retained_bytes: usize,
    journals: HashMap<SourceOrigin, Journal>,
    active: Option<Transaction>,
    parked: HashMap<SourceOrigin, Transaction>,
    output: Vec<ConsoleOutput>,
    output_bytes: usize,
}
impl VfsHost {
    /// Grants belong to this instance. A same-spelled path from another host is denied.
    pub fn new(
        vfs: SharedVfs,
        working: &Path,
        writable: bool,
        console: bool,
        limits: VfsHostLimits,
    ) -> Result<Self, HostError> {
        let root = FileRootId::allocate();
        let scope = FilePathScope::new_virtual(root, Path::new(vfs.root()), working)?;
        let mut capabilities = vec![HostCapability::FileRead];
        if writable {
            capabilities.push(HostCapability::FileWrite);
        }
        if console {
            capabilities.push(HostCapability::Console);
        }
        Ok(Self {
            vfs,
            root,
            scope,
            capabilities,
            limits,
            retained_bytes: 0,
            journals: HashMap::new(),
            active: None,
            parked: HashMap::new(),
            output: vec![],
            output_bytes: 0,
        })
    }
    pub fn root(&self) -> FileRootId {
        self.root
    }
    pub fn take_console(&mut self) -> Vec<ConsoleOutput> {
        self.output_bytes = 0;
        std::mem::take(&mut self.output)
    }
    pub fn vfs(&self) -> &SharedVfs {
        &self.vfs
    }
    /// Validate staging without publication, for native adapters pairing journals.
    pub fn validate_commit(&self) -> Result<(), HostError> {
        let active = self
            .active
            .as_ref()
            .ok_or(HostError::Transaction("VFS transaction is not active"))?;
        let store = self
            .vfs
            .store
            .lock()
            .map_err(|_| HostError::Transaction("VFS store is poisoned"))?;
        self.publication(active, &store).map(|_| ())
    }
    fn publication(
        &self,
        active: &Transaction,
        store: &crate::vfs::Store,
    ) -> Result<Option<Publication>, HostError> {
        if active.failed {
            return Err(HostError::Transaction(
                "failed VFS transaction cannot commit",
            ));
        }
        let journal = &self.journals[&active.origin];
        if active.cursor != journal.observations.len() {
            return Err(HostError::Transaction(
                "VFS replay did not consume its complete journal",
            ));
        }
        if journal.committed {
            return Ok(None);
        }
        let output_bytes = self
            .output_bytes
            .checked_add(active.console_bytes)
            .filter(|bytes| *bytes <= self.limits.console_bytes)
            .ok_or(HostError::Budget("VFS published console limit exceeded"))?;
        if store.revision != journal.snapshot.revision {
            return Err(HostError::Transaction(
                "VFS changed since this source transaction began",
            ));
        }
        let mut bytes = store.bytes;
        let mut files = store.files.len();
        for path in active.writes.keys() {
            match store.files.get(path) {
                Some(old) => bytes -= old.len(),
                None => {
                    files = files
                        .checked_add(1)
                        .ok_or(HostError::Budget("VFS file count overflow"))?;
                    bytes = bytes
                        .checked_add(path.as_os_str().as_encoded_bytes().len())
                        .ok_or(HostError::Budget("VFS filename size overflow"))?;
                }
            }
        }
        for value in active.writes.values() {
            bytes = bytes
                .checked_add(value.len())
                .ok_or(HostError::Budget("VFS byte size overflow"))?;
        }
        if bytes > store.limits.bytes {
            return Err(HostError::Budget("VFS byte limit exceeded"));
        }
        if files > store.limits.files {
            return Err(HostError::Budget("VFS file limit exceeded"));
        }
        let revision = if active.writes.is_empty() {
            store.revision
        } else {
            store
                .revision
                .checked_add(1)
                .ok_or(HostError::Budget("VFS revision space exhausted"))?
        };
        Ok(Some(Publication {
            bytes,
            revision,
            output_bytes,
        }))
    }
    fn path(&self, path: &HostPath) -> Result<PathBuf, HostError> {
        if path.root() != self.root {
            return Err(HostError::UnknownCapability);
        }
        let relative = path.relative().to_str().ok_or(HostError::InvalidPath)?;
        if relative.is_empty()
            || relative.starts_with('/')
            || relative.contains(['\\', ':', '\0'])
            || relative
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(HostError::InvalidPath);
        }
        jai_source::normalize_virtual_path(self.vfs.root(), path.relative())
            .map_err(|_| HostError::InvalidPath)
    }
    fn fail(&mut self, error: HostError) -> HostError {
        if let Some(active) = self.active.as_mut() {
            active.failed = true;
        }
        error
    }
    fn reserve_observation(
        &self,
        operation_bytes: usize,
        response_bytes: usize,
    ) -> Result<usize, HostError> {
        // Cache, returned observation and staged request payload copies are
        // admitted before cloning. Read responses need cache + returned copies.
        operation_bytes
            .checked_mul(3)
            .and_then(|bytes| {
                response_bytes
                    .checked_mul(2)
                    .and_then(|response| bytes.checked_add(response))
            })
            .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<Observation>()))
            .and_then(|bytes| bytes.checked_add(self.retained_bytes))
            .filter(|bytes| *bytes <= self.limits.journal_bytes)
            .ok_or(HostError::Budget("VFS journal byte limit exceeded"))
    }
    fn observe(&mut self, operation: Operation) -> Result<Observation, HostError> {
        let active = self
            .active
            .as_ref()
            .ok_or(HostError::Transaction("VFS transaction is not active"))?;
        if active.failed {
            return Err(HostError::Transaction("VFS transaction already failed"));
        }
        let journal = &self.journals[&active.origin];
        let cached = journal.observations.get(active.cursor).cloned();
        if let Some(cached) = cached {
            if cached.operation != operation {
                return Err(self.fail(HostError::Transaction(
                    "VFS request stream changed during replay",
                )));
            }
            self.active.as_mut().unwrap().cursor += 1;
            return Ok(cached);
        }
        if journal.committed {
            return Err(self.fail(HostError::Transaction(
                "committed VFS source cannot issue new requests",
            )));
        }
        if active.cursor >= self.limits.requests_per_origin {
            return Err(self.fail(HostError::Budget("VFS request limit exceeded")));
        }
        let operation_bytes = match &operation {
            Operation::Request(HostRequest::WriteEntireFile {
                path,
                bytes,
            }) => bytes
                .capacity()
                .checked_add(path.relative().as_os_str().as_encoded_bytes().len())
                .ok_or(HostError::Budget("VFS request size overflow"))?,
            Operation::Request(
                HostRequest::ReadEntireFile(path) | HostRequest::ReadFileForOpen(path),
            ) => path.relative().as_os_str().as_encoded_bytes().len(),
            Operation::Console(output) => output.bytes.capacity(),
            _ => 0,
        };
        self.reserve_observation(operation_bytes, 0)
            .map_err(|error| self.fail(error))?;
        let response = match &operation {
            Operation::Request(
                HostRequest::ReadEntireFile(path) | HostRequest::ReadFileForOpen(path),
            ) => {
                let path = self.path(path).map_err(|error| self.fail(error))?;
                let active = self.active.as_ref().unwrap();
                let bytes = active
                    .writes
                    .get(&path)
                    .or_else(|| self.journals[&active.origin].snapshot.files.get(&path));
                self.reserve_observation(operation_bytes, bytes.map_or(0, |bytes| bytes.len()))?;
                match (&operation, bytes) {
                    (Operation::Request(HostRequest::ReadEntireFile(_)), Some(bytes)) => {
                        Some(HostResponse::FileBytes(bytes.to_vec()))
                    }
                    (Operation::Request(HostRequest::ReadEntireFile(_)), None) => {
                        return Err(
                            self.fail(HostError::Io("virtual file was not supplied".into()))
                        );
                    }
                    (_, Some(bytes)) => Some(HostResponse::FileOpen(FileOpenObservation::Bytes(
                        bytes.to_vec(),
                    ))),
                    (_, None) => Some(HostResponse::FileOpen(FileOpenObservation::Failed(
                        FileOpenFailure::NotFound,
                    ))),
                }
            }
            Operation::Request(HostRequest::WriteEntireFile {
                path, ..
            }) => {
                self.require(HostCapability::FileWrite).map_err(|_| {
                    self.fail(HostError::UnsupportedCapability(HostCapability::FileWrite))
                })?;
                self.path(path).map_err(|error| self.fail(error))?;
                Some(HostResponse::WriteStaged)
            }
            Operation::Request(HostRequest::RunProgram(_)) => {
                return Err(self.fail(HostError::UnsupportedCapability(HostCapability::Processes)));
            }
            Operation::Console(_) => {
                self.require(HostCapability::Console).map_err(|_| {
                    self.fail(HostError::UnsupportedCapability(HostCapability::Console))
                })?;
                None
            }
        };
        let response_bytes = match &response {
            Some(
                HostResponse::FileBytes(bytes)
                | HostResponse::FileOpen(FileOpenObservation::Bytes(bytes)),
            ) => bytes.len(),
            _ => 0,
        };
        let retained = self
            .reserve_observation(operation_bytes, response_bytes)
            .map_err(|error| self.fail(error))?;
        let observation = Observation {
            operation,
            response,
        };
        let active = self.active.as_mut().unwrap();
        self.journals
            .get_mut(&active.origin)
            .unwrap()
            .observations
            .push(observation.clone());
        active.cursor += 1;
        self.retained_bytes = retained;
        Ok(observation)
    }
    fn stage(&mut self, observation: &Observation) -> Result<(), HostError> {
        let active = self.active.as_ref().unwrap();
        if self.journals[&active.origin].committed {
            return Ok(());
        }
        match &observation.operation {
            Operation::Request(HostRequest::WriteEntireFile {
                path,
                bytes,
            }) => {
                let path = self.path(path)?;
                self.active
                    .as_mut()
                    .unwrap()
                    .writes
                    .insert(path, Arc::from(bytes.as_slice()));
            }
            Operation::Console(output) => {
                let active = self.active.as_mut().unwrap();
                let bytes = active
                    .console_bytes
                    .checked_add(output.bytes.len())
                    .filter(|bytes| *bytes <= self.limits.console_bytes)
                    .ok_or(HostError::Budget("VFS console limit exceeded"))?;
                active.console.push(output.clone());
                active.console_bytes = bytes;
            }
            _ => {}
        }
        Ok(())
    }
}
impl HostEffects for VfsHost {
    fn begin(&mut self, origin: SourceOrigin) -> Result<(), HostError> {
        if self.active.is_some() || self.parked.contains_key(&origin) {
            return Err(HostError::Transaction(
                "VFS source is already active or suspended",
            ));
        }
        if !self.journals.contains_key(&origin) {
            if self.journals.len() >= self.limits.origins {
                return Err(HostError::Budget("VFS origin limit exceeded"));
            }
            let origin_bytes = origin
                .path
                .capacity()
                .checked_add(origin.body.capacity())
                .and_then(|n| n.checked_add(origin.specialization.capacity()))
                .ok_or(HostError::Budget("VFS origin size overflow"))?;
            let (snapshot, bytes) = {
                let store = self
                    .vfs
                    .store
                    .lock()
                    .map_err(|_| HostError::Transaction("VFS store is poisoned"))?;
                let metadata = store
                    .files
                    .len()
                    .checked_mul(std::mem::size_of::<(PathBuf, Arc<[u8]>)>() + 64)
                    .ok_or(HostError::Budget("VFS snapshot metadata size overflow"))?;
                let bytes = origin_bytes
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(self.retained_bytes))
                    .and_then(|n| n.checked_add(store.bytes))
                    .and_then(|n| n.checked_add(metadata))
                    .and_then(|n| {
                        n.checked_add(
                            std::mem::size_of::<Journal>() + std::mem::size_of::<Transaction>(),
                        )
                    })
                    .filter(|n| *n <= self.limits.journal_bytes)
                    .ok_or(HostError::Budget("VFS retained snapshot limit exceeded"))?;
                // Admission and snapshot copying share the lock: an editor edit
                // cannot grow the image between checking its size and pinning it.
                let snapshot = VfsSnapshot {
                    root: self.vfs.root.clone(),
                    revision: store.revision,
                    files: store.files.clone(),
                    bytes: store.bytes,
                };
                (snapshot, bytes)
            };
            self.journals.insert(
                origin.clone(),
                Journal {
                    snapshot,
                    observations: vec![],
                    committed: false,
                },
            );
            self.retained_bytes = bytes;
        }
        self.active = Some(Transaction {
            origin,
            cursor: 0,
            writes: BTreeMap::new(),
            console: vec![],
            console_bytes: 0,
            failed: false,
        });
        Ok(())
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        let result = self
            .observe(Operation::Request(request))
            .and_then(|observation| {
                self.stage(&observation)?;
                observation
                    .response
                    .ok_or(HostError::Transaction("file request has no observation"))
            });
        match result {
            Ok(response) => HostOutcome::Ready(response),
            Err(error) => HostOutcome::Rejected(self.fail(error)),
        }
    }
    fn finish(&mut self, commit: bool) -> Result<(), HostError> {
        let active = self
            .active
            .take()
            .ok_or(HostError::Transaction("VFS transaction is not active"))?;
        if !commit {
            return Ok(());
        }
        let mut store = self
            .vfs
            .store
            .lock()
            .map_err(|_| HostError::Transaction("VFS store is poisoned"))?;
        // Validate every publication before changing either output or file bytes.
        let Some(publication) = self.publication(&active, &store)? else {
            return Ok(());
        };
        for (path, value) in active.writes {
            store.files.insert(path, value);
        }
        store.bytes = publication.bytes;
        store.revision = publication.revision;
        self.output.extend(active.console);
        self.output_bytes = publication.output_bytes;
        self.journals.get_mut(&active.origin).unwrap().committed = true;
        Ok(())
    }
}
impl HostServices for VfsHost {
    fn capabilities(&self) -> &[HostCapability] {
        &self.capabilities
    }
    fn file_scope(&self) -> Option<FilePathScope> {
        Some(self.scope.clone())
    }
    fn poll_host_request(&mut self, _: &SourceOrigin, _: HostRequestKey) -> HostOutcome {
        HostOutcome::Rejected(HostError::UnknownCapability)
    }
    fn suspend(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        let active = self
            .active
            .as_ref()
            .ok_or(HostError::Transaction("VFS transaction is not active"))?;
        if &active.origin != origin || active.failed || self.parked.contains_key(origin) {
            return Err(HostError::Transaction(
                "VFS suspension requires the exact active source",
            ));
        }
        self.parked
            .insert(origin.clone(), self.active.take().unwrap());
        Ok(())
    }
    fn resume(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        if self.active.is_some() {
            return Err(HostError::Transaction("another VFS transaction is active"));
        }
        self.active = Some(self.parked.remove(origin).ok_or(HostError::Transaction(
            "source has no suspended VFS transaction",
        ))?);
        Ok(())
    }
    fn cancel(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        self.parked.remove(origin).ok_or(HostError::Transaction(
            "source has no suspended VFS transaction",
        ))?;
        Ok(())
    }
    fn write_console(
        &mut self,
        stream: CompilerOutputStream,
        bytes: Vec<u8>,
    ) -> Result<(), PlatformError> {
        let result = self
            .observe(Operation::Console(ConsoleOutput {
                stream,
                bytes,
            }))
            .and_then(|observation| self.stage(&observation));
        result.map_err(|error| PlatformError::Host(self.fail(error)))
    }
}

pub struct VfsPlatform {
    snapshot: VfsSnapshot,
    host: VfsHost,
}
impl VfsPlatform {
    pub fn new(
        vfs: SharedVfs,
        working: &Path,
        writable: bool,
        console: bool,
        limits: VfsHostLimits,
    ) -> Result<Self, HostError> {
        let snapshot = vfs
            .snapshot()
            .map_err(|error| HostError::Io(error.to_string()))?;
        let host = VfsHost::new(vfs, working, writable, console, limits)?;
        Ok(Self {
            snapshot,
            host,
        })
    }
    pub fn snapshot(&self) -> &VfsSnapshot {
        &self.snapshot
    }
    pub fn host(&self) -> &VfsHost {
        &self.host
    }
    pub fn host_mut(&mut self) -> &mut VfsHost {
        &mut self.host
    }
}
impl Platform for VfsPlatform {
    fn source_files(&self) -> &dyn jai_source::SourceProvider {
        &self.snapshot
    }
    fn host_services(&mut self) -> &mut dyn HostServices {
        &mut self.host
    }
}
