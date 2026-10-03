//! Private proof connecting a selected compiler fallback to its real external.
//!
//! This does not initialize native data. A later native publication must build
//! the actual descriptor table and global segments for the selected target.
use jai_ir::{
    Block, CheckedProcedure, ExternalData, ExternalDataId, ExternalDataSource, GlobalId,
    GlobalInitializer, PlaceKind, ProcedureId, SourceProcedureIdentity, Statement, Transfer,
    ValueExpr,
};
use jai_types::{CallingConvention, IntegerType, RuntimeInfoSchema, TypeKind, Variadic};
use jai_vm::{CompilerIntrinsic, CompilerProcedure, WorkspaceId};

#[derive(Clone, Debug)]
pub(crate) struct NativeRuntimeInfoRole {
    procedure: ProcedureId,
    global: GlobalId,
    data: ExternalData,
    source: SourceProcedureIdentity,
    schema: RuntimeInfoSchema,
    workspace: WorkspaceId,
}

impl NativeRuntimeInfoRole {
    /// The caller must supply the binding from the origin- and ABI-verified
    /// compiler catalog, rather than constructing one from a procedure name.
    pub(crate) fn from_checked_fallback(
        checked: &CheckedProcedure<'_>,
        binding: CompilerProcedure,
        source: SourceProcedureIdentity,
    ) -> Result<Option<Self>, String> {
        let CompilerIntrinsic::SourceRuntimeInfo {
            current_workspace,
            schema,
        } = binding.intrinsic
        else {
            return Ok(None);
        };
        let procedure = checked.procedure();
        if procedure.signature != binding.signature {
            return Err(
                "runtime-info fallback signature differs from its verified compiler binding".into(),
            );
        }
        let signature = checked
            .types()
            .procedure_definition(procedure.signature)
            .map_err(|error| error.to_string())?;
        if signature.parameters.len() != 1
            || !matches!(
                checked.types().kind(signature.parameters[0]),
                Ok(TypeKind::Integer(IntegerType::S64))
            )
            || signature.results.as_ref() != [schema.ty()]
            || signature.convention != CallingConvention::Jai
            || signature.variadic != Variadic::None
        {
            return Err(
                "runtime-info fallback requires the canonical (s64) -> Runtime_Info ABI".into(),
            );
        }
        schema
            .revalidate(checked.types())
            .map_err(|error| error.to_string())?;
        let Some(place) = direct_return(&procedure.body) else {
            return Err(
                "runtime-info fallback requires a checked direct return of its program external"
                    .into(),
            );
        };
        let PlaceKind::Global(global) = place.kind() else {
            return Err("runtime-info fallback does not return an external global".into());
        };
        let declaration = checked
            .globals()
            .get(global.index())
            .filter(|declaration| declaration.id() == global)
            .ok_or("runtime-info fallback references an unknown external global")?;
        let GlobalInitializer::External(data) = declaration.initializer() else {
            return Err("runtime-info fallback cannot use an owned initializer".into());
        };
        if data.ty() != schema.ty()
            || place.ty() != schema.ty()
            || !matches!(data.source(), ExternalDataSource::Program)
            || !matches!(data.id(), ExternalDataId::Local { procedure: owner, .. } if owner == procedure.id)
        {
            return Err("runtime-info external requires the same source procedure, program binding, and exact schema".into());
        }
        let body = source.location();
        let location = data.location();
        if location.source != body.source
            || location.span.start < body.span.start
            || location.span.end > body.span.end
        {
            return Err("runtime-info external is outside its retained source procedure".into());
        }
        data.validate(checked.types())
            .map_err(|error| error.to_string())?;
        Ok(Some(Self {
            procedure: procedure.id,
            global,
            data: data.clone(),
            source,
            schema,
            workspace: current_workspace,
        }))
    }

    pub(crate) fn procedure(&self) -> ProcedureId {
        self.procedure
    }
    pub(crate) fn global(&self) -> GlobalId {
        self.global
    }
    pub(crate) fn data(&self) -> &ExternalData {
        &self.data
    }
    pub(crate) fn source(&self) -> &SourceProcedureIdentity {
        &self.source
    }
    pub(crate) fn schema(&self) -> RuntimeInfoSchema {
        self.schema
    }
    pub(crate) fn workspace(&self) -> WorkspaceId {
        self.workspace
    }

    /// Preserve the catalog-selected role while rechecking the final immutable
    /// library. The snapshot cannot change the selected source storage schema.
    pub(crate) fn publication(
        &self,
        library: &jai_ir::Library,
        snapshot: std::sync::Arc<jai_ir::RuntimeInfoSnapshot>,
    ) -> Result<jai_ir::NativeRuntimeInfoPublication, String> {
        if snapshot.schema() != self.schema {
            return Err("runtime-info publication differs from its selected source schema".into());
        }
        let checked = library
            .checked_procedure(self.procedure)
            .ok_or("runtime-info selected fallback is absent from the final library")?;
        let publication = jai_ir::NativeRuntimeInfoPublication::new_checked(
            &checked,
            self.global,
            self.source.clone(),
            snapshot,
        )
        .map_err(|error| error.to_string())?;
        if publication.data() != &self.data {
            return Err("runtime-info selected external changed before publication".into());
        }
        Ok(publication)
    }
}

/// Only inert declaration blocks and the one cleanup-free returned load fit
/// this initial fallback contract. Nothing is skipped or rewritten by spelling.
fn direct_return(block: &Block) -> Option<jai_ir::Place> {
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

#[cfg(test)]
#[path = "native_runtime_info/tests.rs"]
mod tests;
