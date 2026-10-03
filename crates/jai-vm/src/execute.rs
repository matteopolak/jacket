mod any_values;
mod bindings;
mod budgets;
mod compiler_messages;
mod compiler_output;
mod compiler_runtime_info;
mod compiler_version;
mod constant_values;
mod context;
mod execution_phase;
mod floating;
pub use execution_phase::ExecutionPhase;
mod host_arguments;
mod host_files;
mod host_heap;
mod numbers;
mod ordered_records;
mod process;
#[cfg(test)]
mod publication_transactions_tests;
mod resumable;
pub use resumable::{
    CompilerCodeSelection, CompilerFrameId, ContinuationState, ResumableExecution, ResumableOutcome,
};
mod runtime_types;
mod sequence_concat;
mod sequences;
mod simd;
mod static_data;
mod storage_bitcasts;
mod string_comparison;
mod value_bindings;
mod values;
use crate::{
    CompilerEffects, CompilerProcedure, Dependency, EffectError, Error, Execution, LimitKind,
    Limits, Memory, Number, Outcome, Pointer, Statistics, Value, scalar,
};
use jai_ir::*;
use jai_types::Equality;
use jai_types::{Integer, ProcedureType, TypeError, TypeId, TypeKind, TypeView};

/// A body can be executed while unrelated declarations/types are still unresolved.
#[allow(clippy::large_enum_variant)]
pub enum ProcedureAvailability<'a> {
    Ready(CheckedProcedure<'a>),
    Failed(Error),
    Compiler(CompilerProcedure),
    Runtime(crate::RuntimeProcedure),
    FileAbi(crate::file_abi::FileAbiProcedure),
    HeapAbi(crate::heap_abi::HeapAbiProcedure),
    ProcessAbi(crate::process_abi::ProcessAbiProcedure),
    Pending(Dependency),
    Missing,
    Foreign,
}
pub enum RuntimeInfoAvailability<'a> {
    Ready {
        workspace: crate::WorkspaceId,
        snapshot: &'a RuntimeInfoSnapshot,
    },
    Pending(Dependency),
    Missing,
}
pub trait ProcedureProvider {
    fn types(&self) -> &dyn TypeView;
    fn procedure_execution(&self, _id: ProcedureId) -> jai_types::ProcedureExecution {
        jai_types::ProcedureExecution::RuntimeAndCompileTime
    }
    fn global_alignment_pending(&self, _id: GlobalId) -> bool {
        false
    }
    fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
        static EMPTY: std::sync::OnceLock<std::collections::HashMap<ProcedureId, TypeId>> =
            std::sync::OnceLock::new();
        EMPTY.get_or_init(std::collections::HashMap::new)
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_>;
    fn globals(&self) -> &[Global] {
        &[]
    }
    fn places(&self) -> Option<&Places> {
        None
    }
    fn context(&self) -> Option<&ContextDefinition> {
        None
    }
    fn storage_alignments(&self) -> Option<&StorageAlignments> {
        None
    }
    fn runtime_info(&self) -> RuntimeInfoAvailability<'_> {
        RuntimeInfoAvailability::Missing
    }
}
impl ProcedureProvider for Program {
    fn types(&self) -> &dyn TypeView {
        self.types()
    }
    fn procedure_execution(&self, id: ProcedureId) -> jai_types::ProcedureExecution {
        self.library().procedure_phase(id)
    }
    fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
        self.signatures()
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if let Some(checked) = self.checked_procedure(id) {
            ProcedureAvailability::Ready(checked)
        } else if let Some(prototype) = self
            .library()
            .prototypes()
            .iter()
            .find(|prototype| prototype.id == id)
        {
            runtime_prototype(prototype)
        } else {
            ProcedureAvailability::Missing
        }
    }
    fn globals(&self) -> &[Global] {
        self.globals()
    }
    fn storage_alignments(&self) -> Option<&StorageAlignments> {
        Some(self.storage_alignments())
    }
    fn places(&self) -> Option<&Places> {
        Some(self.places())
    }
    fn context(&self) -> Option<&ContextDefinition> {
        self.context()
    }
}
impl ProcedureProvider for Library {
    fn types(&self) -> &dyn TypeView {
        self.types()
    }
    fn procedure_execution(&self, id: ProcedureId) -> jai_types::ProcedureExecution {
        self.procedure_phase(id)
    }
    fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
        self.signatures()
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if let Some(checked) = self.checked_procedure(id) {
            ProcedureAvailability::Ready(checked)
        } else if let Some(prototype) = self
            .prototypes()
            .iter()
            .find(|prototype| prototype.id == id)
        {
            runtime_prototype(prototype)
        } else {
            ProcedureAvailability::Missing
        }
    }
    fn globals(&self) -> &[Global] {
        self.globals()
    }
    fn storage_alignments(&self) -> Option<&StorageAlignments> {
        Some(self.storage_alignments())
    }
    fn places(&self) -> Option<&Places> {
        Some(self.places())
    }
    fn context(&self) -> Option<&ContextDefinition> {
        self.context()
    }
}
fn runtime_prototype(prototype: &ProcedurePrototype) -> ProcedureAvailability<'_> {
    match prototype.origin {
        PrototypeOrigin::Intrinsic(intrinsic) => {
            ProcedureAvailability::Runtime(crate::RuntimeProcedure {
                signature: prototype.signature,
                intrinsic,
            })
        }
        PrototypeOrigin::Foreign {
            ..
        }
        | PrototypeOrigin::Compiler => ProcedureAvailability::Foreign,
        PrototypeOrigin::SourceContract {
            ..
        } => ProcedureAvailability::Failed(Error::InvalidIr(
            "source contract has no compile-time implementation provider",
        )),
    }
}
#[derive(Debug)]
enum Halt {
    Pending(Dependency),
    Failed(Error),
}
impl From<Error> for Halt {
    fn from(error: Error) -> Self {
        match error {
            Error::Type(TypeError::Incomplete(ty)) => Self::Pending(Dependency::Type(ty)),
            error => Self::Failed(error),
        }
    }
}
impl From<jai_types::LayoutError> for Halt {
    fn from(error: jai_types::LayoutError) -> Self {
        Error::from(error).into()
    }
}
impl From<TypeError> for Halt {
    fn from(error: TypeError) -> Self {
        Error::Type(error).into()
    }
}
impl From<EffectError> for Halt {
    fn from(error: EffectError) -> Self {
        match error {
            EffectError::Pending(dependency) => Self::Pending(dependency),
            EffectError::Failed(error) => Self::Failed(error),
        }
    }
}
type Result<T> = std::result::Result<T, Halt>;
#[derive(Clone)]
enum FrameCode<'a> {
    Borrowed(&'a Procedure),
    Owned(std::sync::Arc<Procedure>),
}
impl std::ops::Deref for FrameCode<'_> {
    type Target = Procedure;
    fn deref(&self) -> &Procedure {
        match self {
            Self::Borrowed(procedure) => procedure,
            Self::Owned(procedure) => procedure,
        }
    }
}
struct Frame<'a> {
    procedure: FrameCode<'a>,
    slots: Vec<Pointer>,
    loops: Vec<LoopId>,
    temporaries: Vec<Pointer>,
    sequence_temp_bytes: u64,
    sequence_temp_roots: Vec<Pointer>,
    procedure_context: Option<Pointer>,
    push_contexts: std::collections::HashMap<PushContextId, Pointer>,
}
enum Control {
    Next,
    Return(Vec<Value>),
    Break(LoopId),
    Continue(LoopId),
}

/// Opaque persistent virtual state, transferable between ready-provider snapshots.
pub struct VmState {
    memory: Memory,
    host_files: crate::host_files::HostFileMachine,
    host_heap: crate::virtual_heap::VirtualHeap,
    processes: Option<process::ProcessState>,
    globals: Vec<Option<Pointer>>,
    definitions: Vec<Global>,
    static_objects: std::collections::HashMap<StaticObjectId, Pointer>,
    static_publications: std::collections::HashMap<u64, usize>,
    static_data: std::collections::HashMap<u64, std::sync::Arc<StaticData>>,
    literal_backing: std::collections::HashMap<(TypeId, Value), Pointer>,
    registry: TypeId,
    limits: Limits,
    execution_phase: ExecutionPhase,
    default_context: Option<Pointer>,
    context_definition: Option<ContextDefinition>,
}
pub struct Vm<'a, P: ProcedureProvider + ?Sized, E: CompilerEffects> {
    provider: &'a P,
    effects: crate::effects::JournalEffects<E>,
    memory: Memory,
    host_files: crate::host_files::HostFileMachine,
    host_heap: crate::virtual_heap::VirtualHeap,
    processes: Option<process::ProcessState>,
    globals: Vec<Option<Pointer>>,
    frames: Vec<Frame<'a>>,
    limits: Limits,
    execution_phase: ExecutionPhase,
    statistics: Statistics,
    static_objects: std::collections::HashMap<StaticObjectId, Pointer>,
    static_publications: std::collections::HashMap<u64, usize>,
    static_data: std::collections::HashMap<u64, std::sync::Arc<StaticData>>,
    literal_backing: std::collections::HashMap<(TypeId, Value), Pointer>,
    current_context: Option<Pointer>,
    default_context: Option<Pointer>,
    root_temporaries: Vec<Pointer>,
    root_sequence_temp_bytes: u64,
    continuation: Option<resumable::Session>,
    expression_bindings: bindings::BindingEnvironment,
    process_branch_execution: bool,
    transaction_retained_cells: usize,
    transaction_ancillary_cells: usize,
    transaction_value_cell_limit: Option<usize>,
    publication_origin: Option<crate::SourceOrigin>,
    publication_work: std::cell::Cell<u64>,
}
impl<'a, P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'a, P, E> {
    pub fn new(provider: &'a P, effects: E, limits: Limits) -> std::result::Result<Self, Error> {
        Self::new_with_target(provider, effects, limits, crate::ByteTarget::default())
    }
    pub fn new_with_target(
        provider: &'a P,
        effects: E,
        limits: Limits,
        target: crate::ByteTarget,
    ) -> std::result::Result<Self, Error> {
        Self::new_with_execution_phase(
            provider,
            effects,
            limits,
            target,
            ExecutionPhase::CompileTime,
        )
    }
    pub fn new_with_execution_phase(
        provider: &'a P,
        effects: E,
        limits: Limits,
        target: crate::ByteTarget,
        execution_phase: ExecutionPhase,
    ) -> std::result::Result<Self, Error> {
        preflight_provider(provider, limits)?;
        let memory = Memory::with_target(limits, target);
        let mut globals = vec![];
        for (index, global) in provider.globals().iter().enumerate() {
            if global.id().index() != index {
                return Err(Error::InvalidIr("global IDs are not dense"));
            }
            globals.push(None);
        }
        Ok(Self {
            provider,
            effects: crate::effects::JournalEffects::new(effects),
            memory,
            host_files: crate::host_files::HostFileMachine::default(),
            host_heap: crate::virtual_heap::VirtualHeap::default(),
            processes: None,
            globals,
            frames: vec![],
            limits,
            execution_phase,
            statistics: Statistics::default(),
            static_objects: std::collections::HashMap::new(),
            static_publications: std::collections::HashMap::new(),
            static_data: std::collections::HashMap::new(),
            literal_backing: std::collections::HashMap::new(),
            current_context: None,
            default_context: None,
            root_temporaries: vec![],
            root_sequence_temp_bytes: 0,
            continuation: None,
            expression_bindings: bindings::BindingEnvironment::default(),
            process_branch_execution: false,
            transaction_retained_cells: 0,
            transaction_ancillary_cells: 0,
            transaction_value_cell_limit: None,
            publication_origin: None,
            publication_work: std::cell::Cell::new(0),
        })
    }
    /// Cancels an active continuation before transferring persistent state.
    /// Use `into_continuation` to retain suspended execution progress.
    pub fn into_state(self) -> VmState {
        self.try_into_state()
            .expect("effect rollback failed during VM state transfer")
    }
    pub fn try_into_state(mut self) -> std::result::Result<VmState, Error> {
        if self.continuation.is_some() {
            self.cancel_resumable()?;
        }
        Ok(self.take_state())
    }
    fn take_state(self) -> VmState {
        debug_assert!(self.frames.is_empty());
        VmState {
            memory: self.memory,
            host_files: self.host_files,
            host_heap: self.host_heap,
            processes: self.processes,
            static_objects: self.static_objects,
            static_publications: self.static_publications,
            static_data: self.static_data,
            literal_backing: self.literal_backing,
            globals: self.globals,
            definitions: self.provider.globals().to_vec(),
            registry: self.provider.types().scalar(jai_types::ScalarType::Bool),
            limits: self.limits,
            execution_phase: self.execution_phase,
            default_context: self.default_context,
            context_definition: self.provider.context().cloned(),
        }
    }
    pub fn with_state(
        provider: &'a P,
        effects: E,
        limits: Limits,
        state: VmState,
    ) -> std::result::Result<Self, Error> {
        Self::with_state_checked(provider, effects, limits, state, false)
    }
    fn with_state_checked(
        provider: &'a P,
        effects: E,
        limits: Limits,
        state: VmState,
        allow_open_files: bool,
    ) -> std::result::Result<Self, Error> {
        Self::validate_state(provider, limits, &state, allow_open_files)?;
        Ok(Self::from_validated_state(provider, effects, limits, state))
    }
    fn validate_state(
        provider: &P,
        limits: Limits,
        state: &VmState,
        allow_open_files: bool,
    ) -> std::result::Result<(), Error> {
        preflight_provider(provider, limits)?;
        if !allow_open_files {
            state.host_files.require_closed()?;
        }
        if state.registry != provider.types().scalar(jai_types::ScalarType::Bool) {
            return Err(Error::InvalidIr(
                "VM state belongs to another type registry",
            ));
        }
        if state.limits != limits {
            return Err(Error::InvalidIr("VM state uses different execution limits"));
        }
        if state.context_definition.as_ref() != provider.context() {
            return Err(Error::InvalidIr("VM state context definition changed"));
        }
        if state.definitions.len() > provider.globals().len() {
            return Err(Error::InvalidIr("VM state global definitions were removed"));
        }
        for (index, global) in provider.globals().iter().enumerate() {
            if global.id().index() != index {
                return Err(Error::InvalidIr("global IDs are not dense"));
            }
            if let Some(old) = state.definitions.get(index)
                && (old.ty() != global.ty()
                    || !same_initializer(old.initializer(), global.initializer()))
            {
                return Err(Error::InvalidIr("VM state global definition changed"));
            }
            if let Some(pointer) = state.globals.get(index).and_then(Option::as_ref)
                && let Some(requested) = provider
                    .storage_alignments()
                    .and_then(|alignments| alignments.global(global.id()))
                && (!requested.is_power_of_two()
                    || requested > state.memory.storage_alignment(pointer)?)
            {
                return Err(Error::InvalidIr(
                    "VM state global storage alignment changed",
                ));
            }
        }
        Ok(())
    }
    fn from_validated_state(
        provider: &'a P,
        effects: E,
        limits: Limits,
        mut state: VmState,
    ) -> Self {
        state.globals.resize(provider.globals().len(), None);
        Self {
            provider,
            effects: crate::effects::JournalEffects::new(effects),
            memory: state.memory,
            host_files: state.host_files,
            host_heap: state.host_heap,
            processes: state.processes,
            static_objects: state.static_objects,
            static_publications: state.static_publications,
            static_data: state.static_data,
            literal_backing: state.literal_backing,
            globals: state.globals,
            frames: vec![],
            limits,
            execution_phase: state.execution_phase,
            statistics: Statistics::default(),
            current_context: None,
            default_context: state.default_context,
            root_temporaries: vec![],
            root_sequence_temp_bytes: 0,
            continuation: None,
            expression_bindings: bindings::BindingEnvironment::default(),
            process_branch_execution: false,
            transaction_retained_cells: 0,
            transaction_ancillary_cells: 0,
            transaction_value_cell_limit: None,
            publication_origin: None,
            publication_work: std::cell::Cell::new(0),
        }
    }
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    pub fn configure_file_limits(
        &mut self,
        limits: crate::virtual_files::FileLimits,
    ) -> std::result::Result<(), Error> {
        if !self.frames.is_empty() || self.continuation.is_some() {
            return Err(Error::InvalidIr(
                "FILE limits cannot change during execution",
            ));
        }
        self.host_files.require_closed()?;
        self.host_files = crate::host_files::HostFileMachine::new(limits);
        Ok(())
    }
    pub fn memory_mut(&mut self) -> &mut Memory {
        assert!(
            self.continuation.is_none(),
            "cancel or finish the active continuation before mutating Memory"
        );
        &mut self.memory
    }
    /// Admit host-fed typed values using the same backing rules as language stores.
    pub fn allocate_storage(
        &mut self,
        ty: TypeId,
        value: Value,
    ) -> std::result::Result<Pointer, Error> {
        if self.continuation.is_some() {
            return Err(Error::InvalidIr(
                "host storage cannot change during an active continuation",
            ));
        }
        let action = (|| -> Result<Pointer> {
            value.validate(
                self.provider.types(),
                ty,
                self.limits.evaluation_depth.min(256),
            )?;
            self.charge_work(value.cells(self.limits.value_cells)?)?;
            let value = self.normalize_storage_value(value, 0)?;
            self.prepare_layout(ty)?;
            Ok(self
                .memory
                .allocate(self.provider.types(), ty, Some(value))?)
        })();
        action.map_err(|halt| match halt {
            Halt::Failed(error) => error,
            Halt::Pending(Dependency::Type(ty)) => Error::Type(TypeError::Incomplete(ty)),
            Halt::Pending(_) => Error::InvalidIr("host storage admission requires ready types"),
        })
    }
    pub fn configure_heap_limits(
        &mut self,
        limits: crate::virtual_heap::HeapLimits,
    ) -> std::result::Result<(), Error> {
        if !self.frames.is_empty()
            || self.continuation.is_some()
            || self.host_heap.allocation_count() != 0
        {
            return Err(Error::InvalidIr(
                "heap limits can change only with no active calls or live heap allocations",
            ));
        }
        self.host_heap = crate::virtual_heap::VirtualHeap::new(limits);
        Ok(())
    }
    pub fn effects(&self) -> &E {
        self.effects.inner()
    }
    pub fn effects_mut(&mut self) -> &mut E {
        self.effects.inner_mut()
    }
    /// Pin the scheduler's genuine directive facts before ordinary validation.
    pub fn pin_publication_source_origin(
        &mut self,
        origin: crate::SourceOrigin,
    ) -> std::result::Result<(), Error> {
        if self.continuation.is_some()
            || !self.frames.is_empty()
            || self.transaction_value_cell_limit.is_some()
        {
            return Err(Error::InvalidIr(
                "source publication origin changed during execution",
            ));
        }
        if self.publication_origin.is_none() {
            self.flush_publication_work().map_err(|halt| match halt {
                Halt::Failed(error) => error,
                Halt::Pending(_) => Error::InvalidIr("source preparation unexpectedly suspended"),
            })?;
            self.statistics = Statistics::default();
        }
        let cells = origin
            .path
            .capacity()
            .checked_add(origin.body.capacity())
            .and_then(|n| n.checked_add(origin.specialization.capacity()))
            .and_then(|n| n.checked_add(8))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let (retained, _) = resumable::publication_transaction_admission(self)?;
        retained
            .checked_mul(2)
            .and_then(|n| n.checked_add(self.publication_origin_cells()))
            .and_then(|n| n.checked_add(cells))
            .filter(|n| *n <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.charge_work(cells).map_err(|halt| match halt {
            Halt::Failed(error) => error,
            Halt::Pending(_) => Error::InvalidIr("source origin work unexpectedly suspended"),
        })?;
        self.publication_origin = Some(origin);
        Ok(())
    }
    pub(super) fn publication_origin_cells(&self) -> usize {
        self.publication_source_origin().map_or(0, |origin| {
            origin
                .path
                .capacity()
                .saturating_add(origin.body.capacity())
                .saturating_add(origin.specialization.capacity())
                .saturating_add(8)
        })
    }
    /// Successful runs retain globals. Pending/failed runs roll back memory and effects.
    pub fn execute(&mut self, procedure: ProcedureId, arguments: Vec<Value>) -> Execution {
        self.transaction(|vm| vm.invoke(procedure, arguments, 0))
    }
    /// Evaluate checked global/call expressions before a complete Program exists.
    pub fn evaluate(&mut self, expression: &ValueExpr) -> Execution {
        self.evaluate_validated(expression, |_, _| Ok(()))
    }
    /// Validate result publication while memory and compiler effects can still roll back.
    pub fn evaluate_validated(
        &mut self,
        expression: &ValueExpr,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
        self.evaluate_with_owner(None, expression, validator)
    }
    pub fn evaluate_owned(&mut self, owner: ProcedureId, expression: &ValueExpr) -> Execution {
        self.evaluate_owned_validated(owner, expression, |_, _| Ok(()))
    }
    pub fn evaluate_owned_validated(
        &mut self,
        owner: ProcedureId,
        expression: &ValueExpr,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
        self.evaluate_with_owner(Some(owner), expression, validator)
    }
    fn evaluate_with_owner(
        &mut self,
        owner: Option<ProcedureId>,
        expression: &ValueExpr,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
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
        if let Err(error) = proof {
            return verification_outcome(error);
        }
        self.transaction(|vm| {
            let values = vec![vm.value(expression, 0)?];
            vm.refresh_publication_transaction()?;
            validator(vm, &values)?;
            Ok(values)
        })
    }
    pub fn evaluate_call(&mut self, call: &Call) -> Execution {
        self.evaluate_call_validated(call, |_, _| Ok(()))
    }
    pub fn evaluate_call_validated(
        &mut self,
        call: &Call,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
        self.evaluate_call_with_owner(None, call, validator)
    }
    pub fn evaluate_call_owned(&mut self, owner: ProcedureId, call: &Call) -> Execution {
        self.evaluate_call_owned_validated(owner, call, |_, _| Ok(()))
    }
    pub fn evaluate_call_owned_validated(
        &mut self,
        owner: ProcedureId,
        call: &Call,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
        self.evaluate_call_with_owner(Some(owner), call, validator)
    }
    fn evaluate_call_with_owner(
        &mut self,
        owner: Option<ProcedureId>,
        call: &Call,
        validator: impl FnOnce(&Self, &[Value]) -> std::result::Result<(), Error>,
    ) -> Execution {
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
        if let Err(error) = proof {
            return verification_outcome(error);
        }
        self.transaction(|vm| {
            let values = vm.call(call, 0)?;
            vm.refresh_publication_transaction()?;
            validator(vm, &values)?;
            Ok(values)
        })
    }
    fn begin_publication_transaction(&mut self) -> Result<()> {
        let original = self.limits.value_cells;
        let (retained, ancillary) = resumable::publication_transaction_admission(self)?;
        self.charge_work(self.publication_origin_cells())?;
        let available = original
            .checked_sub(retained)
            .and_then(|n| n.checked_sub(ancillary))
            .and_then(|n| n.checked_sub(self.publication_origin_cells()))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.memory.replace_value_cell_limit(available)?;
        self.limits.value_cells = available;
        self.transaction_retained_cells = retained;
        self.transaction_ancillary_cells = ancillary;
        self.transaction_value_cell_limit = Some(original);
        Ok(())
    }
    fn refresh_publication_transaction(&mut self) -> Result<()> {
        let original = self.transaction_value_cell_limit.ok_or(Error::InvalidIr(
            "publication validation is outside an admitted transaction",
        ))?;
        let saved = self.limits.value_cells;
        self.limits.value_cells = original;
        let residual = resumable::publication_transaction_residual(self);
        self.limits.value_cells = saved;
        let ancillary = residual?;
        let available = original
            .checked_sub(self.transaction_retained_cells)
            .and_then(|n| n.checked_sub(ancillary))
            .and_then(|n| n.checked_sub(self.publication_origin_cells()))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.memory.replace_value_cell_limit(available)?;
        self.limits.value_cells = available;
        self.transaction_ancillary_cells = ancillary;
        Ok(())
    }
    fn finish_publication_transaction(&mut self) -> Result<()> {
        let flush = self.flush_publication_work();
        let restore = if let Some(original) = self.transaction_value_cell_limit.take() {
            self.limits.value_cells = original;
            self.memory
                .replace_value_cell_limit(original)
                .map(|_| ())
                .map_err(Halt::from)
        } else {
            Ok(())
        };
        self.transaction_retained_cells = 0;
        self.transaction_ancillary_cells = 0;
        self.publication_origin = None;
        flush.and(restore)
    }
    fn transaction(&mut self, action: impl FnOnce(&mut Self) -> Result<Vec<Value>>) -> Execution {
        if self.continuation.is_some() {
            return Execution {
                outcome: Outcome::Failed(Error::InvalidIr("a resumable transaction is active")),
                statistics: self.statistics,
            };
        }
        if self.publication_origin.is_none() {
            self.statistics = Statistics::default();
        }
        if let Err(halt) = self.begin_publication_transaction() {
            let _ = self.finish_publication_transaction();
            return Execution {
                outcome: Outcome::Failed(match halt {
                    Halt::Failed(error) => error,
                    Halt::Pending(_) => {
                        Error::InvalidIr("transaction admission unexpectedly suspended")
                    }
                }),
                statistics: self.statistics,
            };
        }
        debug_assert_eq!(self.expression_bindings.depth(), 0);
        if let Some(processes) = &self.processes {
            let work = usize::try_from(processes.work_cost()).unwrap_or(usize::MAX);
            if let Err(halt) = self.charge_work(work) {
                let _ = self.finish_publication_transaction();
                return Execution {
                    outcome: Outcome::Failed(match halt {
                        Halt::Failed(error) => error,
                        _ => unreachable!(),
                    }),
                    statistics: self.statistics,
                };
            }
        }
        debug_assert!(self.root_temporaries.is_empty());
        self.root_sequence_temp_bytes = 0;
        let snapshot = self.memory.snapshot();
        let files_snapshot = self.host_files.clone();
        let heap_snapshot = self.host_heap.clone();
        let process_snapshot = self.processes.clone();
        let global_snapshot = self.globals.clone();
        let static_snapshot = self.static_objects.clone();
        let publications_snapshot = self.static_publications.clone();
        let data_snapshot = self.static_data.clone();
        let literal_snapshot = self.literal_backing.clone();
        let context_snapshot = self.default_context.clone();
        self.effects.begin();
        let mut outcome = match self.initialize_context().and_then(|()| action(self)) {
            Ok(values) => Outcome::Complete(values),
            Err(Halt::Pending(dependency)) => Outcome::Pending(vec![dependency]),
            Err(Halt::Failed(error)) => Outcome::Failed(error),
        };
        if let Outcome::Complete(values) = &outcome
            && let Err(error) = self.validate_sequence_escape(values, &self.root_temporaries)
        {
            outcome = Outcome::Failed(error);
        }
        if matches!(outcome, Outcome::Complete(_))
            && let Err(error) = self.host_files.require_closed()
        {
            outcome = Outcome::Failed(error);
        }
        if matches!(outcome, Outcome::Complete(_))
            && let Some(processes) = &self.processes
            && let Err(error) = processes
                .world
                .require_quiescent(processes.branch.current())
        {
            outcome = Outcome::Failed(Error::EffectRejected(error.to_string()));
        }
        for pointer in self.root_temporaries.drain(..) {
            if let Err(error) = self.memory.release(&pointer) {
                outcome = Outcome::Failed(error);
            }
        }
        let complete = matches!(outcome, Outcome::Complete(_));
        if let Err(error) = self.effects.finish(complete) {
            outcome = Outcome::Failed(error);
        }
        let complete = matches!(outcome, Outcome::Complete(_));
        self.current_context = None;
        self.expression_bindings.clear();
        if !complete {
            self.memory.restore(snapshot);
            self.host_files = files_snapshot;
            self.host_heap = heap_snapshot;
            self.processes = process_snapshot;
            self.globals = global_snapshot;
            self.static_objects = static_snapshot;
            self.static_publications = publications_snapshot;
            self.static_data = data_snapshot;
            self.literal_backing = literal_snapshot;
            self.default_context = context_snapshot;
        }
        if let Err(halt) = self.finish_publication_transaction() {
            outcome = Outcome::Failed(match halt {
                Halt::Failed(error) => error,
                Halt::Pending(_) => {
                    Error::InvalidIr("publication accounting unexpectedly suspended")
                }
            });
        }
        debug_assert!(self.frames.is_empty());
        Execution {
            outcome,
            statistics: self.statistics,
        }
    }
    fn step(&mut self, depth: usize) -> Result<()> {
        self.flush_publication_work()?;
        // Conservative hard ceilings also protect the interpreter's Rust call stack.
        if depth > self.limits.evaluation_depth.min(256) {
            return Err(Error::Limit(LimitKind::EvaluationDepth).into());
        }
        if self.statistics.steps >= self.limits.fuel {
            return Err(Error::Limit(LimitKind::Fuel).into());
        }
        self.statistics.steps += 1;
        Ok(())
    }
    fn validate_provided_signature(&self, id: ProcedureId, signature: TypeId) -> Result<()> {
        if self.provider.signatures().get(&id) != Some(&signature) {
            return Err(
                Error::InvalidIr("provider procedure signature differs from declaration").into(),
            );
        }
        Ok(())
    }
    fn signature(&self, ty: TypeId) -> Result<ProcedureType> {
        let TypeKind::Procedure(id) = self.provider.types().kind(ty)? else {
            return Err(Error::InvalidIr("procedure signature is not a procedure type").into());
        };
        Ok(self.provider.types().procedure_type(*id)?.clone())
    }
    fn validate_values(&self, values: &[Value], expected: &[TypeId]) -> Result<()> {
        if values.len() != expected.len() {
            return Err(
                Error::InvalidIr("procedure argument/result count differs from signature").into(),
            );
        }
        let mut cells = 0usize;
        for (value, &ty) in values.iter().zip(expected) {
            value.cells(self.limits.value_cells)?;
            value.validate(
                self.provider.types(),
                ty,
                self.limits.evaluation_depth.min(256),
            )?;
            self.memory
                .validate_runtime_type_values(self.provider.types(), value)?;
            cells = cells
                .checked_add(value.cells(self.limits.value_cells)?)
                .filter(|n| *n <= self.limits.value_cells)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
        }
        Ok(())
    }
    fn invoke(
        &mut self,
        id: ProcedureId,
        arguments: Vec<Value>,
        depth: usize,
    ) -> Result<Vec<Value>> {
        self.step(depth)?;
        if self.frames.len() >= self.limits.stack_depth.min(128) {
            return Err(Error::Limit(LimitKind::StackDepth).into());
        }
        self.statistics.calls += 1;
        self.statistics.maximum_stack_depth = self
            .statistics
            .maximum_stack_depth
            .max(self.frames.len() + 1);
        self.invoke_available(id, arguments, depth, self.provider.procedure(id))
    }
    // The continuation engine pins this classification before dispatch, so an
    // interior-mutable provider cannot switch a capability into a source body.
    fn invoke_available(
        &mut self,
        id: ProcedureId,
        arguments: Vec<Value>,
        depth: usize,
        availability: ProcedureAvailability<'a>,
    ) -> Result<Vec<Value>> {
        self.require_procedure_phase(id)?;
        match availability {
            ProcedureAvailability::ProcessAbi(procedure) => {
                match self.invoke_process_available(id, &procedure, &arguments)? {
                    crate::process_source_machine::ProcessCallOutcome::Values(values) => Ok(values),
                    crate::process_source_machine::ProcessCallOutcome::Pending(event) => {
                        Err(Halt::Pending(Dependency::Process(event)))
                    }
                    _ => Err(Error::UnsupportedPointerOperation(
                        "process control transfer requires a branch continuation scheduler",
                    )
                    .into()),
                }
            }
            ProcedureAvailability::FileAbi(procedure) => {
                if self.process_branch_execution {
                    return Err(Error::UnsupportedPointerOperation(
                        "FILE operations after fork require shared open descriptions",
                    )
                    .into());
                }
                self.file_call(id, procedure, arguments)
            }
            ProcedureAvailability::HeapAbi(procedure) => self.heap_call(id, procedure, arguments),
            ProcedureAvailability::Pending(dependency) => Err(Halt::Pending(dependency)),
            ProcedureAvailability::Failed(error) => Err(error.into()),
            ProcedureAvailability::Missing => Err(Error::MissingProcedure(id).into()),
            ProcedureAvailability::Foreign => Err(Error::UnsupportedForeignProcedure(id).into()),
            ProcedureAvailability::Compiler(procedure) => {
                self.validate_provided_signature(id, procedure.signature)?;
                let signature = self.signature(procedure.signature)?;
                self.validate_values(&arguments, &signature.parameters)?;
                let arguments = self.materialize_values(&arguments)?;
                let mut work = 0u64;
                for argument in &arguments {
                    work = work
                        .checked_add(
                            u64::try_from(argument.cells(self.limits.value_cells)?)
                                .map_err(|_| Error::Limit(LimitKind::Fuel))?,
                        )
                        .ok_or(Error::Limit(LimitKind::Fuel))?;
                }
                self.charge_work(
                    usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                )?;
                let previous_context = self.enter_call_context(signature.context)?;
                let into_effect_error = |halt| match halt {
                    Halt::Pending(dependency) => EffectError::Pending(dependency),
                    Halt::Failed(error) => EffectError::Failed(error),
                };
                let result = match procedure.intrinsic {
                    crate::CompilerIntrinsic::SourceWriteStrings => self
                        .compiler_write_strings(&arguments)
                        .map_err(into_effect_error),
                    crate::CompilerIntrinsic::SourceVersionInfo {
                        record,
                    } => self
                        .compiler_version_info(record, &arguments)
                        .map_err(into_effect_error),
                    crate::CompilerIntrinsic::SourceWaitForMessage {
                        schema,
                    } => self
                        .compiler_wait_for_message(schema, &arguments)
                        .map_err(into_effect_error),
                    crate::CompilerIntrinsic::SourceRuntimeInfo {
                        current_workspace,
                        schema,
                    } => self
                        .compiler_runtime_info(current_workspace, schema, &arguments)
                        .map_err(into_effect_error),
                    intrinsic => {
                        intrinsic.invoke(&arguments, &mut self.effects, self.provider.types())
                    }
                };
                self.restore_call_context(previous_context);
                let values = result?;
                self.validate_values(&values, &signature.results)?;
                Ok(values)
            }
            ProcedureAvailability::Runtime(procedure) => {
                self.validate_provided_signature(id, procedure.signature)?;
                procedure
                    .intrinsic
                    .validate_signature_shape(procedure.signature, self.provider.types())
                    .map_err(Error::from)?;
                let signature = self.signature(procedure.signature)?;
                self.validate_values(&arguments, &signature.parameters)?;
                procedure.visit_pointer_layouts(&arguments, |pointer, include_pointee| {
                    self.prepare_pointer_layouts(pointer, include_pointee)
                })?;
                procedure.visit_additional_layouts(&arguments, self.provider.types(), |ty| {
                    self.prepare_layout(ty)
                })?;
                let work = procedure.work_cost_for_target(
                    &arguments,
                    self.provider.types(),
                    &self.memory,
                )?;
                self.charge_work(
                    usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                )?;
                let previous_context = self.enter_call_context(signature.context)?;
                let result = procedure.invoke(&arguments, &mut self.memory, self.provider.types());
                self.restore_call_context(previous_context);
                let values = result?;
                self.validate_values(&values, &signature.results)?;
                Ok(values)
            }
            ProcedureAvailability::Ready(checked) => {
                if checked.types().scalar(jai_types::ScalarType::Bool)
                    != self.provider.types().scalar(jai_types::ScalarType::Bool)
                    || (!std::ptr::eq(checked.signatures(), self.provider.signatures())
                        && checked.signatures() != self.provider.signatures())
                    || (!std::ptr::eq(checked.globals(), self.provider.globals())
                        && checked.globals() != self.provider.globals())
                    || checked.context() != self.provider.context()
                    || self
                        .provider
                        .places()
                        .map_or(!checked.places().is_empty(), |places| {
                            places != checked.places()
                        })
                {
                    return Err(Error::InvalidIr(
                        "checked procedure environment differs from provider",
                    )
                    .into());
                }
                let procedure = checked.procedure();
                if procedure.id != id {
                    return Err(Error::InvalidIr("provider returned the wrong procedure").into());
                }
                let signature = self.signature(procedure.signature)?;
                self.validate_values(&arguments, &signature.parameters)?;
                if procedure.parameters.len() != signature.parameters.len() {
                    return Err(Error::InvalidIr("parameter storage differs from signature").into());
                }
                let mut slots = vec![];
                for (index, local) in procedure.locals.iter().enumerate() {
                    if local.id().procedure() != id || local.id().index() != index {
                        return Err(
                            Error::InvalidIr("local IDs are not dense in their procedure").into(),
                        );
                    }
                    self.prepare_layout(local.ty())?;
                    slots.push(
                        self.memory.allocate_with_alignment(
                            self.provider.types(),
                            local.ty(),
                            None,
                            self.provider
                                .storage_alignments()
                                .and_then(|alignments| alignments.local(local.id()))
                                .unwrap_or(1),
                        )?,
                    );
                }
                for ((parameter, value), &ty) in procedure
                    .parameters
                    .iter()
                    .zip(arguments)
                    .zip(signature.parameters.iter())
                {
                    if parameter.id().procedure() != id || parameter.ty() != ty {
                        return Err(
                            Error::InvalidIr("parameter has the wrong owner or type").into()
                        );
                    }
                    let slot = slots
                        .get(parameter.id().index())
                        .ok_or(Error::InvalidIr("parameter storage is missing"))?;
                    self.store_pointer(slot, value, depth + 1)?;
                }
                let previous_context = self.enter_call_context(signature.context)?;
                self.frames.push(Frame {
                    procedure: FrameCode::Borrowed(procedure),
                    slots,
                    loops: vec![],
                    temporaries: vec![],
                    sequence_temp_bytes: 0,
                    sequence_temp_roots: vec![],
                    procedure_context: self.current_context.clone(),
                    push_contexts: std::collections::HashMap::new(),
                });
                self.statistics.maximum_stack_depth =
                    self.statistics.maximum_stack_depth.max(self.frames.len());
                let control = self.block(&procedure.body, depth + 1);
                let frame = self
                    .frames
                    .pop()
                    .ok_or(Error::InvalidIr("missing execution frame"))?;
                self.restore_call_context(previous_context);
                let escape = match &control {
                    Ok(Control::Return(values)) => {
                        self.validate_sequence_escape(values, &frame.sequence_temp_roots)
                    }
                    _ => Ok(()),
                };
                for slot in frame.slots.iter().chain(&frame.temporaries) {
                    self.memory.release(slot)?;
                }
                escape?;
                let values = match control? {
                    Control::Return(values) => values,
                    Control::Next if signature.results.is_empty() => vec![],
                    Control::Next => {
                        return Err(Error::InvalidIr(
                            "value procedure fell through without returning",
                        )
                        .into());
                    }
                    _ => return Err(Error::InvalidIr("loop transfer escaped procedure").into()),
                };
                self.validate_values(&values, &signature.results)?;
                Ok(values)
            }
        }
    }
    fn frame(&self) -> Result<&Frame<'a>> {
        self.frames
            .last()
            .ok_or_else(|| Error::InvalidIr("local access without an execution frame").into())
    }
    fn place(&mut self, place: Place, depth: usize) -> Result<Pointer> {
        let mut cursor = place;
        let mut cursor_depth = depth;
        let mut projections = Vec::new();
        loop {
            if cursor_depth > self.limits.evaluation_depth.min(256) {
                return Err(Error::Limit(LimitKind::EvaluationDepth).into());
            }
            let base = match cursor.kind() {
                PlaceKind::Field(id) => {
                    self.provider
                        .places()
                        .ok_or(Error::InvalidIr("provider has no projected places"))?
                        .projection(id)
                        .map_err(|_| Error::InvalidIr("invalid projected place"))?
                        .base
                }
                PlaceKind::Index(id) => {
                    self.provider
                        .places()
                        .ok_or(Error::InvalidIr("provider has no indices"))?
                        .index(id)
                        .map_err(|_| Error::InvalidIr("invalid index place"))?
                        .base
                }
                PlaceKind::SequenceField(id) => {
                    self.provider
                        .places()
                        .ok_or(Error::InvalidIr("provider has no sequence fields"))?
                        .sequence_field(id)
                        .map_err(|_| Error::InvalidIr("invalid sequence field place"))?
                        .base
                }
                _ => break,
            };
            if projections.len() >= self.limits.value_cells {
                return Err(Error::Limit(LimitKind::ValueCells).into());
            }
            self.charge_work(1)?;
            projections.push((cursor, cursor_depth));
            cursor = base;
            cursor_depth += 1;
        }
        let mut pointer = self.place_root(cursor, cursor_depth)?;
        for (projection, projection_depth) in projections.into_iter().rev() {
            pointer = self.project_place(projection, pointer, projection_depth)?;
        }
        Ok(pointer)
    }
    fn place_root(&mut self, place: Place, depth: usize) -> Result<Pointer> {
        let pointer = match place.kind() {
            PlaceKind::Context(record_type) => self.context_pointer(record_type)?,
            PlaceKind::Local(id) => {
                let frame = self.frame()?;
                if id.procedure() != frame.procedure.id {
                    return Err(Error::InvalidIr("local belongs to another procedure").into());
                }
                frame
                    .slots
                    .get(id.index())
                    .cloned()
                    .ok_or(Error::InvalidIr("missing local storage"))?
            }
            PlaceKind::Global(id) => {
                if self.provider.global_alignment_pending(id) {
                    return Err(Halt::Pending(Dependency::GlobalAlignment(id)));
                }
                if matches!(
                    self.provider
                        .globals()
                        .get(id.index())
                        .map(Global::initializer),
                    Some(GlobalInitializer::External(_))
                ) {
                    return Err(Error::UnsupportedExternalGlobal(id).into());
                }
                let slot = self
                    .globals
                    .get(id.index())
                    .ok_or(Error::InvalidIr("missing global storage"))?;
                if let Some(pointer) = slot {
                    pointer.clone()
                } else {
                    let global = self
                        .provider
                        .globals()
                        .get(id.index())
                        .ok_or(Error::InvalidIr("missing global definition"))?;
                    let value = self.global_value(global.initializer(), depth + 1)?;
                    let value = self.normalize_storage_value(value, depth + 1)?;
                    self.prepare_layout(global.ty())?;
                    let pointer = self.memory.allocate_with_alignment(
                        self.provider.types(),
                        global.ty(),
                        Some(value),
                        self.provider
                            .storage_alignments()
                            .and_then(|alignments| alignments.global(global.id()))
                            .unwrap_or(1),
                    )?;
                    self.globals[id.index()] = Some(pointer.clone());
                    pointer
                }
            }
            PlaceKind::Dereference(id) => {
                let expression = &self
                    .provider
                    .places()
                    .ok_or(Error::InvalidIr("provider has no dereferences"))?
                    .dereference(id)
                    .map_err(|_| Error::InvalidIr("invalid dereference place"))?
                    .pointer;
                self.value(expression, depth + 1)?.pointer()?.clone()
            }
            _ => return Err(Error::InvalidIr("projected place used as a storage root").into()),
        };
        if pointer.pointee() != place.ty() {
            return Err(Error::TypeMismatch {
                expected: place.ty(),
            }
            .into());
        }
        Ok(pointer)
    }
    fn project_place(&mut self, place: Place, base: Pointer, depth: usize) -> Result<Pointer> {
        let pointer = match place.kind() {
            PlaceKind::Index(id) => {
                let projection = self
                    .provider
                    .places()
                    .ok_or(Error::InvalidIr("provider has no indices"))?
                    .index(id)
                    .map_err(|_| Error::InvalidIr("invalid index place"))?;
                let fixed = matches!(
                    self.provider.types().kind(projection.base.ty())?,
                    TypeKind::FixedArray { .. }
                );
                let snapshot = if fixed {
                    None
                } else {
                    self.ensure_string_storage(&base, depth + 1)?;
                    self.prepare_pointer_layouts(&base, true)?;
                    self.charge_work(self.memory.load_work_cost(self.provider.types(), &base)?)?;
                    Some(self.memory.load(self.provider.types(), &base)?)
                };
                let index = self
                    .integer(&projection.index, depth + 1)?
                    .portable_integer()?
                    .value();
                let index = usize::try_from(index).map_err(|_| Error::OutOfBounds {
                    index: usize::MAX,
                    length: 0,
                })?;
                match self.provider.types().kind(projection.base.ty())? {
                    TypeKind::FixedArray {
                        ..
                    } => {
                        self.prepare_pointer_layouts(&base, true)?;
                        self.memory.index(self.provider.types(), &base, index)?
                    }
                    _ => {
                        let value = snapshot
                            .ok_or(Error::InvalidIr("missing indexed descriptor snapshot"))?;
                        let (pointer, count) = match value {
                            Value::Slice {
                                pointer,
                                count,
                                ..
                            }
                            | Value::DynamicArray {
                                pointer,
                                count,
                                ..
                            }
                            | Value::StringView {
                                pointer,
                                count,
                            } => (pointer, Some(count)),
                            Value::Pointer(pointer) => (pointer, None),
                            _ => {
                                return Err(Error::InvalidIr(
                                    "index place requires array, slice or pointer",
                                )
                                .into());
                            }
                        };
                        if let Some(count) = count
                            && projection.check.enabled()
                            && (count < 0 || index as u128 >= count as u128)
                        {
                            return Err(Error::OutOfBounds {
                                index,
                                length: usize::try_from(count.max(0)).unwrap_or(usize::MAX),
                            }
                            .into());
                        }
                        self.indexed_sequence_pointer(&pointer, index)?
                    }
                }
            }
            PlaceKind::SequenceField(id) => {
                let projection = *self
                    .provider
                    .places()
                    .ok_or(Error::InvalidIr("provider has no sequence fields"))?
                    .sequence_field(id)
                    .map_err(|_| Error::InvalidIr("invalid sequence field place"))?;
                self.ensure_string_storage(&base, depth + 1)?;
                self.prepare_pointer_layouts(&base, true)?;
                self.prepare_layout(place.ty())?;
                self.memory
                    .sequence_field(self.provider.types(), &base, projection.field)?
            }
            PlaceKind::Field(id) => {
                let places = self
                    .provider
                    .places()
                    .ok_or(Error::InvalidIr("provider has no projected places"))?;
                let projection = places
                    .projection(id)
                    .map_err(|_| Error::InvalidIr("invalid projected place"))?;
                self.provider
                    .types()
                    .validate_field(projection.base.ty(), projection.field)?;
                self.prepare_field_layouts(&base, projection.field.index())?;
                self.memory
                    .field(self.provider.types(), &base, projection.field.index())?
            }
            _ => return Err(Error::InvalidIr("storage root used as a projected place").into()),
        };
        if pointer.pointee() != place.ty() {
            return Err(Error::TypeMismatch {
                expected: place.ty(),
            }
            .into());
        }
        Ok(pointer)
    }
    fn load(&mut self, place: Place, depth: usize) -> Result<Value> {
        let pointer = self.place(place, depth)?;
        self.prepare_pointer_layouts(&pointer, true)?;
        self.charge_work(
            self.memory
                .load_work_cost(self.provider.types(), &pointer)?,
        )?;
        Ok(self.memory.load(self.provider.types(), &pointer)?)
    }
    fn store(&mut self, place: Place, value: Value, depth: usize) -> Result<()> {
        let pointer = self.place(place, depth)?;
        self.store_pointer(&pointer, value, depth + 1)
    }
    fn store_pointer(&mut self, pointer: &Pointer, value: Value, depth: usize) -> Result<()> {
        self.prepare_pointer_layouts(pointer, true)?;
        self.charge_work(
            self.memory
                .store_work_cost(self.provider.types(), pointer)?,
        )?;
        let value = self.normalize_storage_value(value, depth + 1)?;
        Ok(self.memory.store(self.provider.types(), pointer, value)?)
    }
    fn call(&mut self, call: &Call, depth: usize) -> Result<Vec<Value>> {
        self.call_arguments(call.procedure, &call.arguments, depth)
    }
    fn call_arguments(
        &mut self,
        procedure: ProcedureId,
        arguments: &[(ParameterId, ValueExpr)],
        depth: usize,
    ) -> Result<Vec<Value>> {
        self.step(depth)?;
        if arguments.len() > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells).into());
        }
        let mut values = Vec::with_capacity(arguments.len());
        let mut cells = 0;
        // Arguments/defaults already belong to checked IR. Preserve their source order.
        for (parameter, expression) in arguments {
            let value = self.value(expression, depth + 1)?;
            self.admit_value(&mut cells, &value)?;
            values.push((parameter.index(), value));
        }
        values.sort_by_key(|(parameter, _)| *parameter);
        if values
            .iter()
            .enumerate()
            .any(|(index, (parameter, _))| index != *parameter)
        {
            return Err(
                Error::InvalidIr("call parameter bindings are missing or duplicated").into(),
            );
        }
        self.invoke(
            procedure,
            values.into_iter().map(|(_, value)| value).collect(),
            depth + 1,
        )
    }
    fn indirect_call(
        &mut self,
        callee: &ValueExpr,
        arguments: &[(ParameterId, ValueExpr)],
        depth: usize,
    ) -> Result<Vec<Value>> {
        let Value::Procedure {
            signature,
            procedure,
        } = self.value(callee, depth + 1)?
        else {
            return Err(Error::InvalidIr("indirect callee is not a procedure value").into());
        };
        let procedure = procedure.ok_or(Error::NullProcedure)?;
        let actual = match self.provider.procedure(procedure) {
            ProcedureAvailability::Ready(checked) => checked.procedure().signature,
            ProcedureAvailability::Compiler(compiler) => compiler.signature,
            ProcedureAvailability::Runtime(runtime) => runtime.signature,
            ProcedureAvailability::FileAbi(file) => file.signature,
            ProcedureAvailability::HeapAbi(heap) => heap.signature,
            ProcedureAvailability::ProcessAbi(process) => process.signature(),
            ProcedureAvailability::Pending(dependency) => return Err(Halt::Pending(dependency)),
            ProcedureAvailability::Foreign => {
                return Err(Error::UnsupportedForeignProcedure(procedure).into());
            }
            ProcedureAvailability::Missing => return Err(Error::MissingProcedure(procedure).into()),
            ProcedureAvailability::Failed(error) => return Err(error.into()),
        };
        if signature != actual {
            return Err(Error::TypeMismatch {
                expected: actual,
            }
            .into());
        }
        self.call_arguments(procedure, arguments, depth + 1)
    }
    fn one(&mut self, call: &Call, depth: usize) -> Result<Value> {
        let mut values = self.call(call, depth)?;
        if values.len() != 1 {
            return Err(Error::InvalidIr("scalar call requires exactly one result").into());
        }
        Ok(values.remove(0))
    }
    fn integer(&mut self, expression: &IntExpr, depth: usize) -> Result<Number> {
        self.step(depth)?;
        let value = match expression.kind() {
            IntExprKind::FromPointer {
                value,
                mode,
            } => {
                let pointer = self.value(value, depth + 1)?.pointer()?.clone();
                self.prepare_pointer_layouts(&pointer, false)?;
                self.charge_work(pointer.metadata_cells())?;
                self.memory.pointer_to_integer(
                    self.provider.types(),
                    &pointer,
                    expression.ty(),
                    *mode,
                )?
            }
            IntExprKind::PointerDifference {
                left,
                right,
            } => {
                let left = self.value(left, depth + 1)?.pointer()?.clone();
                let right = self.value(right, depth + 1)?.pointer()?.clone();
                self.prepare_pointer_layouts(&left, true)?;
                self.prepare_pointer_layouts(&right, true)?;
                self.charge_work(left.metadata_cells().saturating_add(right.metadata_cells()))?;
                Number::plain(Integer::wrapping(
                    expression.ty(),
                    i128::from(self.memory.distance(self.provider.types(), &left, &right)?),
                ))
            }
            IntExprKind::FromFloat(mode, value) => Number::plain(crate::floats::to_integer(
                self.float(value, depth + 1)?,
                expression.ty(),
                *mode,
            )?),
            IntExprKind::Value(value) => self.value(value, depth + 1)?.number()?,
            IntExprKind::EnumValue(value) => match self.value(value, depth + 1)? {
                Value::Enum {
                    value, ..
                } => Number::plain(value),
                _ => return Err(Error::InvalidIr("enum conversion requires an enum value").into()),
            },
            IntExprKind::Constant(value) => Number::plain(*value),
            IntExprKind::InvalidCheckedCast => return Err(Error::CheckedCast.into()),
            IntExprKind::FromBool(source) => Number::plain(Integer::wrapping(
                expression.ty(),
                i128::from(self.boolean(source, depth + 1)?),
            )),
            IntExprKind::Load(place) => self.load(place.place(), depth + 1)?.number()?,
            IntExprKind::Call(call) => self.one(call, depth + 1)?.number()?,
            IntExprKind::Cast(mode, source) => {
                let number = self.integer(source, depth + 1)?;
                self.number_cast(expression.ty(), number, *mode)?
            }
            IntExprKind::Negate(source) => {
                let source = self.integer(source, depth + 1)?;
                self.charge_work(source.metadata_cells())?;
                scalar::negate_number(expression.ty(), source, expression.overflow_check())?
            }
            IntExprKind::Complement(source) => {
                let source = self.integer(source, depth + 1)?;
                self.charge_work(source.metadata_cells())?;
                scalar::complement_number(expression.ty(), source)?
            }
            IntExprKind::Binary(op, lhs, rhs) => {
                let lhs = self.integer(lhs, depth + 1)?;
                let rhs = self.integer(rhs, depth + 1)?;
                self.number_binary(expression.ty(), *op, lhs, rhs, expression.overflow_check())?
            }
            IntExprKind::Conditional(conditional) => {
                if self.boolean(&conditional.condition, depth + 1)? {
                    self.integer(&conditional.then_value, depth + 1)?
                } else {
                    self.integer(&conditional.else_value, depth + 1)?
                }
            }
        };
        if value.ty() != expression.ty() {
            return Err(Error::InvalidIr("integer expression result type differs").into());
        }
        Ok(value)
    }
    fn boolean(&mut self, expression: &BoolExpr, depth: usize) -> Result<bool> {
        self.step(depth)?;
        Ok(match expression {
            BoolExpr::CompileTime => self.execution_phase.is_compile_time(),
            BoolExpr::Value(value) => self.value(value, depth + 1)?.boolean()?,
            BoolExpr::Constant(value) => *value,
            BoolExpr::FromInt(expression) => {
                let number = self.integer(expression, depth + 1)?;
                self.number_truth(&number)?
            }
            BoolExpr::Load(place) => self.load(place.place(), depth + 1)?.boolean()?,
            BoolExpr::Call(call) => self.one(call, depth + 1)?.boolean()?,
            BoolExpr::Not(expression) => !self.boolean(expression, depth + 1)?,
            BoolExpr::CompareInts(op, lhs, rhs) => {
                let lhs = self.integer(lhs, depth + 1)?;
                let rhs = self.integer(rhs, depth + 1)?;
                self.compare_numbers(*op, &lhs, &rhs)?
            }
            BoolExpr::FromPointer(value) => match self.value(value, depth + 1)? {
                Value::Pointer(pointer) => self.pointer_truth(&pointer)?,
                Value::Procedure {
                    procedure, ..
                } => procedure.is_some(),
                _ => {
                    return Err(
                        Error::InvalidIr("pointer truth requires a pointer or procedure").into(),
                    );
                }
            },
            BoolExpr::ComparePointers(op, lhs, rhs) => {
                let lhs = self.value(lhs, depth + 1)?;
                let rhs = self.value(rhs, depth + 1)?;
                let equal = match (lhs, rhs) {
                    (Value::Pointer(lhs), Value::Pointer(rhs)) => {
                        self.prepare_pointer_layouts(&lhs, false)?;
                        self.prepare_pointer_layouts(&rhs, false)?;
                        self.charge_work(
                            lhs.metadata_cells().saturating_add(rhs.metadata_cells()),
                        )?;
                        self.memory
                            .same_address(self.provider.types(), &lhs, &rhs)?
                    }
                    (
                        Value::Procedure {
                            signature: lhs_sig,
                            procedure: lhs,
                        },
                        Value::Procedure {
                            signature: rhs_sig,
                            procedure: rhs,
                        },
                    ) if lhs_sig == rhs_sig => lhs == rhs,
                    _ => {
                        return Err(Error::InvalidIr(
                            "pointer comparison requires one exact pointer or procedure type",
                        )
                        .into());
                    }
                };
                match op {
                    Equality::Equal => equal,
                    Equality::NotEqual => !equal,
                }
            }
            BoolExpr::CompareStrings(op, lhs, rhs) => {
                self.compare_strings(*op, lhs, rhs, depth + 1)?
            }
            BoolExpr::CompareFloats(op, lhs, rhs) => {
                let lhs = self.float(lhs, depth + 1)?;
                let rhs = self.float(rhs, depth + 1)?;
                crate::floats::compare(*op, lhs, rhs)?
            }
            BoolExpr::CompareBools(op, lhs, rhs) => {
                let lhs = self.boolean(lhs, depth + 1)?;
                let rhs = self.boolean(rhs, depth + 1)?;
                match op {
                    Equality::Equal => lhs == rhs,
                    Equality::NotEqual => lhs != rhs,
                }
            }
            BoolExpr::And(lhs, rhs) => {
                self.boolean(lhs, depth + 1)? && self.boolean(rhs, depth + 1)?
            }
            BoolExpr::Or(lhs, rhs) => {
                self.boolean(lhs, depth + 1)? || self.boolean(rhs, depth + 1)?
            }
            BoolExpr::Conditional(conditional) => {
                if self.boolean(&conditional.condition, depth + 1)? {
                    self.boolean(&conditional.then_value, depth + 1)?
                } else {
                    self.boolean(&conditional.else_value, depth + 1)?
                }
            }
        })
    }
    fn block(&mut self, block: &Block, depth: usize) -> Result<Control> {
        self.step(depth)?;
        for statement in &block.statements {
            match self.statement(statement, depth + 1)? {
                Control::Next => {}
                control => return Ok(control),
            }
        }
        Ok(Control::Next)
    }
    fn cleanup(&mut self, id: CleanupId, depth: usize) -> Result<()> {
        let procedure = self.frame()?.procedure.clone();
        let cleanup = procedure
            .cleanups
            .get(id.index())
            .ok_or(Error::InvalidIr("missing cleanup body"))?;
        self.cleanup_in_context(cleanup.context, &cleanup.body, depth + 1)
    }
    fn statement(&mut self, statement: &Statement, depth: usize) -> Result<Control> {
        self.step(depth)?;
        match statement {
            Statement::Simd(block) => return self.simd(block, depth + 1),
            Statement::PushContext {
                id,
                value,
                body,
            } => {
                return self.push_context(*id, value, body, depth + 1);
            }
            Statement::IndirectCallResults {
                callee,
                arguments,
                destinations,
                ..
            } => {
                let pointers = destinations
                    .iter()
                    .map(|place| place.map(|place| self.place(place, depth + 1)).transpose())
                    .collect::<Result<Vec<_>>>()?;
                let values = self.indirect_call(callee, arguments, depth + 1)?;
                if values.len() != destinations.len() {
                    return Err(Error::InvalidIr(
                        "indirect call result destinations differ from result count",
                    )
                    .into());
                }
                for (value, pointer) in values.into_iter().zip(pointers) {
                    if let Some(pointer) = pointer {
                        self.store_pointer(&pointer, value, depth + 1)?;
                    }
                }
            }
            Statement::Store(place, expression) => {
                let pointer = self.place(*place, depth + 1)?;
                let value = self.value(expression, depth + 1)?;
                self.store_pointer(&pointer, value, depth + 1)?;
            }
            Statement::DiscardValue(expression) => {
                self.value(expression, depth + 1)?;
            }
            Statement::CallResults {
                call,
                destinations,
            } => {
                let pointers = destinations
                    .iter()
                    .map(|place| place.map(|place| self.place(place, depth + 1)).transpose())
                    .collect::<Result<Vec<_>>>()?;
                let values = self.call(call, depth + 1)?;
                if values.len() != destinations.len() {
                    return Err(Error::InvalidIr(
                        "call result destinations differ from result count",
                    )
                    .into());
                }
                for (value, pointer) in values.into_iter().zip(pointers) {
                    if let Some(pointer) = pointer {
                        self.store_pointer(&pointer, value, depth + 1)?;
                    }
                }
            }
            Statement::StoreInt(place, expression) => {
                let pointer = self.place(place.place(), depth + 1)?;
                let value = self.integer(expression, depth + 1)?;
                self.store_pointer(&pointer, value.into_value(), depth + 1)?;
            }
            Statement::StoreBool(place, expression) => {
                let pointer = self.place(place.place(), depth + 1)?;
                let value = self.boolean(expression, depth + 1)?;
                self.store_pointer(&pointer, Value::Bool(value), depth + 1)?;
            }
            Statement::DiscardInt(expression) => {
                self.integer(expression, depth + 1)?;
            }
            Statement::DiscardBool(expression) => {
                self.boolean(expression, depth + 1)?;
            }
            Statement::CallVoid(call) => {
                if !self.call(call, depth + 1)?.is_empty() {
                    return Err(Error::InvalidIr("void call returned values").into());
                }
            }
            Statement::Cleanup(id) => self.cleanup(*id, depth + 1)?,
            Statement::Block(block) => return self.block(block, depth + 1),
            Statement::If(condition, yes, no) => {
                return if self.boolean(condition, depth + 1)? {
                    self.block(yes, depth + 1)
                } else {
                    self.block(no, depth + 1)
                };
            }
            Statement::Exit(exit) => {
                // Return values are captured before any cleanup can mutate their storage.
                let control = match &exit.transfer {
                    Transfer::ReturnValues(expressions) => {
                        if expressions.len() > self.limits.value_cells {
                            return Err(Error::Limit(LimitKind::ValueCells).into());
                        }
                        let mut values = Vec::with_capacity(expressions.len());
                        let mut cells = 0;
                        for expression in expressions {
                            let value = self.value(expression, depth + 1)?;
                            self.push_value(&mut values, &mut cells, value)?;
                        }
                        Control::Return(values)
                    }
                    Transfer::ReturnVoid => Control::Return(vec![]),
                    Transfer::ReturnInt(expression) => {
                        Control::Return(vec![self.integer(expression, depth + 1)?.into_value()])
                    }
                    Transfer::ReturnBool(expression) => {
                        Control::Return(vec![Value::Bool(self.boolean(expression, depth + 1)?)])
                    }
                    Transfer::Break(id) | Transfer::Continue(id) => {
                        if !self.frame()?.loops.contains(id) {
                            return Err(
                                Error::InvalidIr("transfer targets an inactive loop").into()
                            );
                        }
                        match exit.transfer {
                            Transfer::Break(_) => Control::Break(*id),
                            _ => Control::Continue(*id),
                        }
                    }
                };
                for &cleanup in &exit.cleanups {
                    self.cleanup(cleanup, depth + 1)?;
                }
                return Ok(control);
            }
            Statement::While {
                id,
                condition,
                body,
            } => {
                self.enter_loop(*id)?;
                let result = self.while_loop(*id, condition, body, depth + 1);
                self.leave_loop()?;
                return result;
            }
            Statement::Range(range) => {
                self.enter_loop(range.id)?;
                let result = self.range_loop(range, depth + 1);
                self.leave_loop()?;
                return result;
            }
            Statement::Cases(cases) => return self.cases(cases, depth + 1),
        }
        Ok(Control::Next)
    }
    fn enter_loop(&mut self, id: LoopId) -> Result<()> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(Error::InvalidIr("loop without a frame"))?;
        if frame.loops.contains(&id) {
            return Err(Error::InvalidIr("duplicate active loop ID").into());
        }
        frame.loops.push(id);
        Ok(())
    }
    fn leave_loop(&mut self) -> Result<()> {
        self.frames
            .last_mut()
            .ok_or(Error::InvalidIr("loop without a frame"))?
            .loops
            .pop()
            .ok_or(Error::InvalidIr("missing active loop"))?;
        Ok(())
    }
    fn condition(&mut self, condition: &LoopCondition, depth: usize) -> Result<bool> {
        self.step(depth)?;
        match condition {
            LoopCondition::Value(expression) => self.boolean(expression, depth + 1),
            LoopCondition::BoundInt(local, expression) => {
                let value = self.integer(expression, depth + 1)?;
                self.store(local.place().place(), value.clone().into_value(), depth + 1)?;
                self.number_truth(&value)
            }
            LoopCondition::BoundBool(local, expression) => {
                let value = self.boolean(expression, depth + 1)?;
                self.store(local.place().place(), Value::Bool(value), depth + 1)?;
                Ok(value)
            }
        }
    }
    fn while_loop(
        &mut self,
        id: LoopId,
        condition: &LoopCondition,
        body: &Block,
        depth: usize,
    ) -> Result<Control> {
        while self.condition(condition, depth + 1)? {
            self.step(depth)?;
            match self.block(body, depth + 1)? {
                Control::Next => {}
                Control::Continue(target) if target == id => {}
                Control::Break(target) if target == id => return Ok(Control::Next),
                control => return Ok(control),
            }
        }
        Ok(Control::Next)
    }
    fn range_loop(&mut self, range: &RangeLoop, depth: usize) -> Result<Control> {
        let start = self.integer(&range.start, depth + 1)?;
        let end = self.integer(&range.end, depth + 1)?;
        start.portable_integer()?;
        end.portable_integer()?;
        if start.ty() != end.ty() || start.ty() != range.iterator.ty() {
            return Err(Error::InvalidIr("range endpoint/iterator types differ").into());
        }
        let reverse = range.direction == jai_ir::Direction::Reverse;
        let (first, last) = if reverse {
            (end.clone(), start.clone())
        } else {
            (start.clone(), end.clone())
        };
        self.store(
            range.iterator.place().place(),
            first.into_value(),
            depth + 1,
        )?;
        if start.value() > end.value() {
            return Ok(Control::Next);
        }
        loop {
            self.step(depth)?;
            match self.block(&range.body, depth + 1)? {
                Control::Next => {}
                Control::Continue(target) if target == range.id => {}
                Control::Break(target) if target == range.id => return Ok(Control::Next),
                control => return Ok(control),
            }
            let current = self
                .load(range.iterator.place().place(), depth + 1)?
                .number()?;
            current.portable_integer()?;
            if (!reverse && current.value() >= last.value())
                || (reverse && current.value() <= last.value())
            {
                return Ok(Control::Next);
            }
            let next = scalar::binary_number(
                current.ty(),
                jai_types::IntOp::Add,
                current.clone(),
                Number::plain(Integer::wrapping(
                    current.ty(),
                    if reverse {
                        -1
                    } else {
                        1
                    },
                )),
                CheckMode::Disabled,
                self.limits.value_cells,
            )?;
            self.store(range.iterator.place().place(), next.into_value(), depth + 1)?;
        }
    }
    fn cases(&mut self, cases: &Cases, depth: usize) -> Result<Control> {
        if !matches!(self.statement(&cases.subject, depth + 1)?, Control::Next) {
            return Err(Error::InvalidIr("case subject transferred control").into());
        }
        let mut through = false;
        for arm in &cases.arms {
            if through || self.boolean(&arm.condition, depth + 1)? {
                match self.block(&arm.body, depth + 1)? {
                    Control::Next => {}
                    control => return Ok(control),
                }
                if !arm.through {
                    return Ok(Control::Next);
                }
                through = true;
            }
        }
        if let Some(default) = &cases.default {
            return self.block(default, depth + 1);
        }
        if cases.exhaustive {
            return Err(Error::InvalidIr("exhaustive case did not match").into());
        }
        Ok(Control::Next)
    }
}

fn preflight_provider(
    provider: &(impl ProcedureProvider + ?Sized),
    limits: Limits,
) -> std::result::Result<(), Error> {
    crate::constants::preflight_provider_constants(provider.globals(), provider.context(), limits)?;
    let verify = |constant: &ConstantValue| {
        verify_constant_procedures(provider.types(), constant, provider.signatures()).map_err(
            |error| match error {
                IrError::Type(error) => Error::Type(error),
                error => Error::IrValidation(error.to_string()),
            },
        )
    };
    for global in provider.globals() {
        if let GlobalInitializer::Value(value) = global.initializer() {
            verify(value)?;
        }
    }
    if let Some(context) = provider.context() {
        verify(&context.default)?;
    }
    Ok(())
}

fn same_initializer(a: &GlobalInitializer, b: &GlobalInitializer) -> bool {
    match (a, b) {
        (GlobalInitializer::Int(a), GlobalInitializer::Int(b)) => a == b,
        (GlobalInitializer::Bool(a), GlobalInitializer::Bool(b)) => a == b,
        (GlobalInitializer::Value(a), GlobalInitializer::Value(b)) => a == b,
        (GlobalInitializer::External(a), GlobalInitializer::External(b)) => a == b,
        _ => false,
    }
}

fn verification_outcome(error: IrError) -> Execution {
    let outcome = match error {
        IrError::Type(TypeError::Incomplete(ty)) => Outcome::Pending(vec![Dependency::Type(ty)]),
        IrError::Type(error) => Outcome::Failed(Error::Type(error)),
        error => Outcome::Failed(Error::IrValidation(error.to_string())),
    };
    Execution {
        outcome,
        statistics: Statistics::default(),
    }
}
