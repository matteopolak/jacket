//! The only replay here is a pure adapter's already-issued request prefix.
//! Source expressions, procedures and effect requests are never replayed.
use super::*;
use crate::host_effects::{FileOpenObservation, HostOutcome, HostRequest, HostResponse};

pub(crate) struct EffectJournal {
    records: Vec<Record>,
    cursor: usize,
    cells: usize,
    limit: usize,
    work: usize,
    remaining_fuel: u64,
    failure: Option<Error>,
}
enum Record {
    CompilerReady(CompilerRequest, CompilerResponse),
    CompilerPending(CompilerRequest, EffectKey),
    HostReady(HostRequest, HostResponse),
    HostPending(HostRequest, crate::host_effects::HostRequestKey),
}
pub(crate) struct JournalEffects<E> {
    inner: E,
    journal: Option<EffectJournal>,
}
impl<E> JournalEffects<E> {
    pub(crate) fn new(inner: E) -> Self {
        Self {
            inner,
            journal: None,
        }
    }
    pub(crate) fn inner(&self) -> &E {
        &self.inner
    }
    pub(crate) fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }
    pub(crate) fn begin_leaf(&mut self, limit: usize, remaining_fuel: u64) -> Result<(), Error> {
        if self.journal.is_some() {
            return Err(Error::InvalidIr("nested resumable effect leaf"));
        }
        self.journal = Some(EffectJournal {
            records: vec![],
            cursor: 0,
            cells: 0,
            limit,
            work: 0,
            remaining_fuel,
            failure: None,
        });
        Ok(())
    }
    pub(crate) fn retry_leaf(&mut self, remaining_fuel: u64) -> Result<(), Error> {
        let journal = self
            .journal
            .as_mut()
            .ok_or(Error::InvalidIr("missing suspended effect leaf"))?;
        journal.cursor = 0;
        journal.remaining_fuel = remaining_fuel;
        Ok(())
    }
    pub(crate) fn end_leaf(&mut self) -> Result<(), Error> {
        let journal = self
            .journal
            .take()
            .ok_or(Error::InvalidIr("missing effect leaf"))?;
        if let Some(error) = journal.failure {
            return Err(error);
        }
        if journal.cursor != journal.records.len()
            || journal.records.iter().any(|record| {
                matches!(
                    record,
                    Record::CompilerPending(..) | Record::HostPending(..)
                )
            })
        {
            return Err(Error::InvalidIr(
                "resumed effect adapter changed its request stream",
            ));
        }
        Ok(())
    }
    pub(crate) fn take_failure(&mut self) -> Option<Error> {
        self.journal
            .as_mut()
            .and_then(|journal| journal.failure.take())
    }
    pub(crate) fn take_work(&mut self) -> usize {
        self.journal
            .as_mut()
            .map_or(0, |journal| std::mem::take(&mut journal.work))
    }
    pub(crate) fn take_journal(&mut self) -> Option<EffectJournal> {
        self.journal.take()
    }
    pub(crate) fn set_journal(&mut self, journal: Option<EffectJournal>) {
        self.journal = journal;
    }
    pub(crate) fn clear_journal(&mut self) {
        self.journal = None;
    }
    pub(crate) fn has_journal(&self) -> bool {
        self.journal.is_some()
    }
}
impl EffectJournal {
    /// Retained storage and a transient copy share the same value-cell limit.
    /// Work is admitted against this attempt's available fuel before any clone.
    fn admit(&mut self, retained: usize, transient: usize, work: usize) -> Result<(), Error> {
        let retained_cells = self
            .cells
            .checked_add(retained)
            .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        retained_cells
            .checked_add(transient)
            .filter(|cells| *cells <= self.limit)
            .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
        let fuel = u64::try_from(work).map_err(|_| Error::Limit(crate::LimitKind::Fuel))?;
        let remaining = self
            .remaining_fuel
            .checked_sub(fuel)
            .ok_or(Error::Limit(crate::LimitKind::Fuel))?;
        let accumulated = self
            .work
            .checked_add(work)
            .ok_or(Error::Limit(crate::LimitKind::Fuel))?;
        self.cells = retained_cells;
        self.remaining_fuel = remaining;
        self.work = accumulated;
        Ok(())
    }
    fn fail(&mut self, error: Error) {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
    }
    fn compiler_failure(&mut self, error: Error) -> EffectOutcome {
        self.fail(error);
        EffectOutcome::Rejected("resumable effect journal rejected the request".into())
    }
    fn host_failure(&mut self, error: Error) -> HostOutcome {
        self.fail(error);
        HostOutcome::Rejected(crate::host_effects::HostError::Denied(
            "resumable effect journal rejected the request",
        ))
    }
}
impl<E: CompilerEffects> CompilerEffects for JournalEffects<E> {
    fn set_source_origin(&mut self, origin: SourceOrigin) {
        self.inner.set_source_origin(origin);
    }
    fn host_file_scope(&self) -> Option<crate::host_effects::FilePathScope> {
        self.inner.host_file_scope()
    }
    fn begin(&mut self) {
        self.inner.begin();
    }
    fn suspend(&mut self) -> Result<(), Error> {
        self.inner.suspend()
    }
    fn resume(&mut self) -> Result<(), Error> {
        self.inner.resume()
    }
    fn service_pending(&mut self, dependencies: &[Dependency]) -> Result<bool, Error> {
        self.inner.service_pending(dependencies)
    }
    fn finish(&mut self, commit: bool) -> Result<(), Error> {
        self.inner.finish(commit)
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        self.inner.poll_request(request, key)
    }
    fn poll_host_request(&mut self, key: crate::host_effects::HostRequestKey) -> HostOutcome {
        self.inner.poll_host_request(key)
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        let Some(journal) = self.journal.as_mut() else {
            return self.inner.request(request);
        };
        if journal.failure.is_some() {
            return EffectOutcome::Rejected("resumable effect journal is failed".into());
        }
        let request_cells = compiler_request_cells(&request);
        if let Some(record) = journal.records.get(journal.cursor) {
            let expected = match record {
                Record::CompilerReady(expected, _) | Record::CompilerPending(expected, _) => {
                    expected
                }
                _ => {
                    return journal.compiler_failure(Error::InvalidIr(
                        "resumed compiler adapter changed its request stream",
                    ));
                }
            };
            let compare_work = compiler_request_cells(expected).saturating_add(request_cells);
            if let Err(error) = journal.admit(0, request_cells, compare_work) {
                return journal.compiler_failure(error);
            }
            let record = &journal.records[journal.cursor];
            let same = match record {
                Record::CompilerReady(expected, _) | Record::CompilerPending(expected, _) => {
                    expected == &request
                }
                _ => false,
            };
            if !same {
                return journal.compiler_failure(Error::InvalidIr(
                    "resumed compiler adapter changed its request stream",
                ));
            }
            match &journal.records[journal.cursor] {
                Record::CompilerReady(_, response) => {
                    let response_cells = compiler_response_cells(response);
                    if let Err(error) = journal.admit(
                        0,
                        request_cells.saturating_add(response_cells),
                        response_cells,
                    ) {
                        return journal.compiler_failure(error);
                    }
                    let Record::CompilerReady(_, response) = &journal.records[journal.cursor]
                    else {
                        unreachable!()
                    };
                    let response = response.clone();
                    journal.cursor += 1;
                    return EffectOutcome::Ready(response);
                }
                Record::CompilerPending(_, key) => {
                    if journal.cursor + 1 != journal.records.len() {
                        return journal.compiler_failure(Error::InvalidIr(
                            "pending compiler request is not the journal leaf",
                        ));
                    }
                    let outcome = self.inner.poll_request(&request, *key);
                    if let EffectOutcome::Ready(response) = outcome {
                        let response_cells = compiler_response_cells(&response);
                        if let Err(error) =
                            journal.admit(response_cells, response_cells, response_cells)
                        {
                            return journal.compiler_failure(error);
                        }
                        journal.records[journal.cursor] =
                            Record::CompilerReady(request, response.clone());
                        journal.cursor += 1;
                        return EffectOutcome::Ready(response);
                    }
                    return outcome;
                }
                _ => unreachable!(),
            }
        }
        if let Err(error) = journal.admit(request_cells, request_cells, request_cells) {
            return journal.compiler_failure(error);
        }
        let outcome = self.inner.request(request.clone());
        match outcome {
            EffectOutcome::Ready(response) => {
                let response_cells = compiler_response_cells(&response);
                if let Err(error) = journal.admit(response_cells, response_cells, response_cells) {
                    return journal.compiler_failure(error);
                }
                journal
                    .records
                    .push(Record::CompilerReady(request, response.clone()));
                journal.cursor += 1;
                EffectOutcome::Ready(response)
            }
            EffectOutcome::Pending(key) => {
                journal.records.push(Record::CompilerPending(request, key));
                EffectOutcome::Pending(key)
            }
            rejected => rejected,
        }
    }
    fn host_request(&mut self, request: HostRequest) -> HostOutcome {
        let Some(journal) = self.journal.as_mut() else {
            return self.inner.host_request(request);
        };
        if journal.failure.is_some() {
            return HostOutcome::Rejected(crate::host_effects::HostError::Denied(
                "resumable effect journal is failed",
            ));
        }
        let request_cells = host_request_cells(&request);
        if let Some(record) = journal.records.get(journal.cursor) {
            let expected = match record {
                Record::HostReady(expected, _) | Record::HostPending(expected, _) => expected,
                _ => {
                    return journal.host_failure(Error::InvalidIr(
                        "resumed host adapter changed its request stream",
                    ));
                }
            };
            let compare_work = host_request_cells(expected).saturating_add(request_cells);
            if let Err(error) = journal.admit(0, request_cells, compare_work) {
                return journal.host_failure(error);
            }
            let record = &journal.records[journal.cursor];
            let same = match record {
                Record::HostReady(expected, _) | Record::HostPending(expected, _) => {
                    expected == &request
                }
                _ => false,
            };
            if !same {
                return journal.host_failure(Error::InvalidIr(
                    "resumed host adapter changed its request stream",
                ));
            }
            match &journal.records[journal.cursor] {
                Record::HostReady(_, response) => {
                    let response_cells = host_response_cells(response);
                    if let Err(error) = journal.admit(
                        0,
                        request_cells.saturating_add(response_cells),
                        response_cells,
                    ) {
                        return journal.host_failure(error);
                    }
                    let Record::HostReady(_, response) = &journal.records[journal.cursor] else {
                        unreachable!()
                    };
                    let response = response.clone();
                    journal.cursor += 1;
                    return HostOutcome::Ready(response);
                }
                Record::HostPending(_, key) => {
                    if journal.cursor + 1 != journal.records.len() {
                        return journal.host_failure(Error::InvalidIr(
                            "pending host request is not the journal leaf",
                        ));
                    }
                    let outcome = self.inner.poll_host_request(*key);
                    if let HostOutcome::Ready(response) = outcome {
                        let response_cells = host_response_cells(&response);
                        if let Err(error) =
                            journal.admit(response_cells, response_cells, response_cells)
                        {
                            return journal.host_failure(error);
                        }
                        journal.records[journal.cursor] =
                            Record::HostReady(request, response.clone());
                        journal.cursor += 1;
                        return HostOutcome::Ready(response);
                    }
                    return outcome;
                }
                _ => unreachable!(),
            }
        }
        if let Err(error) = journal.admit(request_cells, request_cells, request_cells) {
            return journal.host_failure(error);
        }
        let outcome = self.inner.host_request(request.clone());
        match outcome {
            HostOutcome::Ready(response) => {
                let response_cells = host_response_cells(&response);
                if let Err(error) = journal.admit(response_cells, response_cells, response_cells) {
                    return journal.host_failure(error);
                }
                journal
                    .records
                    .push(Record::HostReady(request, response.clone()));
                journal.cursor += 1;
                HostOutcome::Ready(response)
            }
            HostOutcome::Pending(key) => {
                journal.records.push(Record::HostPending(request, key));
                HostOutcome::Pending(key)
            }
            rejected => rejected,
        }
    }
}
// Count owned bytes and the fixed typed fields that their enum variants retain.
fn path_cells(path: &std::path::Path) -> usize {
    path.as_os_str().as_encoded_bytes().len().saturating_add(1)
}
fn option_cells(option: &BuildOption) -> usize {
    match option {
        BuildOption::OutputPath(path) => path_cells(path).saturating_add(1),
        BuildOption::Target(target) => target.as_str().len().saturating_add(2),
        _ => 2,
    }
}
fn location_cells(location: &SourceLocation) -> usize {
    path_cells(&location.path).saturating_add(2)
}
fn compiler_request_cells(request: &CompilerRequest) -> usize {
    let n = match request {
        CompilerRequest::BeginIntercept {
            ..
        }
        | CompilerRequest::SetWorkspaceStatus {
            ..
        } => 2,
        CompilerRequest::EndIntercept {
            ..
        }
        | CompilerRequest::GetWorkspaceName {
            ..
        }
        | CompilerRequest::DestroyWorkspace {
            ..
        }
        | CompilerRequest::GetBuildOptions {
            ..
        } => 1,
        CompilerRequest::WaitForMessage => 0,
        CompilerRequest::SetBuildOptionAt {
            option,
            location,
            ..
        } => option_cells(option)
            .saturating_add(location_cells(location))
            .saturating_add(1),
        CompilerRequest::SetBuildOption {
            option, ..
        } => option_cells(option).saturating_add(1),
        CompilerRequest::WriteOutput {
            bytes, ..
        } => bytes.len().saturating_add(2),
        CompilerRequest::AddSourceAt {
            source,
            location,
            ..
        } => source
            .len()
            .saturating_add(location_cells(location))
            .saturating_add(2),
        CompilerRequest::AddSourceFileAt {
            path,
            location,
            ..
        } => path_cells(path)
            .saturating_add(location_cells(location))
            .saturating_add(1),
        CompilerRequest::Report {
            text,
            location,
            ..
        } => text
            .len()
            .saturating_add(location_cells(location))
            .saturating_add(3),
        CompilerRequest::AddSource {
            source, ..
        } => source.len().saturating_add(2),
        CompilerRequest::AddSourceFile {
            path, ..
        } => path_cells(path).saturating_add(1),
        CompilerRequest::CreateWorkspace {
            name,
        } => name.len().saturating_add(1),
        CompilerRequest::Message {
            text, ..
        } => text.len().saturating_add(2),
    };
    n.saturating_add(1)
}
fn compiler_response_cells(response: &CompilerResponse) -> usize {
    match response {
        CompilerResponse::WorkspaceName(name) => name.len().saturating_add(2),
        CompilerResponse::BuildOptions(options) => 9usize
            .saturating_add(options.output_path.as_deref().map_or(0, path_cells))
            .saturating_add(
                options
                    .target
                    .as_ref()
                    .map_or(0, |target| target.as_str().len().saturating_add(1)),
            ),
        CompilerResponse::Unit => 1,
        CompilerResponse::Workspace(_) => 2,
        CompilerResponse::Message(CompilerEvent::Phase {
            phase, ..
        }) => match phase {
            CompilerPhase::Typechecked {
                ..
            } => 4,
            CompilerPhase::SourceParsed | CompilerPhase::TargetCodeBuilt => 3,
        },
        CompilerResponse::Message(CompilerEvent::Complete {
            ..
        }) => 3,
    }
}
fn host_request_cells(request: &HostRequest) -> usize {
    let path = |path: &crate::host_effects::HostPath| path_cells(path.relative()).saturating_add(1);
    match request {
        HostRequest::ReadEntireFile(value) | HostRequest::ReadFileForOpen(value) => {
            path(value).saturating_add(1)
        }
        HostRequest::WriteEntireFile {
            path: value,
            bytes,
        } => path(value).saturating_add(bytes.len()).saturating_add(2),
        HostRequest::RunProgram(invocation) => {
            invocation
                .arguments
                .as_slice()
                .iter()
                .fold(4usize, |cells, argument| {
                    cells
                        .saturating_add(argument.as_encoded_bytes().len())
                        .saturating_add(1)
                })
        }
    }
}
fn host_response_cells(response: &HostResponse) -> usize {
    match response {
        HostResponse::FileBytes(bytes) => bytes.len().saturating_add(2),
        HostResponse::FileOpen(FileOpenObservation::Bytes(bytes)) => bytes.len().saturating_add(3),
        HostResponse::Process(output) => output
            .stdout
            .len()
            .saturating_add(output.stderr.len())
            .saturating_add(match output.termination {
                crate::host_effects::ProcessTermination::Exited(_) => 5,
                crate::host_effects::ProcessTermination::Signaled
                | crate::host_effects::ProcessTermination::TimedOut => 4,
            }),
        HostResponse::WriteStaged => 1,
        HostResponse::FileOpen(FileOpenObservation::Failed(_)) => 3,
        HostResponse::ProcessLaunchFailed(_) => 3,
    }
}

#[cfg(test)]
mod tests;
