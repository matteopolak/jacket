//! Nominal module restrictions reuse the procedure matcher's canonical ancestry facts.
use super::*;
use crate::restriction_facts::{RestrictionField, nominal_ancestor};
use jai_modules::{DeferredParameter, ModuleType};

pub(super) struct NominalTypes<'a> {
    pub actual: &'a ModuleType,
    pub required: &'a ModuleType,
}

pub(super) fn check(
    graph: &ModuleGraph,
    request: &DeferredParameter,
    input: NominalTypes<'_>,
    nominals: &Nominals<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    constants: &mut Constants<'_>,
) -> Result<(), LocatedDiagnostic> {
    let mut evaluate =
        |file, expression: &syntax::Expression| constants.evaluate_lazy(file, expression);
    let mut resolve = |value| {
        nominals.resolve_module_type_with_specializations(
            graph,
            aggregates::types::ModuleTypeRequest {
                file: request.file,
                value,
                span: request.location.span,
            },
            types,
            &mut meta.record_specializations,
            &mut evaluate,
        )
    };
    let actual = resolve(input.actual)?;
    let required = resolve(input.required)?;
    let proof = nominal_ancestor(types, actual, required, request.location.span, |ty| {
        let records = &meta.record_specializations;
        let origin = records.record(ty).and_then(|record| record.origin.map(|origin| origin.0))
            .or_else(|| nominals.records.get(&ty).map(|record| record.declaration));
        if origin.and_then(|id| graph.declaration(id)).is_some_and(|declaration|
            matches!(&declaration.syntax().kind, FileDeclarationKind::Record(record) if record.modify.is_some())) {
            return Err(Diagnostic::new(request.location.span,
                "modified module nominal ancestry requires committed modifier replay outcomes"));
        }
        if let Some(record) = records.record(ty) {
            Ok(record.shape.fields.iter().map(|field| RestrictionField {
                name: field.name, id: field.id, ty: field.ty, using: field.syntax.using(),
            }).collect())
        } else if let Some(record) = nominals.records.get(&ty) {
            Ok(record.fields.iter().map(|field| RestrictionField {
                name: Some(field.name), id: field.id, ty: field.ty, using: field.syntax.using,
            }).collect())
        } else {
            Err(Diagnostic::new(request.location.span,
                "module nominal restriction requires a ready source record schema"))
        }
    }).map_err(|error| LocatedDiagnostic {
        location: SourceSpan {source: error.source.unwrap_or(request.location.source), span: error.span},
        message: error.message,
    })?;
    if !proof {
        return Err(LocatedDiagnostic {
            location: request.location,
            message: "module type does not satisfy the nominal restriction".into(),
        });
    }
    Ok(())
}
