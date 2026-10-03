//! Immutable checked programs shared by semantic lowering, virtual execution and LLVM.
mod construction;
mod context;
mod control;
mod debug_sources;
mod disposal;
mod expressions;
mod external_data;
pub use external_data::{
    ExternalData, ExternalDataError, ExternalDataId, ExternalDataSource, LocalExternalDataIndex,
};
mod expression_bindings;
pub use expression_bindings::ExpressionBindingId;
mod floats;
mod ordered_records;
pub use ordered_records::{OrderedRecordBacking, ordered_record_path_type};
mod native_pointer_constants;
pub use native_pointer_constants::{
    NativePointerConstant, NativePointerConstantError, NativePointerSource,
};
mod foreign_libraries;
mod procedure_hints;
mod procedure_phases;
pub use procedure_phases::ProcedurePhases;
mod program_exports;
mod runtime_info;
mod runtime_intrinsics;
mod runtime_types;
mod sequences;
mod source_procedure_owners;
mod source_warnings;
pub use source_procedure_owners::{
    CheckedSourceProcedureOwner, GlobalDefinitionsPrefix, SourceProcedureIdentity,
    SourceProcedureOwnerError, SourceProcedureOwners, SourceProcedurePlaces,
};
pub use source_warnings::SourceWarnings;
mod simd;
mod static_byte_views;
mod static_data;
pub use static_byte_views::{StaticByteView, StaticByteViewError};
mod static_values;
mod storage;
mod storage_alignments;
mod verify;
pub use construction::ProgramBuilder;
pub use context::*;
pub use control::*;
pub use debug_sources::*;
pub use expressions::*;
pub use floats::*;
pub use foreign_libraries::*;
use jai_source::DeclarationId;
pub use jai_types::{CastMode, Direction, Equality, IntOp, Relation};
pub use jai_types::{CheckMode, DebugPolicy};
use jai_types::{IntegerType, TypeError, TypeId, TypeView, Types};
pub use program_exports::*;
pub use runtime_info::{RuntimeInfoSnapshot, RuntimeInfoSnapshotError};
pub use runtime_intrinsics::{RuntimeIntrinsic, RuntimeIntrinsicError, atomic_scalar};
pub use runtime_types::{RuntimeTypeConstant, RuntimeTypeIdentity};
pub use sequences::*;
pub use simd::*;
pub use static_data::*;
pub use static_values::is_static_value;
use std::{collections::HashMap, fmt};
pub use storage::*;
pub use storage_alignments::StorageAlignments;
pub use verify::{
    CheckedCall, CheckedExpression, verify_call, verify_call_with_context,
    verify_constant_procedures, verify_expression, verify_expression_with_context,
    verify_owned_call_with_context, verify_owned_expression_with_context, verify_procedure,
    verify_procedure_with_context,
};

macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(usize);
        impl $name {
            /// Staging identity; the program finalizer checks its owner and index.
            #[doc(hidden)]
            pub fn new(index: usize) -> Self {
                Self(index)
            }
            pub fn index(self) -> usize {
                self.0
            }
        }
    };
}
id!(ProcedureId);
id!(ParameterId);
id!(GlobalId);
id!(LoopId);
id!(CleanupId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalId {
    procedure: ProcedureId,
    index: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PushContextId {
    procedure: ProcedureId,
    index: usize,
}
impl PushContextId {
    #[doc(hidden)]
    pub fn new(procedure: ProcedureId, index: usize) -> Self {
        Self { procedure, index }
    }
    pub fn procedure(self) -> ProcedureId {
        self.procedure
    }
    pub fn index(self) -> usize {
        self.index
    }
}
impl LocalId {
    pub fn index(self) -> usize {
        self.index
    }
    pub fn procedure(self) -> ProcedureId {
        self.procedure
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryPoint {
    Void(ProcedureId),
    Int(ProcedureId),
}

/// A validated compilation without an executable entry-point requirement.
#[derive(Debug)]
pub struct Library {
    source_procedure_owners: SourceProcedureOwners,
    source_warnings: SourceWarnings,
    storage_alignments: StorageAlignments,
    program_exports: Vec<ProgramExport>,
    procedure_hints: HashMap<ProcedureId, jai_types::InlineHint>,
    procedure_phases: ProcedurePhases,
    debug_sources: Option<DebugSources>,
    foreign_libraries: Vec<ForeignLibrary>,
    procedures: Vec<Procedure>,
    prototypes: Vec<ProcedurePrototype>,
    context: Option<ContextDefinition>,
    procedure_indices: HashMap<ProcedureId, usize>,
    globals: Vec<Global>,
    types: Types,
    places: Places,
    procedure_ids: HashMap<DeclarationId, ProcedureId>,
    signatures: HashMap<ProcedureId, TypeId>,
}
impl Drop for Library {
    fn drop(&mut self) {
        disposal::staging(
            std::mem::take(&mut self.procedures),
            std::mem::take(&mut self.globals),
            self.context.take(),
            &mut self.places,
        );
    }
}
impl Library {
    pub fn source_procedure_owners(&self) -> &SourceProcedureOwners {
        &self.source_procedure_owners
    }
    pub fn source_warnings(&self) -> &[jai_source::SourceWarning] {
        self.source_warnings.as_slice()
    }
    pub fn storage_alignments(&self) -> &StorageAlignments {
        &self.storage_alignments
    }
    pub fn program_exports(&self) -> &[ProgramExport] {
        &self.program_exports
    }
    /// Source inlining policy; definitions without an explicit hint use backend policy.
    pub fn inline_hint(&self, id: ProcedureId) -> jai_types::InlineHint {
        self.procedure_hints.get(&id).copied().unwrap_or_default()
    }
    pub fn procedure_phase(&self, id: ProcedureId) -> jai_types::ProcedureExecution {
        self.procedure_phases.get(id)
    }
    pub fn debug_sources(&self) -> Option<&DebugSources> {
        self.debug_sources.as_ref()
    }
    pub fn foreign_libraries(&self) -> &[ForeignLibrary] {
        &self.foreign_libraries
    }
    pub fn procedures(&self) -> &[Procedure] {
        &self.procedures
    }
    pub fn context(&self) -> Option<&ContextDefinition> {
        self.context.as_ref()
    }
    pub fn prototypes(&self) -> &[ProcedurePrototype] {
        &self.prototypes
    }
    pub fn procedure_by_id(&self, id: ProcedureId) -> Option<&Procedure> {
        self.procedure_indices
            .get(&id)
            .and_then(|index| self.procedures.get(*index))
    }
    pub fn globals(&self) -> &[Global] {
        &self.globals
    }
    pub fn types(&self) -> &Types {
        &self.types
    }
    pub fn places(&self) -> &Places {
        &self.places
    }
    pub fn signature(&self, id: ProcedureId) -> Option<TypeId> {
        self.signatures.get(&id).copied()
    }
    pub fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    pub fn checked_procedure(&self, id: ProcedureId) -> Option<CheckedProcedure<'_>> {
        self.procedure_by_id(id).map(|procedure| CheckedProcedure {
            procedure,
            types: &self.types,
            signatures: &self.signatures,
            globals: &self.globals,
            places: &self.places,
            context: self.context.as_ref(),
        })
    }
    pub fn procedure(&self, declaration: DeclarationId) -> Option<&Procedure> {
        self.procedure_ids
            .get(&declaration)
            .and_then(|id| self.procedure_by_id(*id))
    }
    pub fn into_program(self, entry: EntryPoint) -> Result<Program, IrError> {
        verify::entry(&self, entry)?;
        Ok(Program {
            library: self,
            entry,
        })
    }
}

/// A checked library with a separately checked executable entry.
#[derive(Debug)]
pub struct Program {
    library: Library,
    entry: EntryPoint,
}
impl Program {
    pub fn storage_alignments(&self) -> &StorageAlignments {
        self.library.storage_alignments()
    }
    pub fn procedures(&self) -> &[Procedure] {
        self.library.procedures()
    }
    pub fn globals(&self) -> &[Global] {
        self.library.globals()
    }
    pub fn types(&self) -> &Types {
        self.library.types()
    }
    pub fn places(&self) -> &Places {
        self.library.places()
    }
    pub fn context(&self) -> Option<&ContextDefinition> {
        self.library.context()
    }
    pub fn entry(&self) -> EntryPoint {
        self.entry
    }
    pub fn native_entry(&self) -> NativeEntryPoint {
        self.library
            .program_exports()
            .iter()
            .find_map(|export| match export.target {
                ExportTarget::Procedure(id) if export.symbol.as_str() == "main" => {
                    Some(NativeEntryPoint::ExportedProcedure(id))
                }
                _ => None,
            })
            .unwrap_or(NativeEntryPoint::Synthetic(self.entry))
    }
    pub fn library(&self) -> &Library {
        &self.library
    }
    pub fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.library.signatures()
    }
    pub fn procedure_by_id(&self, id: ProcedureId) -> Option<&Procedure> {
        self.library.procedure_by_id(id)
    }
    pub fn checked_procedure(&self, id: ProcedureId) -> Option<CheckedProcedure<'_>> {
        self.library.checked_procedure(id)
    }
}

/// Proof that a procedure and its ready signature environment have been checked.
pub struct CheckedProcedure<'a> {
    procedure: &'a Procedure,
    types: &'a dyn TypeView,
    signatures: &'a HashMap<ProcedureId, TypeId>,
    globals: &'a [Global],
    places: &'a Places,
    context: Option<&'a ContextDefinition>,
}
impl<'a> CheckedProcedure<'a> {
    pub fn procedure(&self) -> &'a Procedure {
        self.procedure
    }
    pub fn types(&self) -> &'a dyn TypeView {
        self.types
    }
    pub fn signatures(&self) -> &'a HashMap<ProcedureId, TypeId> {
        self.signatures
    }
    pub fn globals(&self) -> &'a [Global] {
        self.globals
    }
    pub fn places(&self) -> &'a Places {
        self.places
    }
    pub fn context(&self) -> Option<&'a ContextDefinition> {
        self.context
    }
}

/// A declaration whose implementation is supplied externally rather than a source block.
#[derive(Clone, Debug)]
pub struct ProcedurePrototype {
    pub id: ProcedureId,
    pub signature: TypeId,
    pub origin: PrototypeOrigin,
}
#[derive(Clone, Debug)]
pub enum PrototypeOrigin {
    SourceContract {
        symbol: String,
    },
    Foreign {
        symbol: String,
        library: Option<ForeignLibrary>,
    },
    Compiler,
    Intrinsic(RuntimeIntrinsic),
}
impl PrototypeOrigin {
    pub fn external_symbol(&self) -> Option<&str> {
        match self {
            Self::Foreign { symbol, .. } | Self::SourceContract { symbol } => Some(symbol),
            Self::Compiler | Self::Intrinsic(_) => None,
        }
    }
}

/// Staging data becomes executable only through `ProgramBuilder` validation.
#[derive(Clone, Debug)]
pub struct Procedure {
    pub id: ProcedureId,
    pub signature: TypeId,
    pub parameters: Vec<Local>,
    pub locals: Vec<Local>,
    pub body: Block,
    pub cleanups: Vec<Cleanup>,
}

#[derive(Debug)]
pub enum IrError {
    SourceProcedureOwner(Box<SourceProcedureOwnerError>),
    ExternalData(Box<ExternalDataError>),
    Simd(SimdError),
    StorageBitcast(jai_types::StorageBitcastError),
    InvalidStorageAlignment(u32),
    ProgramExport(ExportError),
    MissingContext,
    VerificationDepth,
    StaticData(Box<StaticDataError>),
    Type(TypeError),
    RuntimeIntrinsic(RuntimeIntrinsicError),
    UnknownIdentity {
        kind: &'static str,
        index: usize,
    },
    DuplicateIdentity {
        kind: &'static str,
        index: usize,
    },
    LocalOwner {
        expected: ProcedureId,
        actual: ProcedureId,
    },
    TypeMismatch {
        expected: TypeId,
        actual: TypeId,
    },
    IntegerMismatch {
        expected: IntegerType,
        actual: IntegerType,
    },
    Arity {
        kind: &'static str,
        expected: usize,
        actual: usize,
    },
    ForeignProjection(ProjectionId),
    ForeignPlaceProjection {
        kind: &'static str,
        index: usize,
    },
    InvalidFlow,
    InvalidEntry(ProcedureId),
    ReturnFromCleanup,
    CleanupCycle(CleanupId),
    InvalidValue(TypeId),
    InvalidConstant(TypeId),
    InvalidExhaustiveness,
}
impl From<TypeError> for IrError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<ExternalDataError> for IrError {
    fn from(error: ExternalDataError) -> Self {
        Self::ExternalData(Box::new(error))
    }
}
impl From<RuntimeIntrinsicError> for IrError {
    fn from(error: RuntimeIntrinsicError) -> Self {
        Self::RuntimeIntrinsic(error)
    }
}
impl From<StaticDataError> for IrError {
    fn from(error: StaticDataError) -> Self {
        Self::StaticData(Box::new(error))
    }
}
impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceProcedureOwner(error) => {
                write!(f, "invalid source procedure owner: {error}")
            }
            Self::ExternalData(error) => write!(f, "invalid IR external data: {error}"),
            Self::Simd(error) => write!(f, "invalid IR SIMD block: {error}"),
            Self::StorageBitcast(error) => write!(f, "invalid IR storage cast: {error}"),
            Self::InvalidStorageAlignment(alignment) => write!(
                f,
                "storage alignment {alignment} must be a nonzero power of two"
            ),
            Self::MissingContext => f.write_str("implicit context is unavailable"),
            Self::VerificationDepth => f.write_str("IR nesting exceeds verification limit"),
            Self::StaticData(error) => write!(f, "invalid IR static storage: {error}"),
            Self::Type(error) => write!(f, "invalid IR type: {error}"),
            Self::RuntimeIntrinsic(error) => write!(f, "invalid IR runtime intrinsic: {error}"),
            Self::ProgramExport(error) => write!(f, "invalid IR program export: {error}"),
            Self::UnknownIdentity { kind, index } => write!(f, "invalid {kind} identity {index}"),
            Self::DuplicateIdentity { kind, index } => {
                write!(f, "duplicate {kind} identity {index}")
            }
            Self::LocalOwner { expected, actual } => write!(
                f,
                "local belongs to procedure {}, expected {}",
                actual.index(),
                expected.index()
            ),
            Self::TypeMismatch { expected, actual } => {
                write!(f, "IR type mismatch: expected {expected:?}, got {actual:?}")
            }
            Self::IntegerMismatch { expected, actual } => write!(
                f,
                "IR integer mismatch: expected {expected:?}, got {actual:?}"
            ),
            Self::Arity {
                kind,
                expected,
                actual,
            } => write!(f, "IR {kind} count: expected {expected}, got {actual}"),
            Self::ForeignProjection(_) => {
                f.write_str("field projection belongs to another place registry")
            }
            Self::ForeignPlaceProjection { kind, index } => write!(
                f,
                "{kind} projection {index} belongs to another place registry"
            ),
            Self::InvalidFlow => {
                f.write_str("IR control-flow summary does not match its statements")
            }
            Self::InvalidEntry(_) => {
                f.write_str("IR executable entry must take no parameters and return int or void")
            }
            Self::ReturnFromCleanup => {
                f.write_str("IR cleanup cannot return from its enclosing procedure")
            }
            Self::CleanupCycle(_) => f.write_str("IR cleanup dependencies contain a cycle"),
            Self::InvalidValue(_) => f.write_str("IR value has no runtime representation"),
            Self::InvalidConstant(_) => f.write_str("IR constant does not match its declared type"),
            Self::InvalidExhaustiveness => {
                f.write_str("IR case exhaustiveness is not established by its dispatch")
            }
        }
    }
}
impl std::error::Error for IrError {}

pub use disposal::compiler_roots::{discard_call, discard_value_expression};
pub use verify::compiler_bindings::{
    verify_compiler_bound_call_with_context, verify_compiler_bound_expression_with_context,
    verify_compiler_slot_type,
};
