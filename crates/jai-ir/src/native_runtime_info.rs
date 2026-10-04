//! Checked native ownership of a source runtime-info fallback and its table.
use crate::{
    Block, CheckedProcedure, ExternalData, ExternalDataId, ExternalDataSource, GlobalId,
    GlobalInitializer, PlaceKind, ProcedureId, RuntimeInfoSnapshot, SourceProcedureIdentity,
    Statement, Transfer, ValueExpr,
};
use jai_types::{CallingConvention, IntegerType, RuntimeInfoSchema, TypeKind, Variadic};
use std::{fmt, sync::Arc};

/// The source compiler catalog selects the fallback before constructing this
/// receipt. Native publication rechecks its exact checked body and storage;
/// neither an external symbol spelling nor a type-shaped global selects it.
#[derive(Clone, Debug)]
pub struct NativeRuntimeInfoPublication {
    procedure: ProcedureId,
    global: GlobalId,
    data: ExternalData,
    source: SourceProcedureIdentity,
    schema: RuntimeInfoSchema,
    storage: NativeRuntimeInfoStorage,
}

#[derive(Clone, Debug)]
enum NativeRuntimeInfoStorage {
    Ready(Arc<RuntimeInfoSnapshot>),
    Pending(NativeRuntimeInfoPending),
}

/// A real prerequisite of a proved source role, never an empty native table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeRuntimeInfoPending {
    TargetLayout,
}
impl fmt::Display for NativeRuntimeInfoPending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetLayout => f.write_str("a selected source target layout"),
        }
    }
}

#[derive(Debug)]
pub struct NativeRuntimeInfoError(String);
impl fmt::Display for NativeRuntimeInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for NativeRuntimeInfoError {
}
impl NativeRuntimeInfoError {
    pub(crate) fn invalid(message: &str) -> Self {
        Self(message.into())
    }
}

impl NativeRuntimeInfoPublication {
    /// Call only after origin and compiler-capability selection. This boundary
    /// independently verifies the final checked fallback and canonical storage.
    pub fn new_checked(
        checked: &CheckedProcedure<'_>,
        global: GlobalId,
        source: SourceProcedureIdentity,
        snapshot: Arc<RuntimeInfoSnapshot>,
    ) -> Result<Self, NativeRuntimeInfoError> {
        snapshot
            .revalidate(checked.types(), snapshot.policy())
            .map_err(|error| NativeRuntimeInfoError(error.to_string()))?;
        let mut publication =
            Self::new_pending_checked(checked, global, source, snapshot.schema())?;
        publication.storage = NativeRuntimeInfoStorage::Ready(snapshot);
        Ok(publication)
    }

    /// Source-only checks can retain the exact selected role before a target
    /// is selected. Native demand reports that prerequisite instead of
    /// attempting to initialize the external or silently dropping the role.
    pub fn new_pending_checked(
        checked: &CheckedProcedure<'_>,
        global: GlobalId,
        source: SourceProcedureIdentity,
        schema: RuntimeInfoSchema,
    ) -> Result<Self, NativeRuntimeInfoError> {
        let procedure = checked.procedure();
        schema
            .revalidate(checked.types())
            .map_err(|error| NativeRuntimeInfoError(error.to_string()))?;
        let signature = checked
            .types()
            .procedure_definition(procedure.signature)
            .map_err(|error| NativeRuntimeInfoError(error.to_string()))?;
        if signature.parameters.len() != 1
            || !matches!(
                checked.types().kind(signature.parameters[0]),
                Ok(TypeKind::Integer(IntegerType::S64))
            )
            || signature.results.as_ref() != [schema.ty()]
            || signature.convention != CallingConvention::Jai
            || signature.variadic != Variadic::None
        {
            return Err(NativeRuntimeInfoError::invalid(
                "native runtime-info requires the canonical (s64) -> Runtime_Info ABI",
            ));
        }
        if direct_return(&procedure.body).map(|place| (place.kind(), place.ty()))
            != Some((PlaceKind::Global(global), schema.ty()))
        {
            return Err(NativeRuntimeInfoError::invalid(
                "native runtime-info requires the exact checked direct returned global load",
            ));
        }
        let declaration = checked
            .globals()
            .get(global.index())
            .filter(|declaration| declaration.id() == global)
            .ok_or_else(|| {
                NativeRuntimeInfoError::invalid("native runtime-info global is absent")
            })?;
        let GlobalInitializer::External(data) = declaration.initializer() else {
            return Err(NativeRuntimeInfoError::invalid(
                "native runtime-info cannot adopt owned initial storage",
            ));
        };
        let location = data.location();
        let body = source.location();
        if data.ty() != schema.ty()
            || !matches!(data.source(), ExternalDataSource::Program)
            || !matches!(data.id(), ExternalDataId::Local { procedure: owner, .. } if owner == procedure.id)
            || location.source != body.source
            || location.span.start < body.span.start
            || location.span.end > body.span.end
        {
            return Err(NativeRuntimeInfoError::invalid(
                "native runtime-info external must belong to the retained source fallback and exact schema",
            ));
        }
        data.validate(checked.types())
            .map_err(|error| NativeRuntimeInfoError(error.to_string()))?;
        Ok(Self {
            procedure: procedure.id,
            global,
            data: data.clone(),
            source,
            schema,
            storage: NativeRuntimeInfoStorage::Pending(NativeRuntimeInfoPending::TargetLayout),
        })
    }
    pub fn procedure(&self) -> ProcedureId {
        self.procedure
    }
    pub fn global(&self) -> GlobalId {
        self.global
    }
    pub fn data(&self) -> &ExternalData {
        &self.data
    }
    pub fn source(&self) -> &SourceProcedureIdentity {
        &self.source
    }
    pub fn schema(&self) -> RuntimeInfoSchema {
        self.schema
    }
    pub fn snapshot(&self) -> Result<&Arc<RuntimeInfoSnapshot>, NativeRuntimeInfoPending> {
        match &self.storage {
            NativeRuntimeInfoStorage::Ready(snapshot) => Ok(snapshot),
            NativeRuntimeInfoStorage::Pending(prerequisite) => Err(*prerequisite),
        }
    }

    pub(crate) fn validate(&self, library: &crate::Library) -> Result<(), NativeRuntimeInfoError> {
        let checked = library.checked_procedure(self.procedure).ok_or_else(|| {
            NativeRuntimeInfoError::invalid(
                "native runtime-info fallback is absent from publication",
            )
        })?;
        let renewed = match &self.storage {
            NativeRuntimeInfoStorage::Ready(snapshot) => Self::new_checked(
                &checked,
                self.global,
                self.source.clone(),
                Arc::clone(snapshot),
            )?,
            NativeRuntimeInfoStorage::Pending(NativeRuntimeInfoPending::TargetLayout) => {
                Self::new_pending_checked(&checked, self.global, self.source.clone(), self.schema)?
            }
        };
        if renewed.data != self.data {
            return Err(NativeRuntimeInfoError::invalid(
                "native runtime-info external changed after checking",
            ));
        }
        Ok(())
    }
}

fn direct_return(block: &Block) -> Option<crate::Place> {
    let mut blocks = vec![(block, 0)];
    let mut returned = None;
    while let Some((block, index)) = blocks.pop() {
        let Some(statement) = block.statements.get(index) else {
            continue;
        };
        if returned.is_some() {
            return None;
        }
        blocks.push((block, index + 1));
        match statement {
            Statement::Block(nested) => blocks.push((nested, 0)),
            Statement::Exit(exit) if exit.cleanups.is_empty() => {
                let Transfer::ReturnValues(values) = &exit.transfer else {
                    return None;
                };
                let [ValueExpr::Load(place)] = values.as_slice() else {
                    return None;
                };
                returned = Some(*place);
            }
            _ => return None,
        }
    }
    returned
}
