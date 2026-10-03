//! Bind native exports and program-entry aliases through source declaration identities.
use super::*;
use jai_ir::{ExportTarget, NativeSymbol, ProgramExport};

pub(super) fn bind(
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    types: &TypeRegistry,
) -> Result<Vec<ProgramExport>, LocatedDiagnostic> {
    let mut exports = Vec::new();
    let mut symbols = HashMap::new();
    for declaration in graph.declarations() {
        let Some(annotation) = &declaration.syntax().program_export else {
            continue;
        };
        let export_location = jai_source::SourceSpan {
            source: declaration.location().source,
            span: annotation.span,
        };
        let symbol = annotation
            .symbol
            .as_deref()
            .unwrap_or_else(|| graph.symbols().name(declaration.name()));
        let symbol = NativeSymbol::new(symbol)
            .map_err(|error| graph.diagnostic(export_location, error.to_string()))?;
        let target = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(_) => {
                let signature =
                    declarations
                        .signatures
                        .get(&declaration.id())
                        .ok_or_else(|| {
                            graph.diagnostic(
                                declaration.location(),
                                "program export requires a concrete procedure signature",
                            )
                        })?;
                ExportTarget::Procedure(signature.id)
            }
            FileDeclarationKind::Global(_) => {
                let Some(Binding::Storage(storage)) = declarations.values.get(&declaration.id())
                else {
                    return Err(graph.diagnostic(
                        declaration.location(),
                        "program export global has no allocated runtime storage",
                    ));
                };
                let PlaceKind::Global(id) = storage.place().kind() else {
                    return Err(graph.diagnostic(
                        declaration.location(),
                        "program export must target file global storage",
                    ));
                };
                ExportTarget::Global(id)
            }
            _ => {
                return Err(graph.diagnostic(
                    export_location,
                    "program export requires a procedure or global definition",
                ));
            }
        };
        if symbol.as_str() == "main" {
            let ExportTarget::Procedure(id) = target else {
                return Err(graph.diagnostic(
                    export_location,
                    "exported main must be a #c_call procedure definition",
                ));
            };
            let signature = declarations
                .signatures
                .values()
                .find(|signature| signature.id == id)
                .unwrap();
            let procedure = types
                .procedure_definition(signature.ty)
                .map_err(|error| graph.diagnostic(declaration.location(), error.to_string()))?;
            jai_ir::validate_main_signature(procedure, types)
                .map_err(|error| graph.diagnostic(export_location, error.to_string()))?;
        }
        if let Some(previous) = symbols.insert(symbol.clone(), declaration.id()) {
            let previous = graph.declaration(previous).unwrap();
            return Err(graph.diagnostic(
                export_location,
                format!(
                    "conflicting program export symbol {:?}; first declared in {}",
                    symbol.as_str(),
                    graph
                        .sources()
                        .get(previous.location().source)
                        .unwrap()
                        .path()
                        .display()
                ),
            ));
        }
        exports.push(ProgramExport {
            declaration: declaration.id(),
            target,
            symbol,
        });
    }
    Ok(exports)
}

pub(super) fn application_signature(
    graph: &ModuleGraph,
    declarations: &ScopedDeclarations<'_>,
    span: Span,
) -> Result<Signature, Diagnostic> {
    let file = graph.module(graph.root()).unwrap().entry();
    let name = graph.symbols().find("main").ok_or_else(|| {
        Diagnostic::new(span, "#entry_point requires an application main definition")
    })?;
    let jai_modules::Binding::Declaration(id) = graph.lookup(file, &path(name)).map_err(|_| {
        Diagnostic::new(
            span,
            "#entry_point requires a single application main definition",
        )
    })?
    else {
        return Err(Diagnostic::new(
            span,
            "#entry_point requires a single application main definition",
        ));
    };
    let declaration = graph.declaration(id).unwrap();
    if graph.file(declaration.file()).unwrap().module() != graph.root()
        || !matches!(declaration.syntax().kind, FileDeclarationKind::Procedure(_))
    {
        return Err(Diagnostic::new(
            span,
            "#entry_point must bind the application's own main definition",
        ));
    }
    let signature = declarations.signatures.get(&id).ok_or_else(|| {
        Diagnostic::new(span, "#entry_point application main signature is not ready")
    })?;
    Ok(signature.clone())
}

pub(super) fn bind_entry_aliases(
    graph: &ModuleGraph,
    declarations: &mut ScopedDeclarations<'_>,
) -> Result<(), LocatedDiagnostic> {
    for declaration in graph.declarations() {
        let FileDeclarationKind::ProcedurePrototype(source) = &declaration.syntax().kind else {
            continue;
        };
        if !matches!(source.binding, syntax::PrototypeBinding::EntryPoint) {
            continue;
        }
        let main = application_signature(graph, declarations, source.span)
            .map_err(|error| located(graph, declaration.file(), error))?;
        let requested = declarations
            .signatures
            .get(&declaration.id())
            .ok_or_else(|| {
                graph.diagnostic(
                    declaration.location(),
                    "#entry_point aliases cannot be polymorphic",
                )
            })?;
        if requested.ty != main.ty {
            return Err(graph.diagnostic(
                declaration.location(),
                "#entry_point alias signature must match the actual application main signature",
            ));
        }
        declarations.signatures.insert(declaration.id(), main);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};
    use std::path::Path;
    fn graph(source: &str, dependency: &str) -> ModuleGraph {
        let mut sources = SourceOverlay::new();
        sources
            .insert(
                Path::new("/own-export/main.jai"),
                source.as_bytes().to_vec(),
            )
            .unwrap();
        sources
            .insert(
                Path::new("/own-export/Api.jai"),
                dependency.as_bytes().to_vec(),
            )
            .unwrap();
        ModuleGraph::load_with_provider(
            Path::new("/own-export/main.jai"),
            GraphOptions {
                import_dirs: vec!["/own-export".into()],
            },
            &sources,
        )
        .unwrap()
    }
    #[test]
    fn exact_export_identities_survive_imports_independently_of_module_visibility() {
        let graph = graph(
            "Api :: #import \"Api\"; main :: () -> int { return Api.answer(); }",
            "#scope_module #program_export \"native_count\" count:s32=7; #scope_export #program_export \"native_answer\" answer :: () -> s32 #c_call {return count;}",
        );
        let library = crate::resolve_library(&graph).unwrap();
        assert_eq!(library.program_exports().len(), 2);
        for export in library.program_exports() {
            let declaration = graph.declaration(export.declaration).unwrap();
            assert_ne!(
                graph.file(declaration.file()).unwrap().module(),
                graph.root()
            );
            match export.target {
                ExportTarget::Procedure(id) => {
                    assert_eq!(library.procedure(declaration.id()).unwrap().id, id)
                }
                ExportTarget::Global(id) => assert_eq!(library.globals()[id.index()].id(), id),
            }
        }
    }
    #[test]
    fn native_trampoline_does_not_replace_language_vm_entry() {
        let graph = graph(
            "#program_export \"main\" system_entry :: (argc:s32,argv:**u8)->s32 #c_call { return argc; } main :: () -> int { return 42; }",
            "",
        );
        let program = crate::resolve_graph(&graph).unwrap();
        let jai_ir::NativeEntryPoint::ExportedProcedure(native) = program.native_entry() else {
            panic!()
        };
        let EntryPoint::Int(language) = program.entry() else {
            panic!()
        };
        assert_ne!(native, language);
        assert!(
            matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome, jai_vm::Outcome::Complete(ref values) if values == &[jai_vm::Value::Int(IntegerValue::checked(IntegerType::S64, 42).unwrap())])
        );
    }
    #[test]
    fn duplicate_reserved_and_invalid_main_exports_have_source_diagnostics() {
        for (source, expected) in [
            (
                "#program_export \"same\" first :: () {} #program_export \"same\" second :: () {}",
                "conflicting program export symbol",
            ),
            (
                "#program_export \"jai.p0\" first :: () {}",
                "reserved compiler namespace",
            ),
            ("#program_export \"main\" first :: () {}", "#c_call"),
            ("#program_export \"main\" value:s32=0;", "#c_call procedure"),
        ] {
            let graph = graph(source, "");
            let error = crate::resolve_library(&graph).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
            assert_eq!(
                error.location.source,
                graph
                    .module(graph.root())
                    .and_then(|module| graph.file(module.entry()))
                    .unwrap()
                    .source()
            );
            let expected_annotation = graph
                .declarations()
                .iter()
                .filter_map(|declaration| declaration.syntax().program_export.as_ref())
                .next_back()
                .unwrap();
            assert_eq!(error.location.span, expected_annotation.span);
        }
    }

    #[test]
    fn file_and_local_entry_aliases_bind_the_actual_main_without_prototype_bodies() {
        for source in [
            "entry :: () #entry_point; main :: () {} caller :: () { entry(); }",
            "main :: () {} caller :: () { entry :: () #entry_point; entry(); }",
        ] {
            let graph = graph(source, "");
            let library = crate::resolve_library(&graph).unwrap();
            assert!(library.prototypes().is_empty());
            assert_eq!(library.procedures().len(), 2);
            let main = graph
                .declarations()
                .iter()
                .find(|declaration| graph.symbols().name(declaration.name()) == "main")
                .unwrap();
            let canonical = library.procedure(main.id()).unwrap().id;
            let caller = graph
                .declarations()
                .iter()
                .find(|declaration| graph.symbols().name(declaration.name()) == "caller")
                .unwrap();
            let body = &library.procedure(caller.id()).unwrap().body;
            assert!(
                matches!(&body.statements[0], Statement::CallVoid(call) if call.procedure == canonical)
            );
        }
    }

    #[test]
    fn incompatible_entry_aliases_fail_before_allocating_fake_bodies() {
        for source in [
            "entry :: () #entry_point; main :: () -> int {return 42;}",
            "main :: () -> int { return 42; } caller :: () { entry :: () #entry_point; entry(); }",
        ] {
            let graph = graph(source, "");
            assert!(
                crate::resolve_library(&graph)
                    .unwrap_err()
                    .to_string()
                    .contains("#entry_point")
            );
        }
    }
}
