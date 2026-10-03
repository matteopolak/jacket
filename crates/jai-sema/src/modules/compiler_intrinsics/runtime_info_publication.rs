//! Finish-time native publication from proved source fallback roles.
use super::*;
use jai_ir::{Global, Library, Places, Procedure, SourceProcedureIdentity};
use jai_types::{LayoutPolicy, ReflectionReadiness};
use std::sync::Arc;

pub(in crate::modules) struct PreparedRuntimeInfo {
    roles: Vec<(
        native_runtime_info::NativeRuntimeInfoRole,
        Arc<jai_ir::RuntimeInfoSnapshot>,
    )>,
}

pub(in crate::modules) struct RuntimeInfoPublicationInput<'a> {
    pub graph: &'a ModuleGraph,
    pub declarations: &'a ScopedDeclarations<'a>,
    pub compiler: &'a HashMap<ProcedureId, CompilerProcedure>,
    pub procedures: &'a [Procedure],
    pub signatures: &'a HashMap<ProcedureId, TypeId>,
    pub globals: &'a [Global],
    pub places: &'a Places,
    pub context: Option<&'a crate::context::Schema>,
    pub layout: Option<LayoutPolicy>,
}

/// Only a catalog-bound declaration with a real checked body is a candidate.
/// Descriptor allocation occurs while the canonical arena is still mutable.
pub(in crate::modules) fn prepare(
    input: RuntimeInfoPublicationInput<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<PreparedRuntimeInfo, LocatedDiagnostic> {
    let mut roles = Vec::new();
    for declaration in input.graph.declarations() {
        let Some(signature) = input.declarations.signatures.get(&declaration.id()) else {
            continue;
        };
        let Some(binding) = input.compiler.get(&signature.id).copied() else {
            continue;
        };
        if !matches!(
            binding.intrinsic,
            CompilerIntrinsic::SourceRuntimeInfo { .. }
        ) {
            continue;
        }
        let Some(procedure) = input
            .procedures
            .iter()
            .find(|procedure| procedure.id == signature.id)
        else {
            // A declaration without a body grants compile-time access only.
            continue;
        };
        let location = declaration.location();
        let source = input.graph.sources().get(location.source).ok_or_else(|| {
            input
                .graph
                .diagnostic(location, "runtime-info fallback has no original source")
        })?;
        let source = SourceProcedureIdentity::new(source, location)
            .map_err(|error| input.graph.diagnostic(location, error.to_string()))?;
        let role = {
            let checked = jai_ir::verify_procedure_with_context(
                types,
                procedure,
                input.signatures,
                input.globals,
                input.places,
                input.context.map(|context| &context.definition),
            )
            .map_err(|error| input.graph.diagnostic(location, error.to_string()))?;
            native_runtime_info::NativeRuntimeInfoRole::from_checked_fallback(
                &checked, binding, source,
            )
            .map_err(|error| input.graph.diagnostic(location, error))?
            .ok_or_else(|| {
                input
                    .graph
                    .diagnostic(location, "runtime-info binding lost its exact source role")
            })?
        };
        let scope = FileScope {
            declarations: input.declarations,
            file: declaration.file(),
            substitution: None,
        };
        meta.register_reflection_source_headers(scope, types, location)
            .map_err(|error| input.graph.diagnostic(location, error.message))?;
        let mut metadata = scope.reflection_metadata(types);
        meta.record_specializations
            .append_reflection_metadata(&mut metadata, input.graph.symbols());
        meta.local_declarations
            .append_reflection_metadata(&mut metadata, input.graph.symbols());
        if let Some(context) = input.context {
            context.append_reflection_metadata(&mut metadata, input.graph.symbols());
        }
        let checkpoint = match meta
            .runtime_info_checkpoint(types, role.schema(), input.layout, location)
            .map_err(|error| input.graph.diagnostic(location, error.message))?
        {
            ReflectionReadiness::Ready(checkpoint) => checkpoint,
            ReflectionReadiness::Pending(dependencies) => {
                return Err(input.graph.diagnostic(
                    location,
                    format!("native Runtime_Info publication is waiting for {dependencies:?}"),
                ));
            }
        };
        let snapshot =
            match meta
                .runtime_info_snapshot(types, &checkpoint, &metadata, location)
                .map_err(|error| input.graph.diagnostic(location, error.message))?
            {
                ReflectionReadiness::Ready(snapshot) => snapshot,
                ReflectionReadiness::Pending(dependencies) => return Err(input.graph.diagnostic(
                    location,
                    format!(
                        "native Runtime_Info descriptor publication is waiting for {dependencies:?}"
                    ),
                )),
            };
        roles.push((role, snapshot));
    }
    Ok(PreparedRuntimeInfo { roles })
}

impl PreparedRuntimeInfo {
    pub(in crate::modules) fn attach(
        self,
        mut library: Library,
    ) -> Result<Library, LocatedDiagnostic> {
        for (role, snapshot) in self.roles {
            let location = role.source().location();
            let publication =
                role.publication(&library, snapshot)
                    .map_err(|error| LocatedDiagnostic {
                        location,
                        message: error.to_string(),
                    })?;
            library = library
                .with_native_runtime_info(publication)
                .map_err(|error| LocatedDiagnostic {
                    location,
                    message: error.to_string(),
                })?;
        }
        Ok(library)
    }
}
