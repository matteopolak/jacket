//! Replay committed source transactions without repeating compiler mutations.
mod completed_forks;
use crate::{
    CompilerJobId, CompilerSession, CompilerTransactionError, SuspendedCompilerTransaction,
};
use jai_vm::{
    CompilerEffects, CompilerEvent, CompilerRequest, CompilerResponse, EffectKey, EffectOutcome,
    SourceOrigin,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy, Debug)]
pub struct ReplayLimits {
    pub runs: usize,
    pub requests_per_run: usize,
    pub bytes: usize,
}
impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            runs: 10_000,
            requests_per_run: 10_000,
            bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct RecordedRequest {
    request: CompilerRequest,
    response: CompilerResponse,
}

/// This cache belongs to one build session; it contains no TypeId or ProcedureId.
pub struct EffectReplayCache {
    identity: Arc<()>,
    branch_parent: Option<Arc<()>>,
    runs: HashMap<SourceOrigin, Arc<[RecordedRequest]>>,
    suspended: HashMap<SourceOrigin, SuspendedRun>,
    suspended_bytes: usize,
    bytes: usize,
    limits: ReplayLimits,
}
impl Default for EffectReplayCache {
    fn default() -> Self {
        Self::new(ReplayLimits::default())
    }
}
impl EffectReplayCache {
    pub fn new(limits: ReplayLimits) -> Self {
        Self {
            identity: Arc::new(()),
            branch_parent: None,
            runs: HashMap::new(),
            suspended: HashMap::new(),
            suspended_bytes: 0,
            bytes: 0,
            limits,
        }
    }
    pub fn len(&self) -> usize {
        self.runs.len()
    }
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn suspended_jobs(&self) -> usize {
        self.suspended.len()
    }
    pub(crate) fn suspended_origins(&self) -> impl Iterator<Item = &SourceOrigin> {
        self.suspended.keys()
    }
    pub fn suspended_job(&self, origin: &SourceOrigin) -> Option<CompilerJobId> {
        self.suspended
            .get(origin)?
            .compiler
            .as_ref()
            .map(SuspendedCompilerTransaction::id)
    }
    pub fn preview_suspended(
        &self,
        origin: &SourceOrigin,
        session: &CompilerSession,
    ) -> Result<CompilerSession, jai_vm::Error> {
        let parked = self.suspended.get(origin).ok_or_else(|| {
            jai_vm::Error::EffectRejected("no suspended compiler job for this source origin".into())
        })?;
        let compiler = parked.compiler.as_ref().ok_or_else(|| {
            jai_vm::Error::EffectRejected(
                "a replay-only job has no compiler staging to preview".into(),
            )
        })?;
        session
            .validate_suspended(compiler)
            .map_err(|error| jai_vm::Error::EffectRejected(error.to_string()))?;
        if let Some(preview) = &parked.preview {
            return Ok(preview.clone());
        }
        session.preview_transaction(compiler)
    }
    pub fn prepare_suspended_preview(
        &mut self,
        origin: &SourceOrigin,
        preview: CompilerSession,
    ) -> Result<(), CompilerTransactionError> {
        let parked = self
            .suspended
            .get_mut(origin)
            .ok_or(CompilerTransactionError::Inactive)?;
        let compiler = parked
            .compiler
            .as_ref()
            .ok_or(CompilerTransactionError::Inactive)?;
        preview.validate_preview_for(compiler)?;
        parked.preview = Some(preview);
        Ok(())
    }
    pub fn suspended_workspaces(&self, origin: &SourceOrigin) -> Vec<jai_vm::WorkspaceId> {
        self.suspended
            .get(origin)
            .and_then(|parked| parked.compiler.as_ref())
            .map(|compiler| compiler.intercepted_workspaces().collect())
            .unwrap_or_default()
    }
    pub fn publish_suspended_event(
        &mut self,
        origin: &SourceOrigin,
        event: CompilerEvent,
    ) -> Result<bool, jai_vm::Error> {
        let parked = self.suspended.get_mut(origin).ok_or_else(|| {
            jai_vm::Error::EffectRejected("no suspended compiler event receiver".into())
        })?;
        if parked.emitted.contains(&event) {
            return Ok(false);
        }
        parked.emitted.try_reserve(1).map_err(|_| {
            jai_vm::Error::EffectRejected("compiler event history allocation failed".into())
        })?;
        parked
            .compiler
            .as_mut()
            .ok_or_else(|| {
                jai_vm::Error::EffectRejected(
                    "replay-only job has no compiler event receiver".into(),
                )
            })?
            .publish_event(event.clone())
            .map_err(|error| jai_vm::Error::EffectRejected(error.to_string()))?;
        // Only newly published events make a stalled scheduler job runnable.
        // A later successful source/settings mutation removes this readiness.
        // The event contains owned typed data, never a pointer into another VM.
        parked.emitted.push(event);
        Ok(true)
    }
    pub fn fork_committed(&self) -> Self {
        Self {
            identity: Arc::new(()),
            branch_parent: Some(Arc::clone(&self.identity)),
            runs: self.runs.clone(),
            bytes: self.bytes,
            suspended: HashMap::new(),
            suspended_bytes: 0,
            limits: self.limits,
        }
    }
    /// Fork only committed traces and this job's privately completed children.
    /// Other suspended jobs and their pending effect tickets never cross over.
    pub fn fork_suspended_replay(&self, origin: &SourceOrigin) -> Result<Self, jai_vm::Error> {
        let parked = self.suspended.get(origin).ok_or_else(|| {
            jai_vm::Error::EffectRejected("no suspended compiler job for replay preview".into())
        })?;
        let mut branch = self.fork_committed();
        if let Some(prepared) = &parked.branch {
            branch.runs.extend(prepared.runs.clone());
            branch.bytes = branch.bytes.checked_add(prepared.bytes).ok_or_else(|| {
                jai_vm::Error::EffectRejected("compiler replay preview byte limit exceeded".into())
            })?;
        }
        Ok(branch)
    }
    /// Keep a completed child's host preview and replay traces under the
    /// parent's sealed job. Neither is published before the parent commits.
    pub fn prepare_suspended_preview_with_replay(
        &mut self,
        origin: &SourceOrigin,
        preview: CompilerSession,
        branch: Self,
    ) -> Result<(), jai_vm::Error> {
        if !branch
            .branch_parent
            .as_ref()
            .is_some_and(|parent| Arc::ptr_eq(parent, &self.identity))
        {
            return Err(jai_vm::Error::EffectRejected(
                "compiler replay preview belongs to another cache".into(),
            ));
        }
        if !branch.suspended.is_empty() {
            return Err(jai_vm::Error::EffectRejected(
                "compiler replay preview has unfinished child jobs".into(),
            ));
        }
        let mut added = HashMap::new();
        let mut bytes = 0usize;
        for (key, requests) in branch.runs {
            if let Some(committed) = self.runs.get(&key) {
                if committed.as_ref() != requests.as_ref() {
                    return Err(jai_vm::Error::EffectRejected(
                        "compiler replay preview changed a committed child trace".into(),
                    ));
                }
            } else {
                bytes = bytes
                    .checked_add(estimate(&key, &requests))
                    .ok_or_else(|| {
                        jai_vm::Error::EffectRejected(
                            "compiler replay preview byte limit exceeded".into(),
                        )
                    })?;
                added.insert(key, requests);
            }
        }
        let retained_runs = self.retained_runs();
        let parked = self.suspended.get_mut(origin).ok_or_else(|| {
            jai_vm::Error::EffectRejected("no suspended compiler job for replay preview".into())
        })?;
        let compiler = parked.compiler.as_ref().ok_or_else(|| {
            jai_vm::Error::EffectRejected("replay-only job has no compiler preview".into())
        })?;
        preview
            .validate_preview_for(compiler)
            .map_err(|error| jai_vm::Error::EffectRejected(error.to_string()))?;
        let old = parked.branch.as_ref().map_or(0, |branch| branch.bytes);
        let old_runs = parked.branch.as_ref().map_or(0, |branch| branch.runs.len());
        let suspended_bytes = self
            .suspended_bytes
            .checked_sub(old)
            .and_then(|count| count.checked_add(bytes));
        let total = suspended_bytes.and_then(|count| count.checked_add(self.bytes));
        let runs = retained_runs
            .checked_sub(old_runs)
            .and_then(|count| count.checked_add(added.len()));
        if total.is_none_or(|count| count > self.limits.bytes)
            || runs.is_none_or(|count| count > self.limits.runs)
        {
            return Err(jai_vm::Error::EffectRejected(
                "compile-time child replay cache limit exceeded".into(),
            ));
        }
        parked.bytes = parked.bytes - old + bytes;
        parked.branch = Some(ReplayBranch { runs: added, bytes });
        parked.preview = Some(preview);
        self.suspended_bytes = suspended_bytes.expect("checked suspended byte limit");
        Ok(())
    }
    fn retained_runs(&self) -> usize {
        self.suspended.values().fold(self.runs.len(), |count, job| {
            count
                .saturating_add(1)
                .saturating_add(job.branch.as_ref().map_or(0, |branch| branch.runs.len()))
        })
    }
}

struct ReplayBranch {
    runs: HashMap<SourceOrigin, Arc<[RecordedRequest]>>,
    bytes: usize,
}

struct SuspendedRun {
    transaction: Transaction,
    compiler: Option<SuspendedCompilerTransaction>,
    preview: Option<CompilerSession>,
    bytes: usize,
    pending: Option<(CompilerRequest, EffectKey)>,
    emitted: Vec<CompilerEvent>,
    branch: Option<ReplayBranch>,
}

enum Transaction {
    Idle,
    Recording(Vec<RecordedRequest>),
    Replaying {
        requests: Arc<[RecordedRequest]>,
        next: usize,
    },
    Rejected(String),
}

/// The VM validates finish before publishing its memory or compiler transaction.
pub struct ReplayEffects<'a> {
    session: &'a mut CompilerSession,
    cache: &'a mut EffectReplayCache,
    origin: Option<SourceOrigin>,
    active_origin: Option<SourceOrigin>,
    pending: Option<(CompilerRequest, EffectKey)>,
    emitted: Vec<CompilerEvent>,
    branch: Option<ReplayBranch>,
    transaction: Transaction,
}
impl<'a> ReplayEffects<'a> {
    pub fn new(session: &'a mut CompilerSession, cache: &'a mut EffectReplayCache) -> Self {
        Self {
            session,
            cache,
            origin: None,
            active_origin: None,
            pending: None,
            emitted: vec![],
            branch: None,
            transaction: Transaction::Idle,
        }
    }
    pub(crate) fn service_suspended(
        &mut self,
        service: impl FnOnce(
            &mut CompilerSession,
            &mut EffectReplayCache,
            &SourceOrigin,
        ) -> Result<bool, jai_vm::Error>,
    ) -> Result<bool, jai_vm::Error> {
        if !matches!(self.transaction, Transaction::Idle) {
            return Err(jai_vm::Error::EffectRejected(
                "compiler dependency service requires a parked transaction".into(),
            ));
        }
        let origin = self
            .origin
            .as_ref()
            .filter(|origin| self.cache.suspended.contains_key(*origin))
            .ok_or_else(|| {
                jai_vm::Error::EffectRejected(
                    "compiler dependency service requires this source's retained job".into(),
                )
            })?;
        service(self.session, self.cache, origin)
    }
    fn reject(&mut self, reason: String) -> EffectOutcome {
        self.transaction = Transaction::Rejected(reason.clone());
        EffectOutcome::Rejected(reason)
    }
    fn commit_recording(&mut self, requests: Vec<RecordedRequest>) -> Result<(), jai_vm::Error> {
        let branch = self.branch.take();
        if requests.is_empty() && branch.is_none() || self.origin.is_none() && branch.is_none() {
            return self.session.finish(true);
        }
        let Some(origin) = self.origin.as_ref() else {
            self.session.finish(false)?;
            return Err(jai_vm::Error::EffectRejected(
                "compiler child replay requires a trusted parent origin".into(),
            ));
        };
        let mut added = branch.map_or_else(HashMap::new, |branch| branch.runs);
        if added.contains_key(origin) || self.cache.runs.contains_key(origin) {
            self.session.finish(false)?;
            return Err(jai_vm::Error::EffectRejected(
                "compiler replay commit conflicts with its parent trace".into(),
            ));
        }
        for (key, requests) in &added {
            if self
                .cache
                .runs
                .get(key)
                .is_some_and(|committed| committed.as_ref() != requests.as_ref())
            {
                self.session.finish(false)?;
                return Err(jai_vm::Error::EffectRejected(
                    "compiler replay commit changed a committed child trace".into(),
                ));
            }
        }
        added.retain(|key, _| !self.cache.runs.contains_key(key));
        if !requests.is_empty() {
            added.insert(origin.clone(), requests.into());
        }
        let bytes = added.iter().try_fold(0usize, |total, (origin, requests)| {
            total.checked_add(estimate(origin, requests))
        });
        let total = bytes.and_then(|bytes| self.cache.bytes.checked_add(bytes));
        let retained = total.and_then(|total| total.checked_add(self.cache.suspended_bytes));
        let runs = self.cache.retained_runs().checked_add(added.len());
        if retained.is_none_or(|total| total > self.cache.limits.bytes)
            || runs.is_none_or(|count| count > self.cache.limits.runs)
        {
            self.session.finish(false)?;
            return Err(jai_vm::Error::EffectRejected(
                "compile-time effect replay cache limit exceeded".into(),
            ));
        }
        if self.cache.runs.try_reserve(added.len()).is_err() {
            self.session.finish(false)?;
            return Err(jai_vm::Error::EffectRejected(
                "compile-time effect replay cache allocation failed".into(),
            ));
        }
        // Reserve and validate the complete replay delta before host commit.
        // Publication below is infallible and occurs at the same success boundary.
        self.session.finish(true)?;
        self.cache.bytes = total.expect("checked cache size");
        self.cache.runs.extend(added);
        Ok(())
    }
}
impl CompilerEffects for ReplayEffects<'_> {
    fn set_source_origin(&mut self, origin: SourceOrigin) {
        self.origin = Some(origin);
    }
    fn begin(&mut self) {
        self.active_origin = self.origin.clone();
        self.pending = None;
        self.emitted.clear();
        self.branch = None;
        if self
            .origin
            .as_ref()
            .is_some_and(|origin| self.cache.suspended.contains_key(origin))
        {
            self.transaction = Transaction::Rejected(
                "suspended compiler job must resume its retained continuation".into(),
            );
            return;
        }
        // Starting another VM transaction abandons unfinished host staging,
        // including when the new transaction comes entirely from the cache.
        if let Err(error) = self.session.finish(false) {
            self.transaction = Transaction::Rejected(error.to_string());
            return;
        }
        if let Some(origin) = &self.origin {
            if self.session.workspace(origin.workspace).is_none() {
                self.transaction = Transaction::Rejected(
                    "compile-time origin belongs to another compiler session".into(),
                );
                return;
            }
            if let Some(requests) = self.cache.runs.get(origin) {
                self.transaction = Transaction::Replaying {
                    requests: Arc::clone(requests),
                    next: 0,
                };
                return;
            }
        }
        if let Some(origin) = &self.origin {
            self.session.begin_from_workspace(origin.workspace);
        } else {
            self.session.begin();
        }
        self.transaction = Transaction::Recording(vec![]);
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        if self.active_origin != self.origin {
            return Err(jai_vm::Error::EffectRejected(
                "compiler source origin changed during an active job".into(),
            ));
        }
        let origin = self.origin.clone().ok_or_else(|| {
            jai_vm::Error::EffectRejected(
                "compiler suspension requires a trusted source origin".into(),
            )
        })?;
        if self.cache.suspended.contains_key(&origin) {
            return Err(jai_vm::Error::EffectRejected(
                "this compiler source job is already suspended".into(),
            ));
        }
        let stream_bytes = match &self.transaction {
            Transaction::Recording(requests) => estimate(&origin, requests),
            Transaction::Replaying { requests, .. } => estimate(&origin, requests),
            Transaction::Idle | Transaction::Rejected(_) => {
                return Err(jai_vm::Error::EffectRejected(
                    "compiler suspension requires an active transaction".into(),
                ));
            }
        };
        let bytes = stream_bytes
            .checked_add(self.branch.as_ref().map_or(0, |branch| branch.bytes))
            .ok_or_else(|| {
                jai_vm::Error::EffectRejected(
                    "compile-time suspended replay byte limit exceeded".into(),
                )
            })?;
        let total = self
            .cache
            .bytes
            .checked_add(self.cache.suspended_bytes)
            .and_then(|total| total.checked_add(bytes));
        let runs = self.cache.retained_runs().checked_add(1).and_then(|count| {
            count.checked_add(self.branch.as_ref().map_or(0, |branch| branch.runs.len()))
        });
        if runs.is_none_or(|count| count > self.cache.limits.runs)
            || total.is_none_or(|total| total > self.cache.limits.bytes)
        {
            return Err(jai_vm::Error::EffectRejected(
                "compile-time suspended replay cache limit exceeded".into(),
            ));
        }
        self.cache.suspended.try_reserve(1).map_err(|_| {
            jai_vm::Error::EffectRejected("compiler suspended job allocation failed".into())
        })?;
        let compiler = if matches!(self.transaction, Transaction::Recording(_)) {
            Some(
                self.session
                    .suspend_transaction()
                    .map_err(|error| jai_vm::Error::EffectRejected(error.to_string()))?,
            )
        } else {
            None
        };
        let transaction = std::mem::replace(&mut self.transaction, Transaction::Idle);
        self.cache.suspended_bytes += bytes;
        self.cache.suspended.insert(
            origin,
            SuspendedRun {
                transaction,
                compiler,
                preview: None,
                bytes,
                pending: self.pending.take(),
                emitted: std::mem::take(&mut self.emitted),
                branch: self.branch.take(),
            },
        );
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        if !matches!(self.transaction, Transaction::Idle) {
            return Err(jai_vm::Error::EffectRejected(
                "another compiler replay transaction is active".into(),
            ));
        }
        let origin = self.origin.as_ref().ok_or_else(|| {
            jai_vm::Error::EffectRejected("compiler resume requires a trusted source origin".into())
        })?;
        let parked = self.cache.suspended.remove(origin).ok_or_else(|| {
            jai_vm::Error::EffectRejected("no suspended compiler job for this source origin".into())
        })?;
        self.cache.suspended_bytes -= parked.bytes;
        if let Some(compiler) = parked.compiler {
            let resumed = if let Some(preview) = parked.preview {
                self.session
                    .resume_transaction_with_preview(compiler, preview)
            } else {
                self.session.resume_transaction(compiler)
            };
            if let Err(error) = resumed {
                self.transaction = Transaction::Rejected(error.to_string());
                return Err(jai_vm::Error::EffectRejected(error.to_string()));
            }
        }
        self.transaction = parked.transaction;
        self.pending = parked.pending;
        self.emitted = parked.emitted;
        self.branch = parked.branch;
        self.active_origin = self.origin.clone();
        Ok(())
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        if self.active_origin != self.origin
            || !self
                .pending
                .as_ref()
                .is_some_and(|(pending, expected)| pending == request && *expected == key)
        {
            return self
                .reject("compiler poll does not match this source job's pending request".into());
        }
        let Transaction::Recording(requests) = &mut self.transaction else {
            return self.reject("compiler poll requires its retained recording transaction".into());
        };
        let outcome = self.session.poll_request(request, key);
        if let EffectOutcome::Ready(response) = &outcome {
            requests.push(RecordedRequest {
                request: request.clone(),
                response: response.clone(),
            });
            self.pending = None;
        }
        outcome
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        if self.active_origin != self.origin {
            return self.reject("compiler source origin changed during an active job".into());
        }
        if self.pending.is_some() {
            return self.reject("compiler request must poll its existing pending slot".into());
        }
        match &mut self.transaction {
            Transaction::Replaying { requests, next } => {
                let Some(recorded) = requests.get(*next) else {
                    return self.reject(
                        "compile-time effect replay added a request after the committed stream"
                            .into(),
                    );
                };
                if recorded.request != request {
                    let reason =
                        format!("compile-time effect replay changed request {}", *next + 1);
                    return self.reject(reason);
                }
                *next += 1;
                EffectOutcome::Ready(recorded.response.clone())
            }
            Transaction::Recording(requests) => {
                if requests.len() >= self.cache.limits.requests_per_run {
                    return self.reject("compile-time effect replay request limit exceeded".into());
                }
                let outcome = self.session.request(request.clone());
                if let EffectOutcome::Ready(response) = &outcome {
                    let changed = match &request {
                        CompilerRequest::AddSource { workspace, .. }
                        | CompilerRequest::AddSourceAt { workspace, .. }
                        | CompilerRequest::AddSourceFile { workspace, .. }
                        | CompilerRequest::AddSourceFileAt { workspace, .. }
                        | CompilerRequest::SetBuildOption { workspace, .. }
                        | CompilerRequest::SetBuildOptionAt { workspace, .. }
                        | CompilerRequest::DestroyWorkspace { workspace } => Some(*workspace),
                        _ => None,
                    };
                    if let Some(workspace) = changed {
                        self.emitted.retain(|event| event.workspace() != workspace);
                    }
                    requests.push(RecordedRequest {
                        request,
                        response: response.clone(),
                    });
                } else if let EffectOutcome::Pending(key) = outcome {
                    self.pending = Some((request, key));
                }
                outcome
            }
            Transaction::Rejected(reason) => EffectOutcome::Rejected(reason.clone()),
            Transaction::Idle => {
                self.reject("compile-time effect request requires an active transaction".into())
            }
        }
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        if commit && self.pending.is_some() {
            self.session.finish(false)?;
            self.transaction = Transaction::Idle;
            self.pending = None;
            return Err(jai_vm::Error::EffectRejected(
                "compiler continuation has an unresolved request".into(),
            ));
        }
        if commit && self.active_origin != self.origin {
            self.session.finish(false)?;
            self.transaction = Transaction::Idle;
            self.pending = None;
            return Err(jai_vm::Error::EffectRejected(
                "compiler source origin changed before commit".into(),
            ));
        }
        if let Some(origin) = &self.origin
            && self.cache.suspended.contains_key(origin)
        {
            if commit {
                return Err(jai_vm::Error::EffectRejected(
                    "compiler continuation must resume before committing".into(),
                ));
            }
            let parked = self.cache.suspended.remove(origin).expect("checked job");
            self.cache.suspended_bytes -= parked.bytes;
        }
        let transaction = std::mem::replace(&mut self.transaction, Transaction::Idle);
        self.pending = None;
        self.emitted.clear();
        self.active_origin = None;
        if !commit {
            self.branch = None;
        }
        match transaction {
            Transaction::Replaying { requests, next } => {
                if commit && next != requests.len() {
                    return Err(jai_vm::Error::EffectRejected(format!(
                        "compile-time effect replay omitted {} committed request(s)",
                        requests.len() - next
                    )));
                }
                Ok(())
            }
            Transaction::Recording(requests) => {
                if commit {
                    self.commit_recording(requests)
                } else {
                    self.session.finish(false)
                }
            }
            Transaction::Rejected(reason) => {
                self.session.finish(false)?;
                if commit {
                    Err(jai_vm::Error::EffectRejected(reason))
                } else {
                    Ok(())
                }
            }
            Transaction::Idle => {
                if commit {
                    Err(jai_vm::Error::EffectRejected(
                        "compile-time effect transaction was not begun".into(),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

fn estimate(origin: &SourceOrigin, requests: &[RecordedRequest]) -> usize {
    let mut bytes = origin
        .path
        .as_os_str()
        .as_encoded_bytes()
        .len()
        .saturating_add(origin.body.len())
        .saturating_add(origin.specialization.len());
    for recorded in requests {
        bytes = bytes.saturating_add(std::mem::size_of::<RecordedRequest>());
        let text = match &recorded.request {
            CompilerRequest::WriteOutput { bytes, .. } => bytes.len(),
            CompilerRequest::AddSource { source, .. }
            | CompilerRequest::AddSourceAt { source, .. } => source.len(),
            CompilerRequest::AddSourceFile { path, .. }
            | CompilerRequest::AddSourceFileAt { path, .. } => {
                path.as_os_str().as_encoded_bytes().len()
            }
            CompilerRequest::CreateWorkspace { name } => name.len(),
            CompilerRequest::Message { text, .. } | CompilerRequest::Report { text, .. } => {
                text.len()
            }
            CompilerRequest::SetBuildOption { option, .. }
            | CompilerRequest::SetBuildOptionAt { option, .. } => match option {
                jai_vm::BuildOption::OutputPath(path) => path.as_os_str().as_encoded_bytes().len(),
                jai_vm::BuildOption::Target(target) => target.as_str().len(),
                _ => 0,
            },
            CompilerRequest::GetBuildOptions { .. }
            | CompilerRequest::GetWorkspaceName { .. }
            | CompilerRequest::SetWorkspaceStatus { .. }
            | CompilerRequest::DestroyWorkspace { .. }
            | CompilerRequest::BeginIntercept { .. }
            | CompilerRequest::EndIntercept { .. }
            | CompilerRequest::WaitForMessage => 0,
        };
        bytes = bytes.saturating_add(text);
        if let CompilerResponse::WorkspaceName(name) = &recorded.response {
            bytes = bytes.saturating_add(name.len());
        }
        if let CompilerRequest::SetBuildOptionAt { location, .. }
        | CompilerRequest::AddSourceAt { location, .. }
        | CompilerRequest::AddSourceFileAt { location, .. }
        | CompilerRequest::Report { location, .. } = &recorded.request
        {
            bytes = bytes.saturating_add(location.path.as_os_str().as_encoded_bytes().len());
        }
        if let CompilerResponse::BuildOptions(snapshot) = &recorded.response {
            bytes = bytes
                .saturating_add(
                    snapshot
                        .output_path
                        .as_ref()
                        .map_or(0, |path| path.as_os_str().as_encoded_bytes().len()),
                )
                .saturating_add(
                    snapshot
                        .target
                        .as_ref()
                        .map_or(0, |target| target.as_str().len()),
                );
        }
    }
    bytes
}

#[cfg(test)]
#[path = "effect_replay/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effect_replay/continuation_tests.rs"]
mod continuation_tests;
