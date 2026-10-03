//! Bounded execution of independently checked Jai IR. No native code is loaded.
mod byte_memory;
mod constants;
mod effects;
mod execute;
pub mod file_abi;
pub mod file_tokens;
mod floats;
pub mod heap_abi;
pub mod host_effects;
pub mod host_files;
mod memory;
mod number;
pub mod process_abi;
pub mod process_source_machine;
mod runtime_intrinsics;
mod scalar;
mod stored_aggregate;
mod value;
pub mod virtual_files;
pub mod virtual_heap;
pub mod virtual_process;
pub use byte_memory::{ByteImage, ByteTarget, Endian};
pub use effects::*;
pub use execute::*;
pub use jai_ir::RuntimeTypeIdentity;
pub use memory::*;
pub use number::{AddressProvenance, Number};
pub use runtime_intrinsics::RuntimeProcedure;
pub use stored_aggregate::StoredAggregate;
pub use value::*;

use jai_ir::{ProcedureId, Program};
use jai_types::{FloatError, TypeError, TypeId};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub fuel: u64,
    pub stack_depth: usize,
    pub evaluation_depth: usize,
    pub allocations: usize,
    pub value_cells: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            fuel: 1_000_000,
            stack_depth: 128,
            evaluation_depth: 256,
            allocations: 16_384,
            value_cells: 1_000_000,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub steps: u64,
    pub calls: u64,
    pub maximum_stack_depth: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dependency {
    Procedure(ProcedureId),
    GlobalDefinitions,
    GlobalAlignment(jai_ir::GlobalId),
    Type(TypeId),
    Effect(EffectKey),
    Host(host_effects::HostRequestKey),
    Process(virtual_process::ProcessEvent),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LimitKind {
    Fuel,
    StackDepth,
    EvaluationDepth,
    Allocations,
    ValueCells,
    SequenceTemporaryBytes,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Limit(LimitKind),
    Type(TypeError),
    InvalidIr(&'static str),
    IrValidation(String),
    Layout(String),
    MissingProcedure(ProcedureId),
    Arithmetic(ArithmeticError),
    Float(FloatError),
    CheckedCast,
    Uninitialized,
    ReadOnlyStorage,
    NullPointer,
    NullProcedure,
    RuntimeTrap,
    SequenceTemporaryEscape,
    DanglingPointer,
    ForeignPointer,
    OutOfBounds {
        index: usize,
        length: usize,
    },
    TypeMismatch {
        expected: TypeId,
    },
    UnsupportedType(TypeId),
    UnsupportedPointerOperation(&'static str),
    UnsupportedForeignProcedure(ProcedureId),
    CompileTimeOnlyProcedure(ProcedureId),
    UnsupportedExternalGlobal(jai_ir::GlobalId),
    EffectRejected(String),
    CompilerReported(String),
    CompilerDiagnostic {
        location: SourceLocation,
        message: String,
    },
    EffectResponse(&'static str),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticError {
    ZeroDivisor,
    SignedDivisionOverflow,
    IntegerOverflow,
    ShiftCount,
}
impl From<FloatError> for Error {
    fn from(value: FloatError) -> Self {
        Self::Float(value)
    }
}
impl From<TypeError> for Error {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}
impl From<jai_types::LayoutError> for Error {
    fn from(value: jai_types::LayoutError) -> Self {
        match value {
            jai_types::LayoutError::Type(error) => Self::Type(error),
            error => Self::Layout(error.to_string()),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit(kind) => write!(f, "compile-time execution exceeded {kind:?} limit"),
            Self::Type(error) => error.fmt(f),
            Self::InvalidIr(reason) => write!(f, "invalid checked IR: {reason}"),
            Self::IrValidation(reason) => write!(f, "checked IR validation failed: {reason}"),
            Self::Layout(reason) => write!(f, "compile-time target layout failed: {reason}"),
            Self::MissingProcedure(id) => write!(f, "procedure {} is unavailable", id.index()),
            Self::Arithmetic(reason) => write!(f, "invalid compile-time arithmetic: {reason:?}"),
            Self::Float(error) => error.fmt(f),
            Self::CheckedCast => f.write_str("checked integer cast is out of range"),
            Self::ReadOnlyStorage => {
                f.write_str("write or release of immutable compile-time storage")
            }
            Self::Uninitialized => f.write_str("read of uninitialized compile-time storage"),
            Self::NullProcedure => f.write_str("call of null compile-time procedure value"),
            Self::RuntimeTrap => f.write_str("compile-time runtime debug trap"),
            Self::SequenceTemporaryEscape => {
                f.write_str("sequence pack backing escapes its caller frame")
            }
            Self::NullPointer => f.write_str("dereference of null compile-time pointer"),
            Self::DanglingPointer => f.write_str("compile-time pointer refers to released storage"),
            Self::ForeignPointer => {
                f.write_str("compile-time pointer belongs to another virtual memory")
            }
            Self::OutOfBounds {
                index,
                length,
            } => {
                write!(f, "compile-time index {index} is outside length {length}")
            }
            Self::TypeMismatch {
                ..
            } => f.write_str("compile-time value has the wrong type"),
            Self::UnsupportedPointerOperation(reason) => {
                write!(f, "unsupported virtual pointer operation: {reason}")
            }
            Self::UnsupportedType(_) => {
                f.write_str("compile-time operation does not support this type")
            }
            Self::UnsupportedForeignProcedure(id) => write!(
                f,
                "foreign procedure {} cannot execute in the compile-time VM",
                id.index()
            ),
            Self::CompileTimeOnlyProcedure(id) => write!(
                f,
                "procedure {} is compile-time only and cannot execute as a script",
                id.index()
            ),
            Self::UnsupportedExternalGlobal(id) => write!(
                f,
                "external global {} has no checked compile-time data provider",
                id.index()
            ),
            Self::CompilerDiagnostic {
                location,
                message,
            } => write!(
                f,
                "{}:{}:{}: {}",
                location.path.display(),
                location.line,
                location.column,
                message
            ),
            Self::CompilerReported(message) => write!(f, "compiler reported error: {message}"),
            Self::EffectRejected(reason) => write!(f, "compiler effect rejected: {reason}"),
            Self::EffectResponse(reason) => {
                write!(f, "compiler effect returned an invalid response: {reason}")
            }
        }
    }
}
impl std::error::Error for Error {
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Complete(Vec<Value>),
    Pending(Vec<Dependency>),
    Failed(Error),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Execution {
    pub outcome: Outcome,
    pub statistics: Statistics,
}

/// Execute a frozen program's selected entry point in an isolated VM.
pub fn execute(program: &Program, limits: Limits) -> Execution {
    let entry = match program.entry() {
        jai_ir::EntryPoint::Void(id) | jai_ir::EntryPoint::Int(id) => id,
    };
    match Vm::new(program, NoEffects, limits) {
        Ok(mut vm) => vm.execute(entry, vec![]),
        Err(error) => Execution {
            outcome: Outcome::Failed(error),
            statistics: Statistics::default(),
        },
    }
}

#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod memory_tests;
#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod constants_shape_tests;

#[cfg(test)]
mod provider_signature_tests;

pub mod compiler_code_plan;
pub use compiler_code_plan::{
    CompilerCodePlan, CompilerCodePlanBuilder, CompilerCodePlanId, CompilerCodePlanLimits,
    CompilerControl, CompilerControlId, CompilerReturnSiteId, CompilerRuntimeInput,
    CompilerRuntimeLeaf, CompilerRuntimeLeafKind, CompilerSlotId, CompilerSlotSchema,
};
