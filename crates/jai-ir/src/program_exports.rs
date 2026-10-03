//! Native export identity and entry policy are independent of debug information.
use crate::{EntryPoint, GlobalId, Library, ProcedureId};
use jai_source::DeclarationId;
use jai_types::{
    CallingConvention, ContextMode, IntegerType, ScalarType, TypeError, TypeKind, TypeView,
    Variadic,
};
use std::{collections::HashSet, fmt};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NativeSymbol(String);
impl NativeSymbol {
    pub fn new(name: impl Into<String>) -> Result<Self, ExportError> {
        let name = name.into();
        if name.is_empty() || name.as_bytes().contains(&0) {
            return Err(ExportError::InvalidSymbol);
        }
        if name.starts_with("jai.") || name.starts_with("llvm.") {
            return Err(ExportError::ReservedSymbol(name));
        }
        Ok(Self(name))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportTarget {
    Procedure(ProcedureId),
    Global(GlobalId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramExport {
    pub declaration: DeclarationId,
    pub target: ExportTarget,
    pub symbol: NativeSymbol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeEntryPoint {
    Synthetic(EntryPoint),
    ExportedProcedure(ProcedureId),
}

#[derive(Debug)]
pub enum ExportError {
    InvalidSymbol,
    ReservedSymbol(String),
    DuplicateSymbol(String),
    DuplicateDeclaration(DeclarationId),
    DuplicateTarget(ExportTarget),
    DeclarationTargetMismatch(DeclarationId),
    UnknownTarget(ExportTarget),
    ConflictingForeignSymbol(String),
    InvalidMain,
    Type(TypeError),
}
impl From<TypeError> for ExportError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSymbol => f.write_str("program export symbol must be nonempty and contain no NUL"),
            Self::ReservedSymbol(symbol) => write!(f, "program export symbol {symbol:?} uses a reserved compiler namespace"),
            Self::DuplicateSymbol(symbol) => write!(f, "conflicting program export symbol {symbol:?}"),
            Self::DuplicateDeclaration(declaration) => write!(f, "duplicate program export declaration {}", declaration.index()),
            Self::DuplicateTarget(target) => write!(f, "duplicate program export target {target:?}"),
            Self::DeclarationTargetMismatch(declaration) => write!(f, "program export declaration {} does not identify its published target", declaration.index()),
            Self::UnknownTarget(target) => write!(f, "program export target {target:?} has no checked definition"),
            Self::ConflictingForeignSymbol(symbol) => write!(f, "program export symbol {symbol:?} conflicts with a foreign prototype"),
            Self::InvalidMain => f.write_str("exported main must be a #c_call procedure returning s32 with no parameters or (s32, **u8) parameters"),
            Self::Type(error) => write!(f, "program export type: {error}"),
        }
    }
}
impl std::error::Error for ExportError {
}

pub(crate) fn validate(library: &Library, exports: &[ProgramExport]) -> Result<(), ExportError> {
    let mut symbols = HashSet::new();
    let mut declarations = HashSet::new();
    let mut targets = HashSet::new();
    for export in exports {
        NativeSymbol::new(export.symbol.as_str())?;
        if !symbols.insert(export.symbol.as_str()) {
            return Err(ExportError::DuplicateSymbol(export.symbol.as_str().into()));
        }
        if !declarations.insert(export.declaration) {
            return Err(ExportError::DuplicateDeclaration(export.declaration));
        }
        if !targets.insert(export.target) {
            return Err(ExportError::DuplicateTarget(export.target));
        }
        if let Some(id) = library.procedure_ids.get(&export.declaration)
            && export.target != ExportTarget::Procedure(*id)
        {
            return Err(ExportError::DeclarationTargetMismatch(export.declaration));
        }
        match export.target {
            ExportTarget::Procedure(id) if library.procedure_by_id(id).is_some() => {}
            ExportTarget::Global(id)
                if library
                    .globals()
                    .get(id.index())
                    .is_some_and(|global| global.id() == id) => {}
            target => return Err(ExportError::UnknownTarget(target)),
        }
        for prototype in library.prototypes() {
            if let Some(symbol) = prototype.origin.external_symbol()
                && symbol == export.symbol.as_str()
            {
                let compatible = match export.target {
                    ExportTarget::Procedure(id) => library
                        .procedure_by_id(id)
                        .is_some_and(|procedure| procedure.signature == prototype.signature),
                    ExportTarget::Global(_) => false,
                };
                if !compatible {
                    return Err(ExportError::ConflictingForeignSymbol(
                        export.symbol.as_str().into(),
                    ));
                }
            }
        }
        if export.symbol.as_str() == "main" {
            validate_main(library, export.target)?;
        }
    }
    Ok(())
}

fn validate_main(library: &Library, target: ExportTarget) -> Result<(), ExportError> {
    let ExportTarget::Procedure(id) = target else {
        return Err(ExportError::InvalidMain);
    };
    let procedure = library
        .procedure_by_id(id)
        .ok_or(ExportError::UnknownTarget(target))?;
    let types = library.types();
    let signature = types.procedure_definition(procedure.signature)?;
    validate_main_signature(signature, types)
}

pub fn validate_main_signature(
    signature: &jai_types::ProcedureType,
    types: &dyn TypeView,
) -> Result<(), ExportError> {
    let integer = types.scalar(ScalarType::Int(IntegerType::S32));
    if signature.convention != CallingConvention::C
        || signature.context != ContextMode::None
        || signature.variadic != Variadic::None
        || signature.results.as_ref() != [integer]
    {
        return Err(ExportError::InvalidMain);
    }
    if signature.parameters.is_empty() {
        return Ok(());
    }
    let [argc, argv] = signature.parameters.as_ref() else {
        return Err(ExportError::InvalidMain);
    };
    if *argc != integer {
        return Err(ExportError::InvalidMain);
    }
    let TypeKind::Pointer(pointer) = types.kind(*argv)? else {
        return Err(ExportError::InvalidMain);
    };
    let TypeKind::Pointer(byte) = types.kind(*pointer)? else {
        return Err(ExportError::InvalidMain);
    };
    if !matches!(
        types.kind(*byte)?,
        TypeKind::Integer(IntegerType::U8 | IntegerType::S8)
    ) {
        return Err(ExportError::InvalidMain);
    }
    Ok(())
}
