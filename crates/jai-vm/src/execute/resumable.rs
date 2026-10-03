//! An owned evaluation stack; suspension retains the exact next operation.
mod apply;
mod branches;
mod checkpoint;
mod compiler;
pub use compiler::{CompilerCodeSelection, CompilerFrameId};
#[cfg(test)]
mod control_tests;
mod machine;
mod pack;
mod plan;
mod process_scheduler;
#[cfg(test)]
mod tests;

use super::*;
use crate::effects::EffectJournal;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub(super) enum Operand {
    Value(Value),
    Place(Pointer),
    Results(Vec<Value>),
}
impl Operand {
    pub(super) fn into_value(self) -> Result<Value> {
        match self {
            Self::Value(value) => Ok(value),
            _ => Err(Error::InvalidIr("continuation operand is not a value").into()),
        }
    }
    pub(super) fn into_place(self) -> Result<Pointer> {
        match self {
            Self::Place(pointer) => Ok(pointer),
            _ => Err(Error::InvalidIr("continuation operand is not a place").into()),
        }
    }
    pub(super) fn into_results(self) -> Result<Vec<Value>> {
        match self {
            Self::Results(values) => Ok(values),
            Self::Value(value) => Ok(vec![value]),
            _ => Err(Error::InvalidIr("continuation operand is not a result list").into()),
        }
    }
}

/// Values remain private until the caller validates their publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResumableOutcome {
    Suspended(Vec<Dependency>),
    AwaitingPublication,
    Failed(Error),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumableExecution {
    pub outcome: ResumableOutcome,
    pub statistics: Statistics,
}

enum Phase {
    Running,
    Suspended(Dependency),
    AwaitingPublication,
}
pub(super) struct Session {
    root: SessionRoot,
    process_scheduler: process_scheduler::ProcessScheduler,
    checkpoint: Checkpoint,
    compiler_reserved_cells: usize,
    phase: Phase,
    origin: Option<crate::SourceOrigin>,
    signatures: HashMap<ProcedureId, TypeId>,
}
enum SessionRoot {
    Native(machine::Machine),
    Compiler(Box<compiler::CompilerController>),
}
impl SessionRoot {
    fn retain_metadata(&mut self, cells: usize, limit: usize) -> std::result::Result<(), Error> {
        match self {
            Self::Native(machine) => machine.retain_metadata(cells, limit),
            Self::Compiler(controller) => controller.retain_metadata(cells, limit),
        }
    }
    fn retained_cells(&self) -> std::result::Result<usize, Error> {
        match self {
            Self::Native(machine) => Ok(machine.retained_cells()),
            Self::Compiler(controller) => controller.retained_cells(),
        }
    }
    fn values(&self) -> Option<&[Value]> {
        match self {
            Self::Native(machine) => machine.values(),
            Self::Compiler(_) => None,
        }
    }
    fn take_values(&mut self) -> Option<Vec<Value>> {
        match self {
            Self::Native(machine) => machine.take_values(),
            Self::Compiler(_) => None,
        }
    }
}
struct Checkpoint {
    memory: crate::memory::MemorySnapshot,
    host_files: crate::host_files::HostFileMachine,
    host_heap: crate::virtual_heap::VirtualHeap,
    processes: Option<process::ProcessState>,
    globals: Vec<Option<Pointer>>,
    static_objects: HashMap<StaticObjectId, Pointer>,
    static_publications: HashMap<u64, usize>,
    static_data: HashMap<u64, Arc<StaticData>>,
    literal_backing: HashMap<(TypeId, Value), Pointer>,
    default_context: Option<Pointer>,
}
struct OwnedFrame {
    procedure: Arc<Procedure>,
    slots: Vec<Pointer>,
    loops: Vec<LoopId>,
    temporaries: Vec<Pointer>,
    sequence_temp_bytes: u64,
    sequence_temp_roots: Vec<Pointer>,
    procedure_context: Option<Pointer>,
    push_contexts: HashMap<PushContextId, Pointer>,
}

/// A non-cloneable suspended execution, including its rollback checkpoint.
/// Restore its trusted source origin on the effect handler before resuming it.
#[must_use = "restore or cancel the suspended execution to retire its effect transaction"]
pub struct ContinuationState {
    state: VmState,
    session: Session,
    frames: Vec<OwnedFrame>,
    current_context: Option<Pointer>,
    root_temporaries: Vec<Pointer>,
    root_sequence_temp_bytes: u64,
    statistics: Statistics,
    journal: Option<EffectJournal>,
    expression_bindings: bindings::BindingEnvironment,
}

impl<'a, P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'a, P, E> {
    /// The pinned origin remains available during validated result publication.
    pub fn publication_source_origin(&self) -> Option<&crate::SourceOrigin> {
        self.continuation
            .as_ref()
            .and_then(|session| session.origin.as_ref())
            .or(self.publication_origin.as_ref())
    }
    pub fn start_resumable_expression(&mut self, expression: &ValueExpr) -> ResumableExecution {
        self.start_resumable_expression_with_origin(None, expression, None)
    }
    pub fn start_resumable_expression_at(
        &mut self,
        expression: &ValueExpr,
        origin: crate::SourceOrigin,
    ) -> ResumableExecution {
        self.start_resumable_expression_with_origin(None, expression, Some(origin))
    }
    pub fn start_resumable_expression_owned(
        &mut self,
        owner: ProcedureId,
        expression: &ValueExpr,
    ) -> ResumableExecution {
        self.start_resumable_expression_with_origin(Some(owner), expression, None)
    }
    pub fn start_resumable_expression_owned_at(
        &mut self,
        owner: ProcedureId,
        expression: &ValueExpr,
        origin: crate::SourceOrigin,
    ) -> ResumableExecution {
        self.start_resumable_expression_with_origin(Some(owner), expression, Some(origin))
    }
    fn start_resumable_expression_with_origin(
        &mut self,
        owner: Option<ProcedureId>,
        expression: &ValueExpr,
        origin: Option<crate::SourceOrigin>,
    ) -> ResumableExecution {
        if self.continuation.is_some() {
            return self.progress_failed(Error::InvalidIr("a continuation is already active"));
        }
        let empty = Places::default();
        let proof = if let Some(owner) = owner {
            verify_owned_expression_with_context(
                self.provider.types(),
                expression,
                self.provider.signatures(),
                self.provider.globals(),
                self.provider.places().unwrap_or(&empty),
                owner,
                self.provider.context(),
            )
        } else {
            verify_expression_with_context(
                self.provider.types(),
                expression,
                self.provider.signatures(),
                self.provider.globals(),
                self.provider.places().unwrap_or(&empty),
                self.provider.context(),
            )
        };
        let code = proof
            .map_err(|error| Error::IrValidation(error.to_string()))
            .and_then(|checked| plan::compile_checked_expression(checked, self.limits));
        match code {
            Ok(code) => self.begin_resumable(machine::Machine::expression(code), origin),
            Err(error) => self.progress_failed(error),
        }
    }
    pub fn start_resumable_call(&mut self, call: &Call) -> ResumableExecution {
        self.start_resumable_call_with_origin(None, call, None)
    }
    pub fn start_resumable_call_at(
        &mut self,
        call: &Call,
        origin: crate::SourceOrigin,
    ) -> ResumableExecution {
        self.start_resumable_call_with_origin(None, call, Some(origin))
    }
    pub fn start_resumable_call_owned(
        &mut self,
        owner: ProcedureId,
        call: &Call,
    ) -> ResumableExecution {
        self.start_resumable_call_with_origin(Some(owner), call, None)
    }
    pub fn start_resumable_call_owned_at(
        &mut self,
        owner: ProcedureId,
        call: &Call,
        origin: crate::SourceOrigin,
    ) -> ResumableExecution {
        self.start_resumable_call_with_origin(Some(owner), call, Some(origin))
    }
    fn start_resumable_call_with_origin(
        &mut self,
        owner: Option<ProcedureId>,
        call: &Call,
        origin: Option<crate::SourceOrigin>,
    ) -> ResumableExecution {
        if self.continuation.is_some() {
            return self.progress_failed(Error::InvalidIr("a continuation is already active"));
        }
        let empty = Places::default();
        let proof = if let Some(owner) = owner {
            verify_owned_call_with_context(
                self.provider.types(),
                call,
                self.provider.signatures(),
                self.provider.globals(),
                self.provider.places().unwrap_or(&empty),
                owner,
                self.provider.context(),
            )
        } else {
            verify_call_with_context(
                self.provider.types(),
                call,
                self.provider.signatures(),
                self.provider.globals(),
                self.provider.places().unwrap_or(&empty),
                self.provider.context(),
            )
        };
        let code = proof
            .map_err(|error| Error::IrValidation(error.to_string()))
            .and_then(|checked| plan::compile_checked_call(checked, self.limits));
        match code {
            Ok(code) => self.begin_resumable(machine::Machine::expression(code), origin),
            Err(error) => self.progress_failed(error),
        }
    }
    pub fn start_resumable_procedure(
        &mut self,
        id: ProcedureId,
        arguments: Vec<Value>,
    ) -> ResumableExecution {
        if self.continuation.is_some() {
            return self.progress_failed(Error::InvalidIr("a continuation is already active"));
        }
        self.begin_resumable(machine::Machine::procedure(id, arguments), None)
    }
    pub fn start_resumable_procedure_at(
        &mut self,
        id: ProcedureId,
        arguments: Vec<Value>,
        origin: crate::SourceOrigin,
    ) -> ResumableExecution {
        if self.continuation.is_some() {
            return self.progress_failed(Error::InvalidIr("a continuation is already active"));
        }
        self.begin_resumable(machine::Machine::procedure(id, arguments), Some(origin))
    }
    fn begin_resumable(
        &mut self,
        machine: machine::Machine,
        origin: Option<crate::SourceOrigin>,
    ) -> ResumableExecution {
        self.begin_resumable_root(SessionRoot::Native(machine), origin)
    }
    fn begin_resumable_root(
        &mut self,
        mut root: SessionRoot,
        origin: Option<crate::SourceOrigin>,
    ) -> ResumableExecution {
        self.statistics = Statistics::default();
        debug_assert_eq!(self.expression_bindings.depth(), 0);
        if let Some(processes) = &self.processes {
            let work = usize::try_from(processes.work_cost()).unwrap_or(usize::MAX);
            if let Err(halt) = self.charge_work(work) {
                return self.fail_resumable(halt);
            }
        }
        self.root_sequence_temp_bytes = 0;
        let origin_cells = origin.as_ref().map_or(0, source_origin_cells);
        let signatures_work = self.provider.signatures().capacity();
        let metadata = signatures_work.saturating_mul(2);
        if metadata > self.limits.value_cells {
            return self.progress_failed(Error::Limit(LimitKind::ValueCells));
        }
        if let Err(halt) = self.charge_work(signatures_work.saturating_add(origin_cells)) {
            return self.fail_resumable(halt);
        }
        if let Err(error) = root.retain_metadata(
            metadata,
            self.limits
                .value_cells
                .saturating_sub(self.memory.value_cells()),
        ) {
            return self.progress_failed(error);
        }
        let root_cells = match root.retained_cells() {
            Ok(cells) => cells,
            Err(error) => return self.progress_failed(error),
        };
        // Move the pinned origin while the checkpoint gate measures common owners.
        let previous_origin = self.publication_origin.take();
        self.publication_origin = origin;
        let admission = match checkpoint::prepare(self, root_cells) {
            Ok(admission) => admission,
            Err(halt) => {
                self.publication_origin = previous_origin;
                return self.fail_resumable(halt);
            }
        };
        let origin = self.publication_origin.take();
        self.publication_origin = previous_origin;
        let signatures = self.provider.signatures().clone();
        let checkpoint = Checkpoint {
            memory: self.memory.snapshot(),
            host_files: self.host_files.clone(),
            host_heap: self.host_heap.clone(),
            processes: self.processes.clone(),
            globals: self.globals.clone(),
            static_objects: self.static_objects.clone(),
            static_publications: self.static_publications.clone(),
            static_data: self.static_data.clone(),
            literal_backing: self.literal_backing.clone(),
            default_context: self.default_context.clone(),
        };
        if let Some(origin) = &origin {
            self.effects.set_source_origin(origin.clone());
        }
        self.effects.begin();
        self.continuation = Some(Session {
            root,
            process_scheduler: process_scheduler::ProcessScheduler::new(
                admission.cells,
                admission.resident_ancillary_cells,
            ),
            checkpoint,
            compiler_reserved_cells: admission.cells,
            phase: Phase::Running,
            origin,
            signatures,
        });
        self.drive_resumable()
    }
    pub fn resume_resumable(&mut self) -> ResumableExecution {
        let Some(session) = self.continuation.as_ref() else {
            return self.progress_failed(Error::InvalidIr("no continuation is active"));
        };
        if !matches!(session.phase, Phase::Suspended(_)) {
            return self.progress_failed(Error::InvalidIr("continuation is not suspended"));
        }
        let work = session
            .signatures
            .capacity()
            .saturating_add(session.origin.as_ref().map_or(0, source_origin_cells));
        if let Err(halt) = self.charge_work(work) {
            return self.fail_resumable(halt);
        }
        let session = self.continuation.as_ref().unwrap();
        if let Err(error) = validate_signature_prefix(self.provider, &session.signatures) {
            return self.fail_resumable(error.into());
        }
        if let Some(origin) = &session.origin {
            self.effects.set_source_origin(origin.clone());
        }
        if let Err(error) = self.effects.resume() {
            return self.fail_resumable(error.into());
        }
        self.continuation.as_mut().unwrap().phase = Phase::Running;
        self.drive_resumable()
    }
    fn drive_resumable(&mut self) -> ResumableExecution {
        let mut session = self.continuation.take().expect("active continuation");
        let previous_origin = self.publication_origin.take();
        self.publication_origin = session.origin.take();
        let previous_retention = (
            self.transaction_retained_cells,
            self.transaction_ancillary_cells,
            self.transaction_value_cell_limit,
        );
        let result = match &mut session.root {
            SessionRoot::Native(machine) => session.process_scheduler.drive(self, machine),
            SessionRoot::Compiler(controller) => controller
                .drive_with_checkpoint(self, session.compiler_reserved_cells)
                .map(|()| machine::DriveStatus::Complete),
        };
        session
            .process_scheduler
            .refresh_publication_ancillary(self.transaction_ancillary_cells);
        session.origin = self.publication_origin.take();
        self.publication_origin = previous_origin;
        (
            self.transaction_retained_cells,
            self.transaction_ancillary_cells,
            self.transaction_value_cell_limit,
        ) = previous_retention;
        self.continuation = Some(session);
        if let Err(halt) = self.flush_publication_work() {
            return self.fail_resumable(halt);
        }
        match result {
            Ok(machine::DriveStatus::Complete) => {
                self.continuation.as_mut().unwrap().phase = Phase::AwaitingPublication;
                self.progress(ResumableOutcome::AwaitingPublication)
            }
            Ok(machine::DriveStatus::ProcessControl | machine::DriveStatus::Retired) => self
                .fail_resumable(
                    Error::UnsupportedPointerOperation(
                        "process control transfer requires a branch continuation scheduler",
                    )
                    .into(),
                ),
            Err(Halt::Pending(dependency)) => {
                if let Err(error) = self.effects.suspend() {
                    return self.fail_resumable(error.into());
                }
                self.continuation.as_mut().unwrap().phase = Phase::Suspended(dependency.clone());
                self.progress(ResumableOutcome::Suspended(vec![dependency]))
            }
            Err(halt) => self.fail_resumable(halt),
        }
    }
    pub fn resumable_values(&self) -> Option<&[Value]> {
        let session = self.continuation.as_ref()?;
        matches!(session.phase, Phase::AwaitingPublication)
            .then(|| session.root.values())
            .flatten()
    }
    pub fn resumable_dependency(&self) -> Option<&Dependency> {
        match &self.continuation.as_ref()?.phase {
            Phase::Suspended(dependency) => Some(dependency),
            _ => None,
        }
    }
    pub fn finish_resumable_validated(
        &mut self,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
        let Some(values) = self.resumable_values() else {
            return Execution {
                outcome: Outcome::Failed(Error::InvalidIr(
                    "continuation has no publishable result",
                )),
                statistics: self.publication_statistics(),
            };
        };
        let validation = validator(self, values)
            .and_then(|()| self.validate_sequence_escape(values, &self.root_temporaries))
            .and_then(|()| self.host_files.require_closed());
        let validation = validation.and_then(|()| {
            if let Some(processes) = &self.processes {
                processes
                    .world
                    .require_quiescent(processes.branch.current())
                    .map_err(|error| Error::EffectRejected(error.to_string()))?;
            }
            Ok(())
        });
        if let Err(error) = validation {
            let progress = self.fail_resumable(error.into());
            return Execution {
                outcome: Outcome::Failed(match progress.outcome {
                    ResumableOutcome::Failed(error) => error,
                    _ => unreachable!(),
                }),
                statistics: progress.statistics,
            };
        }
        if let Err(halt) = self.flush_publication_work() {
            let failure = self.fail_resumable(halt);
            return Execution {
                outcome: Outcome::Failed(match failure.outcome {
                    ResumableOutcome::Failed(error) => error,
                    _ => unreachable!(),
                }),
                statistics: failure.statistics,
            };
        }
        let mut session = self.continuation.take().unwrap();
        if let Some(origin) = &session.origin {
            self.effects.set_source_origin(origin.clone());
        }
        let values = session.root.take_values().unwrap();
        let mut result = Ok(());
        for pointer in self.root_temporaries.drain(..) {
            if let Err(error) = self.memory.release(&pointer) {
                result = Err(error);
                break;
            }
        }
        if result.is_ok() {
            result = self.effects.finish(true);
        } else {
            let _ = self.effects.finish(false);
        }
        self.current_context = None;
        self.process_branch_execution = false;
        self.expression_bindings.clear();
        self.effects.clear_journal();
        match result {
            Ok(()) => Execution {
                outcome: Outcome::Complete(values),
                statistics: self.publication_statistics(),
            },
            Err(error) => {
                self.restore_checkpoint(session.checkpoint);
                Execution {
                    outcome: Outcome::Failed(error),
                    statistics: self.publication_statistics(),
                }
            }
        }
    }
    pub fn cancel_resumable(&mut self) -> std::result::Result<(), Error> {
        self.flush_publication_work()
            .map_err(publication_admission_error)?;
        let Some(session) = self.continuation.take() else {
            return Err(Error::InvalidIr("no continuation is active"));
        };
        if let Some(origin) = &session.origin {
            self.effects.set_source_origin(origin.clone());
        }
        let result = self.effects.finish(false);
        self.restore_checkpoint(session.checkpoint);
        result
    }
    fn fail_resumable(&mut self, halt: Halt) -> ResumableExecution {
        let _ = self.flush_publication_work();
        let error = match halt {
            Halt::Failed(error) => error,
            Halt::Pending(_) => Error::InvalidIr("continuation initialization is pending"),
        };
        if let Some(session) = self.continuation.take() {
            if let Some(origin) = &session.origin {
                self.effects.set_source_origin(origin.clone());
            }
            let _ = self.effects.finish(false);
            self.restore_checkpoint(session.checkpoint);
        }
        self.progress_failed(error)
    }
    fn restore_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.memory.restore(checkpoint.memory);
        self.host_files = checkpoint.host_files;
        self.host_heap = checkpoint.host_heap;
        self.processes = checkpoint.processes;
        self.globals = checkpoint.globals;
        self.static_objects = checkpoint.static_objects;
        self.static_publications = checkpoint.static_publications;
        self.static_data = checkpoint.static_data;
        self.literal_backing = checkpoint.literal_backing;
        self.default_context = checkpoint.default_context;
        self.frames.clear();
        self.expression_bindings.clear();
        self.root_temporaries.clear();
        self.root_sequence_temp_bytes = 0;
        self.current_context = None;
        self.process_branch_execution = false;
        self.effects.clear_journal();
    }
    fn progress(&self, outcome: ResumableOutcome) -> ResumableExecution {
        ResumableExecution {
            outcome,
            statistics: self.publication_statistics(),
        }
    }
    fn progress_failed(&self, error: Error) -> ResumableExecution {
        self.progress(ResumableOutcome::Failed(error))
    }
    pub fn into_continuation(mut self) -> std::result::Result<ContinuationState, Error> {
        self.flush_publication_work()
            .map_err(publication_admission_error)?;
        let session = self
            .continuation
            .take()
            .ok_or(Error::InvalidIr("no continuation is active"))?;
        if !matches!(session.phase, Phase::Suspended(_)) {
            self.continuation = Some(session);
            return Err(Error::InvalidIr(
                "only a suspended continuation can be transferred",
            ));
        }
        let statistics = self.publication_statistics();
        let mut frames = Vec::with_capacity(self.frames.len());
        for frame in self.frames {
            let FrameCode::Owned(procedure) = frame.procedure else {
                return Err(Error::InvalidIr(
                    "borrowed legacy frame cannot be transferred",
                ));
            };
            frames.push(OwnedFrame {
                procedure,
                slots: frame.slots,
                loops: frame.loops,
                temporaries: frame.temporaries,
                sequence_temp_bytes: frame.sequence_temp_bytes,
                sequence_temp_roots: frame.sequence_temp_roots,
                procedure_context: frame.procedure_context,
                push_contexts: frame.push_contexts,
            });
        }
        let state = VmState {
            memory: self.memory,
            execution_phase: self.execution_phase,
            host_files: self.host_files,
            host_heap: self.host_heap,
            processes: self.processes,
            globals: self.globals,
            definitions: self.provider.globals().to_vec(),
            static_objects: self.static_objects,
            static_publications: self.static_publications,
            static_data: self.static_data,
            literal_backing: self.literal_backing,
            registry: self.provider.types().scalar(jai_types::ScalarType::Bool),
            limits: self.limits,
            default_context: self.default_context,
            context_definition: self.provider.context().cloned(),
        };
        Ok(ContinuationState {
            state,
            session,
            frames,
            current_context: self.current_context,
            root_temporaries: self.root_temporaries,
            root_sequence_temp_bytes: self.root_sequence_temp_bytes,
            statistics,
            journal: self.effects.take_journal(),
            expression_bindings: self.expression_bindings,
        })
    }
    pub fn with_continuation(
        provider: &'a P,
        mut effects: E,
        state: ContinuationState,
    ) -> std::result::Result<Self, Error> {
        let limits = state.state.limits;
        if let Some(origin) = &state.session.origin {
            effects.set_source_origin(origin.clone());
        }
        if let Err(error) = validate_signature_prefix(provider, &state.session.signatures)
            .and_then(|()| Self::validate_state(provider, limits, &state.state, true))
        {
            let _ = effects.finish(false);
            return Err(error);
        }
        let mut vm = Self::from_validated_state(provider, effects, limits, state.state);
        vm.frames = state
            .frames
            .into_iter()
            .map(|frame| Frame {
                procedure: FrameCode::Owned(frame.procedure),
                slots: frame.slots,
                loops: frame.loops,
                temporaries: frame.temporaries,
                sequence_temp_bytes: frame.sequence_temp_bytes,
                sequence_temp_roots: frame.sequence_temp_roots,
                procedure_context: frame.procedure_context,
                push_contexts: frame.push_contexts,
            })
            .collect();
        vm.current_context = state.current_context;
        vm.root_temporaries = state.root_temporaries;
        vm.root_sequence_temp_bytes = state.root_sequence_temp_bytes;
        vm.statistics = state.statistics;
        vm.expression_bindings = state.expression_bindings;
        vm.effects.set_journal(state.journal);
        vm.continuation = Some(state.session);
        Ok(vm)
    }
}

impl ContinuationState {
    /// Cancels a detached job without reconstructing a VM or replaying source.
    pub fn cancel(self, effects: &mut impl CompilerEffects) -> std::result::Result<(), Error> {
        if let Some(origin) = self.session.origin {
            effects.set_source_origin(origin);
        }
        effects.finish(false)
    }
    pub fn source_origin(&self) -> Option<&crate::SourceOrigin> {
        self.session.origin.as_ref()
    }
}

fn validate_signature_prefix(
    provider: &(impl ProcedureProvider + ?Sized),
    signatures: &HashMap<ProcedureId, TypeId>,
) -> std::result::Result<(), Error> {
    if signatures
        .iter()
        .any(|(id, signature)| provider.signatures().get(id) != Some(signature))
    {
        return Err(Error::InvalidIr(
            "continuation procedure signature ledger changed",
        ));
    }
    Ok(())
}

fn source_origin_cells(origin: &crate::SourceOrigin) -> usize {
    origin
        .path
        .capacity()
        .saturating_add(origin.body.capacity())
        .saturating_add(origin.specialization.capacity())
        .saturating_add(8)
}

// Paired hooks in execute/resumable.rs; fields and ordinary transaction hooks
// are supplied by constant_slices. SessionRoot conversion is supplied by compiler.
pub(super) fn publication_transaction_admission<
    P: ProcedureProvider + ?Sized,
    E: CompilerEffects,
>(
    vm: &mut Vm<'_, P, E>,
) -> std::result::Result<(usize, usize), Error> {
    checkpoint::prepare(vm, 0)
        .map(|admission| (admission.cells, admission.resident_ancillary_cells))
        .map_err(publication_admission_error)
}
pub(super) fn publication_transaction_residual<
    P: ProcedureProvider + ?Sized,
    E: CompilerEffects,
>(
    vm: &mut Vm<'_, P, E>,
) -> std::result::Result<usize, Error> {
    let storage = branches::measured(vm, false).map_err(publication_admission_error)?;
    checkpoint::residual(vm, storage.cells).map_err(publication_admission_error)
}
fn publication_admission_error(halt: Halt) -> Error {
    match halt {
        Halt::Failed(error) => error,
        Halt::Pending(_) => Error::InvalidIr("publication snapshot admission is pending"),
    }
}
pub(super) fn publication_origin_cells<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
) -> usize {
    vm.publication_origin_cells()
}
pub(super) fn cache_drive_publication_retention<
    P: ProcedureProvider + ?Sized,
    E: CompilerEffects,
>(
    vm: &mut Vm<'_, P, E>,
    original_limit: usize,
    retained_roots: usize,
    ancillary_without_pool: usize,
) -> Result<()> {
    vm.transaction_retained_cells = retained_roots;
    vm.transaction_ancillary_cells = ancillary_without_pool;
    vm.transaction_value_cell_limit = Some(original_limit);
    Ok(())
}
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub fn publication_value_cell_limit(&self) -> usize {
        self.transaction_value_cell_limit
            .unwrap_or(self.limits.value_cells)
    }
    pub fn publication_retained_cells(&self) -> std::result::Result<usize, Error> {
        let (reserved, root) = if let Some(session) = &self.continuation {
            (
                session.process_scheduler.publication_cells()?,
                session.root.retained_cells()?,
            )
        } else if self.transaction_value_cell_limit.is_some() {
            (
                self.transaction_retained_cells
                    .checked_add(self.transaction_ancillary_cells)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?,
                0,
            )
        } else {
            return Err(Error::InvalidIr(
                "publication requires an admitted transaction",
            ));
        };
        self.memory
            .value_cells()
            .checked_add(self.expression_bindings.cells())
            .and_then(|cells| {
                cells.checked_add(
                    self.processes
                        .as_ref()
                        .map_or(0, process::ProcessState::cells),
                )
            })
            .and_then(|cells| cells.checked_add(reserved))
            .and_then(|cells| cells.checked_add(root))
            .and_then(|cells| cells.checked_add(publication_origin_cells(self)))
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
    pub fn publication_remaining_value_cells(&self) -> std::result::Result<usize, Error> {
        self.publication_value_cell_limit()
            .checked_sub(self.publication_retained_cells()?)
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
}
