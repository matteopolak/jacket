//! One compiler journal paired with the embedding's actual platform host journal.
use crate::{CompilerSession, SuspendedCompilerTransaction};
use jai_platform::HostServices;
use jai_vm::{
    CompilerEffects, CompilerRequest, EffectOutcome, Error, SourceOrigin, host_effects::*,
};
use std::collections::HashMap;

pub struct PlatformCompilerSession<'a> {
    compiler: CompilerSession,
    host: &'a mut dyn HostServices,
    origin: Option<SourceOrigin>,
    active: Option<SourceOrigin>,
    begin_error: Option<HostError>,
    parked: HashMap<SourceOrigin, SuspendedCompilerTransaction>,
}
impl<'a> PlatformCompilerSession<'a> {
    pub fn new(compiler: CompilerSession, host: &'a mut dyn HostServices) -> Self {
        Self {
            compiler,
            host,
            origin: None,
            active: None,
            begin_error: None,
            parked: HashMap::new(),
        }
    }
    pub fn compiler(&self) -> &CompilerSession {
        &self.compiler
    }
    pub fn compiler_mut(&mut self) -> &mut CompilerSession {
        &mut self.compiler
    }
}
impl CompilerEffects for PlatformCompilerSession<'_> {
    fn set_source_origin(&mut self, origin: SourceOrigin) {
        self.compiler.set_source_origin(origin.clone());
        self.origin = Some(origin);
    }
    fn begin(&mut self) {
        self.compiler.begin();
        self.active = self.origin.clone();
        self.begin_error = match self.origin.clone() {
            Some(origin) => self.host.begin(origin).err(),
            None => Some(HostError::Transaction(
                "platform journal requires an actual source identity",
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
        self.host.file_scope()
    }
    fn poll_host_request(&mut self, key: HostRequestKey) -> HostOutcome {
        let Some(origin) = self.origin.as_ref() else {
            return HostOutcome::Rejected(HostError::Transaction(
                "platform completion requires a source identity",
            ));
        };
        if self.active.as_ref() != Some(origin) && !self.parked.contains_key(origin) {
            return HostOutcome::Rejected(HostError::Denied(
                "platform completion requires the matching live journal",
            ));
        }
        self.host.poll_host_request(origin, key)
    }
    fn service_pending(&mut self, dependencies: &[jai_vm::Dependency]) -> Result<bool, Error> {
        let mut host_ready = false;
        let mut pending = Vec::new();
        for dependency in dependencies {
            if let jai_vm::Dependency::Host(key) = dependency {
                match self.poll_host_request(*key) {
                    HostOutcome::Pending(actual) if actual == *key => {
                        if !pending.contains(key) {
                            pending.push(*key);
                        }
                    }
                    HostOutcome::Pending(_) => {
                        return Err(Error::EffectRejected(
                            "platform readiness ticket changed".into(),
                        ));
                    }
                    HostOutcome::Rejected(HostError::UnknownCapability | HostError::Denied(_)) => {
                        return Err(Error::EffectRejected(
                            "platform readiness ticket has no matching journal".into(),
                        ));
                    }
                    HostOutcome::Ready(_) | HostOutcome::Rejected(_) => host_ready = true,
                }
            }
        }
        // Validate every key before invoking a provider that can launch a process.
        if !pending.is_empty() {
            let origin = self.origin.as_ref().ok_or_else(|| {
                Error::EffectRejected("platform service requires an actual source identity".into())
            })?;
            if !self.parked.contains_key(origin) {
                return Err(Error::EffectRejected(
                    "platform service requires the matching parked journal".into(),
                ));
            }
            for key in pending {
                host_ready |= self
                    .host
                    .service_host_request(origin, key)
                    .map_err(|failure| Error::EffectRejected(failure.to_string()))?;
            }
        }
        if host_ready {
            Ok(true)
        } else {
            self.compiler.service_pending(dependencies)
        }
    }
    fn suspend(&mut self) -> Result<(), Error> {
        let error = |error: HostError| Error::EffectRejected(error.to_string());
        let origin = self
            .active
            .clone()
            .ok_or_else(|| error(HostError::Transaction("platform transaction is not active")))?;
        if self.origin.as_ref() != Some(&origin)
            || self.begin_error.is_some()
            || self.parked.contains_key(&origin)
        {
            return Err(error(HostError::Transaction(
                "platform suspension requires the exact source journal",
            )));
        }
        let compiler = self
            .compiler
            .suspend_transaction()
            .map_err(|failure| Error::EffectRejected(failure.to_string()))?;
        if let Err(failure) = self.host.suspend(&origin) {
            self.compiler
                .resume_transaction(compiler)
                .map_err(|failure| Error::EffectRejected(failure.to_string()))?;
            return Err(error(failure));
        }
        self.parked.insert(origin, compiler);
        self.active = None;
        Ok(())
    }
    fn resume(&mut self) -> Result<(), Error> {
        let error = |error: HostError| Error::EffectRejected(error.to_string());
        if self.active.is_some() {
            return Err(error(HostError::Transaction(
                "another platform transaction is active",
            )));
        }
        let origin = self.origin.clone().ok_or_else(|| {
            error(HostError::Transaction(
                "platform resume requires source identity",
            ))
        })?;
        let compiler = self.parked.remove(&origin).ok_or_else(|| {
            error(HostError::Transaction(
                "source has no suspended platform journal",
            ))
        })?;
        if let Err(failure) = self.compiler.resume_transaction(compiler) {
            let _ = self.host.cancel(&origin);
            return Err(Error::EffectRejected(failure.to_string()));
        }
        if let Err(failure) = self.host.resume(&origin) {
            let _ = self.compiler.finish(false);
            return Err(error(failure));
        }
        self.active = Some(origin);
        self.begin_error = None;
        Ok(())
    }
    fn finish(&mut self, commit: bool) -> Result<(), Error> {
        if !commit
            && let Some(origin) = self.origin.as_ref()
            && self.parked.remove(origin).is_some()
        {
            self.host
                .cancel(origin)
                .map_err(|failure| Error::EffectRejected(failure.to_string()))?;
            return Ok(());
        }
        if commit && (self.active.is_none() || self.active.as_ref() != self.origin.as_ref()) {
            let _ = self.host.finish(false);
            let _ = self.compiler.finish(false);
            self.active = None;
            return Err(Error::EffectRejected(
                "platform publication requires the exact active source identity".into(),
            ));
        }
        self.active = None;
        if !commit {
            let result = self.compiler.finish(false);
            if self.begin_error.take().is_none() {
                let _ = self.host.finish(false);
            }
            return result;
        }
        if let Some(failure) = self.begin_error.take() {
            let _ = self.compiler.finish(false);
            return Err(Error::EffectRejected(failure.to_string()));
        }
        let previous_outputs = self.compiler.clone().take_outputs().len();
        let mut shadow = self.compiler.clone();
        if let Err(failure) = shadow.finish(true) {
            let _ = self.host.finish(false);
            let _ = self.compiler.finish(false);
            return Err(failure);
        }
        // The compiler shadow validates output/lifecycle before platform publication.
        // Inspect a separate clone so draining the preview cannot mutate its revision.
        for output in shadow
            .clone()
            .take_outputs()
            .into_iter()
            .skip(previous_outputs)
        {
            if let Err(failure) = self.host.write_console(output.stream, output.bytes) {
                let _ = self.host.finish(false);
                let _ = self.compiler.finish(false);
                return Err(Error::EffectRejected(failure.to_string()));
            }
        }
        if let Err(failure) = self.host.finish(true) {
            let _ = self.compiler.finish(false);
            return Err(Error::EffectRejected(failure.to_string()));
        }
        self.compiler = shadow;
        Ok(())
    }
}
