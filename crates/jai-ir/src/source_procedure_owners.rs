//! Retain checked compile-time source owners without publishing runtime bodies.
use crate::{
    CheckedProcedure, ConstantKind, ConstantValue, ContextDefinition, Global, GlobalInitializer,
    IrError, Places, Procedure, ProcedureId, storage, verify_procedure_with_context,
};
use jai_source::{SourceRecord, SourceSpan};
use jai_types::{ProcedureExecution, TypeId, TypeView};
use std::{collections::HashMap, fmt, path::Path, sync::Arc};

/// Complete dense file storage, sealed before local external slots are appended.
#[derive(Clone, Debug)]
pub struct GlobalDefinitionsPrefix(Arc<Prefix>);
#[derive(Debug)]
struct Prefix(Vec<Global>);
impl Drop for Prefix {
    fn drop(&mut self) {
        crate::disposal::globals(std::mem::take(&mut self.0));
    }
}
impl GlobalDefinitionsPrefix {
    /// The source scheduler supplies the reserved file-declaration count.
    pub fn new(
        globals: &[Global],
        expected_count: usize,
        types: &dyn TypeView,
        signatures: &HashMap<ProcedureId, TypeId>,
    ) -> Result<Self, SourceProcedureOwnerError> {
        if globals.len() != expected_count {
            return Err(SourceProcedureOwnerError::IncompleteGlobalDefinitions {
                expected: expected_count,
                actual: globals.len(),
            });
        }
        for (index, global) in globals.iter().enumerate() {
            if global.id().index() != index {
                return Err(SourceProcedureOwnerError::ChangedGlobals);
            }
            storage::runtime_type(types, global.ty())?;
            let scalar;
            let value = match global.initializer() {
                GlobalInitializer::Int(value) => {
                    scalar = ConstantValue {
                        ty: global.ty(),
                        kind: ConstantKind::Int(*value),
                    };
                    &scalar
                }
                GlobalInitializer::Bool(value) => {
                    scalar = ConstantValue {
                        ty: global.ty(),
                        kind: ConstantKind::Bool(*value),
                    };
                    &scalar
                }
                GlobalInitializer::Value(value) => {
                    if value.ty != global.ty() {
                        return Err(SourceProcedureOwnerError::ChangedGlobals);
                    }
                    value
                }
                GlobalInitializer::External(data) => {
                    data.validate(types).map_err(IrError::from)?;
                    continue;
                }
            };
            crate::verify::constant(types, value)?;
            crate::verify_constant_procedures(types, value, signatures)?;
        }
        Ok(Self(Arc::new(Prefix(globals.to_vec()))))
    }
    pub fn globals(&self) -> &[Global] {
        &self.0.0
    }
    fn validate_snapshot(&self, globals: &[Global]) -> Result<(), SourceProcedureOwnerError> {
        if globals.get(..self.globals().len()) != Some(self.globals()) {
            return Err(SourceProcedureOwnerError::ChangedGlobals);
        }
        Ok(())
    }
}

/// Source allocation provenance is independent of optional debug policy.
#[derive(Clone, Debug)]
pub struct SourceProcedureIdentity {
    location: SourceSpan,
    path: Arc<Path>,
    text: Arc<str>,
}

/// Share the exact checked place snapshot without recursively cloning unused operands.
#[derive(Clone, Debug)]
pub struct SourceProcedurePlaces(Arc<OwnedPlaces>);
#[derive(Debug)]
struct OwnedPlaces(Places);
impl Drop for OwnedPlaces {
    fn drop(&mut self) {
        crate::disposal::places(&mut self.0);
    }
}
impl SourceProcedurePlaces {
    pub fn new(places: Places) -> Self {
        Self(Arc::new(OwnedPlaces(places)))
    }
    pub fn places(&self) -> &Places {
        &self.0.0
    }
}
impl SourceProcedureIdentity {
    pub fn new(
        source: &SourceRecord,
        location: SourceSpan,
    ) -> Result<Self, SourceProcedureOwnerError> {
        let span = location.span;
        if source.id() != location.source
            || span.start > span.end
            || source.text().get(span.start..span.end).is_none()
        {
            return Err(SourceProcedureOwnerError::InvalidSource);
        }
        Ok(Self {
            location,
            path: Arc::from(source.path()),
            text: source.shared_text(),
        })
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn source_text(&self) -> &str {
        &self.text
    }
    pub fn matches_source(&self, source: &SourceRecord, location: SourceSpan) -> bool {
        self.location == location
            && source.id() == location.source
            && self.path() == source.path()
            && Arc::ptr_eq(&self.text, &source.shared_text())
    }
    pub fn body_text(&self) -> &str {
        &self.text[self.location.span.start..self.location.span.end]
    }
}

/// A body checked in its real environment, retained only as declaration ownership.
#[derive(Clone, Debug)]
pub struct CheckedSourceProcedureOwner(Arc<Owner>);
#[derive(Debug)]
struct Owner {
    procedure: Option<Procedure>,
    identity: SourceProcedureIdentity,
    globals: Vec<Global>,
    prefix: GlobalDefinitionsPrefix,
    places: SourceProcedurePlaces,
    context: Option<ContextDefinition>,
}
impl Drop for Owner {
    fn drop(&mut self) {
        crate::disposal::procedures(self.procedure.take().into_iter().collect());
        crate::disposal::globals(std::mem::take(&mut self.globals));
        if let Some(context) = self.context.take() {
            crate::disposal::constant(context.default);
        }
    }
}
impl CheckedSourceProcedureOwner {
    pub fn new(
        checked: CheckedProcedure<'_>,
        execution: ProcedureExecution,
        identity: SourceProcedureIdentity,
        prefix: GlobalDefinitionsPrefix,
        places: SourceProcedurePlaces,
    ) -> Result<Self, SourceProcedureOwnerError> {
        if execution != ProcedureExecution::CompileTimeOnly {
            return Err(SourceProcedureOwnerError::NotCompileTimeOnly);
        }
        if !std::ptr::eq(checked.places(), places.places()) {
            return Err(SourceProcedureOwnerError::ChangedPlaces);
        }
        prefix.validate_snapshot(checked.globals())?;
        for global in &checked.globals()[prefix.globals().len()..] {
            let GlobalInitializer::External(data) = global.initializer() else {
                return Err(SourceProcedureOwnerError::ChangedGlobals);
            };
            data.validate(checked.types()).map_err(IrError::from)?;
        }
        Ok(Self(Arc::new(Owner {
            procedure: Some(checked.procedure().clone()),
            identity,
            globals: checked.globals().to_vec(),
            prefix,
            places,
            context: checked.context().cloned(),
        })))
    }
    pub fn procedure(&self) -> &Procedure {
        self.0.procedure.as_ref().expect("live owner receipt")
    }
    pub fn id(&self) -> ProcedureId {
        self.procedure().id
    }
    pub fn signature(&self) -> TypeId {
        self.procedure().signature
    }
    pub fn identity(&self) -> &SourceProcedureIdentity {
        &self.0.identity
    }
    pub fn execution(&self) -> ProcedureExecution {
        ProcedureExecution::CompileTimeOnly
    }
    /// Final publication rechecks body types against the real combined signature ledger.
    pub fn revalidate(
        &self,
        types: &dyn TypeView,
        signatures: &HashMap<ProcedureId, TypeId>,
        globals: &[Global],
        context: Option<&ContextDefinition>,
    ) -> Result<(), SourceProcedureOwnerError> {
        self.0.prefix.validate_snapshot(globals)?;
        if globals.get(..self.0.globals.len()) != Some(self.0.globals.as_slice()) {
            return Err(SourceProcedureOwnerError::ChangedGlobals);
        }
        if context != self.0.context.as_ref() {
            return Err(SourceProcedureOwnerError::ChangedContext);
        }
        verify_procedure_with_context(
            types,
            self.procedure(),
            signatures,
            globals,
            self.0.places.places(),
            context,
        )?;
        Ok(())
    }
}

/// Separate from runtime procedure/prototype and native reachability ledgers.
#[derive(Clone, Debug, Default)]
pub struct SourceProcedureOwners(HashMap<ProcedureId, CheckedSourceProcedureOwner>);
impl SourceProcedureOwners {
    pub fn get(&self, id: ProcedureId) -> Option<&CheckedSourceProcedureOwner> {
        self.0.get(&id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &CheckedSourceProcedureOwner> {
        self.0.values()
    }
    /// A rejected replacement leaves the prior certified owner unchanged.
    pub fn insert(
        &mut self,
        owner: CheckedSourceProcedureOwner,
    ) -> Result<(), SourceProcedureOwnerError> {
        if let Some(previous) = self.get(owner.id()) {
            if Arc::ptr_eq(&previous.0, &owner.0) {
                return Ok(());
            }
            return Err(SourceProcedureOwnerError::DuplicateOwner(owner.id()));
        }
        self.0.insert(owner.id(), owner);
        Ok(())
    }
    pub fn validate(
        &self,
        types: &dyn TypeView,
        runtime_signatures: &HashMap<ProcedureId, TypeId>,
        globals: &[Global],
        context: Option<&ContextDefinition>,
    ) -> Result<(), SourceProcedureOwnerError> {
        let mut signatures = runtime_signatures.clone();
        for owner in self.iter() {
            if signatures.insert(owner.id(), owner.signature()).is_some() {
                return Err(SourceProcedureOwnerError::DuplicateOwner(owner.id()));
            }
        }
        for owner in self.iter() {
            owner.revalidate(types, &signatures, globals, context)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SourceProcedureOwnerError {
    InvalidSource,
    NotCompileTimeOnly,
    IncompleteGlobalDefinitions { expected: usize, actual: usize },
    ChangedGlobals,
    ChangedContext,
    ChangedPlaces,
    DuplicateOwner(ProcedureId),
    Ir(IrError),
}
impl From<IrError> for SourceProcedureOwnerError {
    fn from(error: IrError) -> Self {
        Self::Ir(error)
    }
}
impl fmt::Display for SourceProcedureOwnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSource => {
                formatter.write_str("source procedure owner has an invalid source identity or span")
            }
            Self::NotCompileTimeOnly => {
                formatter.write_str("source-only procedure owner must be compile-time-only")
            }
            Self::IncompleteGlobalDefinitions { expected, actual } => write!(
                formatter,
                "source procedure owner waits for {expected} file globals; {actual} are defined"
            ),
            Self::ChangedGlobals => {
                formatter.write_str("source procedure owner's original dense global prefix changed")
            }
            Self::ChangedContext => {
                formatter.write_str("source procedure owner's checked context changed")
            }
            Self::ChangedPlaces => formatter
                .write_str("source procedure owner requires its exact checked place snapshot"),
            Self::DuplicateOwner(id) => write!(
                formatter,
                "source procedure owner {id:?} collides with an existing declaration"
            ),
            Self::Ir(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for SourceProcedureOwnerError {}

#[cfg(test)]
mod tests;
