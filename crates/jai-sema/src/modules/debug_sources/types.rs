//! Publish source identities from actual ready semantic nominal shapes.
use super::*;
use crate::local_declarations::LocalDeclarationRegistry;
use jai_ir::{FieldSource, TypeSource};
use jai_types::{FieldId, TypeId};

pub(in crate::modules) fn retain_types(
    sources: &mut DebugSources,
    graph: &ModuleGraph,
    nominals: &aggregates::Nominals<'_>,
    records: &aggregates::parameterized::RecordSpecializations,
    locals: &LocalDeclarationRegistry,
) -> Result<(), LocatedDiagnostic> {
    for (&ty, record) in &nominals.records {
        let declaration = graph
            .declaration(record.declaration)
            .expect("nominal source declaration");
        let location = declaration.location();
        retain_type(sources, graph, ty, Some(declaration.name()), location)?;
        for field in &record.fields {
            retain_field(
                sources,
                graph,
                field.id,
                field.name,
                SourceSpan {
                    source: location.source,
                    span: field.syntax.span,
                },
            )?;
        }
    }
    for (&declaration, &ty) in &nominals.declarations {
        let declaration = graph
            .declaration(declaration)
            .expect("nominal source declaration");
        let owns_name = matches!(
            &declaration.syntax().kind,
            FileDeclarationKind::Enum(_)
                | FileDeclarationKind::TypeAlias(syntax::TypeAliasDeclaration {
                    ty: syntax::TypeSyntax::Variant { .. },
                    ..
                })
        );
        if owns_name {
            retain_type(
                sources,
                graph,
                ty,
                Some(declaration.name()),
                declaration.location(),
            )?;
        }
    }
    for (ty, record) in records.records() {
        let Some(location) = records.source_location(ty) else {
            // Compiler-created shapes without an AST origin have no source type.
            continue;
        };
        retain_type(sources, graph, ty, record.shape.name, location)?;
        for field in &record.shape.fields {
            let Some(name) = field.name else {
                // An unnamed physical field has no source spelling to publish.
                continue;
            };
            retain_field(
                sources,
                graph,
                field.id,
                name,
                SourceSpan {
                    source: location.source,
                    span: field.syntax.span(),
                },
            )?;
        }
    }
    for (ty, location, name) in locals.debug_type_origins() {
        retain_type(sources, graph, ty, name, location)?;
        if let Some(fields) = locals.debug_record_fields(ty) {
            for field in fields {
                let Some(name) = field.name else {
                    continue;
                };
                retain_field(
                    sources,
                    graph,
                    field.id,
                    name,
                    SourceSpan {
                        source: location.source,
                        span: field.syntax.span(),
                    },
                )?;
            }
        }
    }
    Ok(())
}

fn retain_type(
    sources: &mut DebugSources,
    graph: &ModuleGraph,
    ty: TypeId,
    name: Option<Symbol>,
    location: SourceSpan,
) -> Result<(), LocatedDiagnostic> {
    let record = graph
        .sources()
        .get(location.source)
        .expect("nominal retains its exact source identity");
    let debug_location = sources
        .source_location(record, location)
        .map_err(|error| graph.diagnostic(location, error.to_string()))?;
    sources.insert_type_source(
        ty,
        TypeSource {
            name: name.map(|name| graph.symbols().name(name).to_owned()),
            location: debug_location,
        },
    );
    Ok(())
}

fn retain_field(
    sources: &mut DebugSources,
    graph: &ModuleGraph,
    id: FieldId,
    name: Symbol,
    location: SourceSpan,
) -> Result<(), LocatedDiagnostic> {
    let record = graph
        .sources()
        .get(location.source)
        .expect("field retains its exact source identity");
    let debug_location = sources
        .source_location(record, location)
        .map_err(|error| graph.diagnostic(location, error.to_string()))?;
    sources.insert_field_source(
        id,
        FieldSource {
            name: graph.symbols().name(name).to_owned(),
            location: debug_location,
        },
    );
    Ok(())
}
