//! Explicit foreign result ABI identity, independent of source record storage.
use crate::{ContextMode, ProcedureType, TypeError, TypeKind, TypeView};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ForeignReturnAbi {
    #[default]
    Natural,
    /// Microsoft C++ class result requiring the caller-provided return object.
    /// This is not an Itanium nontrivial-copy/destructor receipt.
    CppNonPod,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignReturnIssue {
    CallingConvention,
    Context,
    ResultCount,
    RecordResult,
    RecursiveRepresentation,
    RepresentationLimit,
}
pub(crate) fn validate(types: &dyn TypeView, signature: &ProcedureType) -> Result<(), TypeError> {
    if signature.return_abi == ForeignReturnAbi::Natural {
        return Ok(());
    }
    let issue = if !signature.convention.uses_c_abi() {
        Some(ForeignReturnIssue::CallingConvention)
    } else if signature.context != ContextMode::None {
        Some(ForeignReturnIssue::Context)
    } else if signature.results.len() != 1 {
        Some(ForeignReturnIssue::ResultCount)
    } else {
        let mut ty = signature.results[0];
        let mut seen = std::collections::HashSet::new();
        loop {
            if seen.len() == 65_536 {
                break Some(ForeignReturnIssue::RepresentationLimit);
            }
            if !seen.insert(ty) {
                break Some(ForeignReturnIssue::RecursiveRepresentation);
            }
            match types.kind(ty)? {
                TypeKind::Distinct(id) => ty = types.distinct(*id)?.representation,
                TypeKind::Record(_) => break None,
                _ => break Some(ForeignReturnIssue::RecordResult),
            }
        }
    };
    match issue {
        Some(issue) => Err(TypeError::InvalidForeignReturn(issue)),
        None => Ok(()),
    }
}
