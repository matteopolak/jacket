//! Preserve declaration provenance without changing executable IR expressions.
use super::*;
use jai_ir::{DebugSources, ProcedureSource};
mod types;
pub(super) use types::retain_types;

pub(super) fn retain_graph(
    sources: &mut DebugSources,
    graph: &ModuleGraph,
    signatures: &HashMap<DeclarationId, Signature>,
) -> Result<(), LocatedDiagnostic> {
    let root = graph
        .file(graph.module(graph.root()).expect("root module").entry())
        .expect("root source file");
    sources.set_primary_source(
        graph
            .sources()
            .get(root.source())
            .expect("root source provenance"),
    );
    for (declaration, signature) in signatures {
        let declaration = graph
            .declaration(*declaration)
            .expect("resolved declaration identity");
        // Callable aliases share a real procedure identity; their declaration
        // must never replace that procedure's source name or defining range.
        if !matches!(
            &declaration.syntax().kind,
            FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
        ) {
            continue;
        }
        if matches!(&declaration.syntax().kind, FileDeclarationKind::ProcedurePrototype(source) if matches!(source.binding, syntax::PrototypeBinding::EntryPoint))
        {
            continue;
        }
        let notes = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(procedure) => &procedure.notes,
            FileDeclarationKind::ProcedurePrototype(prototype) => &prototype.notes,
            _ => unreachable!("only genuine callable declarations publish their source"),
        };
        // The file wrapper owns the complete declaration range. Procedure.span
        // is its name token, used for header diagnostics and local captures.
        let location = declaration.location();
        let source = graph
            .sources()
            .get(location.source)
            .expect("declaration source provenance");
        crate::procedure_notes::retain(sources, signature.id, source, notes).map_err(|error| {
            graph.diagnostic(
                SourceSpan {
                    source: error.source.unwrap_or(location.source),
                    span: error.span,
                },
                error.message,
            )
        })?;
        let debug_location = sources
            .source_location(source, location)
            .map_err(|error| graph.diagnostic(location, error.to_string()))?;
        sources.retain_source(source);
        sources.insert(
            signature.id,
            ProcedureSource {
                name: graph.symbols().name(declaration.name()).to_owned(),
                location: debug_location,
            },
        );
    }
    Ok(())
}
