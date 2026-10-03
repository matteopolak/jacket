//! Native platform services retain the existing original-input and process policies.
use super::*;
use jai_platform::{
    ConsoleOutput, HostCapability, HostServices, PlatformError, SharedVfs, VfsHost, VfsHostLimits,
    VfsLimits,
};
use jai_vm::CompilerOutputStream;

pub struct NativeHostServices {
    host: HostIo,
    scope: FilePathScope,
    capabilities: Vec<HostCapability>,
    console: VfsHost,
    active: Option<SourceOrigin>,
    parked: HashMap<SourceOrigin, SuspendedHostTransaction>,
}
impl NativeHostServices {
    pub fn new(
        host: HostIo,
        root: FileRootId,
        working: &Path,
        console_limits: VfsHostLimits,
    ) -> Result<Self, HostError> {
        if host.transaction.is_some()
            || !host.observations.is_empty()
            || host
                .parked
                .values()
                .any(|lifetime| lifetime.strong_count() != 0)
        {
            return Err(HostError::Transaction(
                "platform attachment must precede native source observations",
            ));
        }
        let scope = host.file_scope(root, working)?;
        let mut capabilities = vec![HostCapability::FileRead, HostCapability::Console];
        if host.roots.get(&root).is_some_and(|root| root.writable) {
            capabilities.push(HostCapability::FileWrite);
        }
        if !host.programs.is_empty() {
            capabilities.push(HostCapability::Processes);
        }
        // This private memory namespace carries console transactions only. It is
        // never exposed as a file grant or substituted for native host file IO.
        let vfs = SharedVfs::new("/jai-native-console", VfsLimits::default()).map_err(io)?;
        let console = VfsHost::new(vfs, Path::new(""), false, true, console_limits)?;
        Ok(Self {
            host,
            scope,
            capabilities,
            console,
            active: None,
            parked: HashMap::new(),
        })
    }
    pub fn host(&self) -> &HostIo {
        &self.host
    }
    pub fn take_console(&mut self) -> Vec<ConsoleOutput> {
        self.console.take_console()
    }
}
impl HostEffects for NativeHostServices {
    fn begin(&mut self, origin: SourceOrigin) -> Result<(), HostError> {
        self.host.begin(origin.clone())?;
        if let Err(failure) = self.console.begin(origin.clone()) {
            let _ = self.host.finish(false);
            return Err(failure);
        }
        self.active = Some(origin);
        Ok(())
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        self.host.request(request)
    }
    fn finish(&mut self, commit: bool) -> Result<(), HostError> {
        self.active = None;
        if !commit {
            let host = self.host.finish(false);
            let console = self.console.finish(false);
            return host.and(console);
        }
        if let Err(failure) = self.console.validate_commit() {
            let _ = self.host.finish(false);
            let _ = self.console.finish(false);
            return Err(failure);
        }
        if let Err(failure) = self.host.finish(true) {
            let _ = self.console.finish(false);
            return Err(failure);
        }
        // The private console has no external writer/poisoning surface; its
        // in-memory publication was validated before the native atomic rename.
        self.console.finish(true)
    }
}
impl HostServices for NativeHostServices {
    fn capabilities(&self) -> &[HostCapability] {
        &self.capabilities
    }
    fn file_scope(&self) -> Option<FilePathScope> {
        Some(self.scope.clone())
    }
    fn poll_host_request(&mut self, origin: &SourceOrigin, key: HostRequestKey) -> HostOutcome {
        if self.active.as_ref() != Some(origin) && !self.parked.contains_key(origin) {
            return HostOutcome::Rejected(HostError::Denied(
                "native completion requires matching live platform journal",
            ));
        }
        self.host
            .completion(origin, key)
            .unwrap_or_else(HostOutcome::Rejected)
    }
    fn service_host_request(
        &mut self,
        origin: &SourceOrigin,
        key: HostRequestKey,
    ) -> Result<bool, HostError> {
        let token = self.parked.get(origin).ok_or(HostError::Transaction(
            "native service requires exact suspended platform journal",
        ))?;
        self.host.service_suspended(token, key)?;
        Ok(true)
    }
    fn suspend(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        if self.active.as_ref() != Some(origin) || self.parked.contains_key(origin) {
            return Err(HostError::Transaction(
                "native suspension requires exact active platform journal",
            ));
        }
        let token = self.host.suspend_transaction()?;
        if let Err(failure) = self.console.suspend(origin) {
            self.host.resume_transaction(token)?;
            return Err(failure);
        }
        self.parked.insert(origin.clone(), token);
        self.active = None;
        Ok(())
    }
    fn resume(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        if self.active.is_some() {
            return Err(HostError::Transaction(
                "another native platform journal is active",
            ));
        }
        let token = self.parked.remove(origin).ok_or(HostError::Transaction(
            "source has no suspended native platform journal",
        ))?;
        if let Err(failure) = self.host.resume_transaction(token) {
            let _ = self.console.cancel(origin);
            return Err(failure);
        }
        if let Err(failure) = self.console.resume(origin) {
            let _ = self.host.finish(false);
            return Err(failure);
        }
        self.active = Some(origin.clone());
        Ok(())
    }
    fn cancel(&mut self, origin: &SourceOrigin) -> Result<(), HostError> {
        let token = self.parked.remove(origin).ok_or(HostError::Transaction(
            "source has no suspended native platform journal",
        ))?;
        let host = self.host.cancel_suspended(token);
        let console = self.console.cancel(origin);
        host.and(console)
    }
    fn write_console(
        &mut self,
        stream: CompilerOutputStream,
        bytes: Vec<u8>,
    ) -> Result<(), PlatformError> {
        self.console.write_console(stream, bytes)
    }
}
