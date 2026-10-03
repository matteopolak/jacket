//! A shared compiler/host transaction commits a validated compiler shadow first.
use super::*;
use crate::compiler_effects::{CompilerSession, SuspendedCompilerTransaction};
use jai_vm::{CompilerEffects, CompilerRequest, EffectOutcome, Error};
use jai_vm::{SourceOrigin, host_effects::*};
use std::{collections::HashMap, path::Path};

struct SuspendedFileTransaction {
    compiler: SuspendedCompilerTransaction,
    host: SuspendedHostTransaction,
}

pub struct FileCompilerSession {
    compiler: CompilerSession,
    host: HostIo,
    scope: FilePathScope,
    origin: Option<SourceOrigin>,
    begin_error: Option<HostError>,
    active_origin: Option<SourceOrigin>,
    parked: HashMap<SourceOrigin, SuspendedFileTransaction>,
}
impl FileCompilerSession {
    pub fn with_root(
        compiler: CompilerSession,
        host: HostIo,
        root: FileRootId,
        working_relative: &Path,
    ) -> Result<Self, HostError> {
        let scope = host.file_scope(root, working_relative)?;
        Ok(Self {
            compiler,
            host,
            scope,
            origin: None,
            begin_error: None,
            active_origin: None,
            parked: HashMap::new(),
        })
    }
    pub fn compiler(&self) -> &CompilerSession {
        &self.compiler
    }
    pub fn compiler_mut(&mut self) -> &mut CompilerSession {
        &mut self.compiler
    }
    pub fn host(&self) -> &HostIo {
        &self.host
    }
    pub fn host_mut(&mut self) -> &mut HostIo {
        &mut self.host
    }
    pub fn service_suspended(
        &mut self,
        origin: &SourceOrigin,
        key: HostRequestKey,
    ) -> Result<(), HostError> {
        let parked = self.parked.get(origin).ok_or(HostError::Transaction(
            "source has no suspended file transaction",
        ))?;
        self.host.service_suspended(&parked.host, key)
    }
}
impl CompilerEffects for FileCompilerSession {
    fn set_source_origin(&mut self, origin: SourceOrigin) {
        self.compiler.set_source_origin(origin.clone());
        self.origin = Some(origin);
    }
    fn begin(&mut self) {
        self.compiler.begin();
        self.active_origin = self.origin.clone();
        self.begin_error = match self.origin.clone() {
            Some(origin) => self.host.begin(origin).err(),
            None => Some(HostError::Transaction(
                "host transaction requires a source identity",
            )),
        };
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        self.compiler.request(request)
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: jai_vm::EffectKey) -> EffectOutcome {
        self.compiler.poll_request(request, key)
    }
    fn host_request(&mut self, request: HostRequest) -> HostOutcome {
        match &self.begin_error {
            Some(error) => HostOutcome::Rejected(error.clone()),
            None => self.host.request(request),
        }
    }
    fn host_file_scope(&self) -> Option<FilePathScope> {
        Some(self.scope.clone())
    }
    fn poll_host_request(&mut self, key: HostRequestKey) -> HostOutcome {
        let Some(origin) = self.origin.as_ref() else {
            return HostOutcome::Rejected(HostError::Transaction(
                "completion requires a source identity",
            ));
        };
        if self.active_origin.as_ref() != Some(origin) && !self.parked.contains_key(origin) {
            return HostOutcome::Rejected(HostError::Denied(
                "completion requires the matching live file transaction",
            ));
        }
        self.host
            .completion(origin, key)
            .unwrap_or_else(HostOutcome::Rejected)
    }
    fn service_pending(&mut self, dependencies: &[jai_vm::Dependency]) -> Result<bool, Error> {
        let keys: Vec<_> = dependencies
            .iter()
            .filter_map(|dependency| match dependency {
                jai_vm::Dependency::Host(key) => Some(*key),
                _ => None,
            })
            .collect();
        if keys.is_empty() {
            return self.compiler.service_pending(dependencies);
        }
        let error = |error: HostError| Error::EffectRejected(error.to_string());
        let origin = self
            .origin
            .as_ref()
            .ok_or_else(|| error(HostError::Transaction("service requires a source identity")))?;
        let parked = self.parked.get(origin).ok_or_else(|| {
            error(HostError::Transaction(
                "service requires the matching parked source",
            ))
        })?;
        // Validate the complete dependency list before launching anything. A
        // foreign or stale ticket cannot partially service this source job.
        let mut pending = Vec::new();
        for key in keys {
            match self.host.completion(origin, key).map_err(error)? {
                HostOutcome::Pending(actual) if actual == key => {
                    if !pending.contains(&key) {
                        pending.push(key);
                    }
                }
                HostOutcome::Pending(_) => {
                    return Err(error(HostError::Transaction("readiness ticket changed")));
                }
                HostOutcome::Ready(_) | HostOutcome::Rejected(_) => {}
            }
        }
        for key in pending {
            self.host
                .service_suspended(&parked.host, key)
                .map_err(error)?;
        }
        // Cached terminal observations are also ready: the VM must resume its
        // pending leaf to consume either the response or the boundary failure.
        Ok(true)
    }
    fn suspend(&mut self) -> Result<(), Error> {
        let error = Error::EffectRejected;
        let origin = self
            .active_origin
            .clone()
            .ok_or_else(|| error("file transaction is not active".into()))?;
        if self.origin.as_ref() != Some(&origin)
            || self.parked.contains_key(&origin)
            || self.begin_error.is_some()
        {
            return Err(error(
                "file transaction source identity cannot suspend".into(),
            ));
        }
        let compiler = self
            .compiler
            .suspend_transaction()
            .map_err(|e| error(e.to_string()))?;
        let host = match self.host.suspend_transaction() {
            Ok(host) => host,
            Err(host_error) => {
                self.compiler
                    .resume_transaction(compiler)
                    .map_err(|e| error(e.to_string()))?;
                return Err(error(host_error.to_string()));
            }
        };
        self.parked
            .insert(origin, SuspendedFileTransaction { compiler, host });
        self.active_origin = None;
        Ok(())
    }
    fn resume(&mut self) -> Result<(), Error> {
        let error = Error::EffectRejected;
        if self.active_origin.is_some() {
            return Err(error("another file transaction is active".into()));
        }
        let origin = self
            .origin
            .clone()
            .ok_or_else(|| error("resume requires a source identity".into()))?;
        let parked = self
            .parked
            .remove(&origin)
            .ok_or_else(|| error("source has no suspended file transaction".into()))?;
        if let Err(compiler_error) = self.compiler.resume_transaction(parked.compiler) {
            self.host
                .cancel_suspended(parked.host)
                .map_err(|host_error| error(host_error.to_string()))?;
            return Err(error(compiler_error.to_string()));
        }
        if let Err(host_error) = self.host.resume_transaction(parked.host) {
            let _ = self.compiler.finish(false);
            return Err(error(host_error.to_string()));
        }
        self.active_origin = Some(origin);
        self.begin_error = None;
        Ok(())
    }
    fn finish(&mut self, commit: bool) -> Result<(), Error> {
        if !commit
            && let Some(parked) = self
                .origin
                .as_ref()
                .and_then(|origin| self.parked.remove(origin))
        {
            self.host
                .cancel_suspended(parked.host)
                .map_err(|error| Error::EffectRejected(error.to_string()))?;
            return Ok(());
        }
        self.active_origin = None;
        if !commit {
            let compiler = self.compiler.finish(false);
            if self.begin_error.take().is_none() {
                let _ = self.host.finish(false);
            }
            return compiler;
        }
        if let Some(error) = self.begin_error.take() {
            let _ = self.compiler.finish(false);
            return Err(Error::EffectRejected(error.to_string()));
        }
        // Compiler finish may reject lifecycle/output state. Validate on the same
        // logical session before publishing the sole host file atomic rename.
        let mut shadow = self.compiler.clone();
        if let Err(error) = shadow.finish(true) {
            let _ = self.host.finish(false);
            let _ = self.compiler.finish(false);
            return Err(error);
        }
        if let Err(error) = self.host.finish(true) {
            let _ = self.compiler.finish(false);
            return Err(Error::EffectRejected(error.to_string()));
        }
        self.compiler = shadow;
        Ok(())
    }
}
