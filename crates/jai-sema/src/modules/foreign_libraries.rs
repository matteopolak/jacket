//! Resolve source identifiers to library declarations without accessing native bytes.
use super::*;
use jai_ir::{ForeignLibrary, ForeignLibraryId, ForeignLibraryKind, ForeignLibraryOptions};

pub(super) fn declaration(
    graph: &ModuleGraph,
    declaration: &jai_modules::Declaration,
) -> Result<ForeignLibrary, LocatedDiagnostic> {
    let FileDeclarationKind::Library(library) = &declaration.syntax().kind else {
        return Err(graph.diagnostic(
            declaration.location(),
            "declaration does not denote a foreign library",
        ));
    };
    let file = graph.file(declaration.file()).unwrap();
    let source = graph.sources().get(file.source()).unwrap();
    metadata(
        ForeignLibraryId::new(declaration.id()),
        library,
        source.path(),
    )
    .map_err(|error| graph.diagnostic(declaration.location(), error.to_string()))
}

pub(crate) fn metadata(
    id: ForeignLibraryId,
    library: &syntax::LibraryDeclaration,
    source_path: &std::path::Path,
) -> Result<ForeignLibrary, Diagnostic> {
    let kind = match library.kind {
        syntax::LibraryKind::System => ForeignLibraryKind::System {
            name: library.target.clone(),
        },
        syntax::LibraryKind::Local => ForeignLibraryKind::Local {
            path: source_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(&library.target),
        },
    };
    let result = ForeignLibrary {
        id,
        kind,
        options: ForeignLibraryOptions {
            link_always: library.options.link_always,
            no_dll: library.options.no_dll,
            no_static_library: library.options.no_static_library,
        },
    };
    result
        .validate()
        .map_err(|error| Diagnostic::new(library.span, error.to_string()))?;
    Ok(result)
}

pub(super) fn all(graph: &ModuleGraph) -> Result<Vec<ForeignLibrary>, LocatedDiagnostic> {
    graph
        .declarations()
        .iter()
        .filter(|declaration| matches!(declaration.syntax().kind, FileDeclarationKind::Library(_)))
        .map(|source| declaration(graph, source))
        .collect()
}

pub(super) fn source_provenance(
    graph: &ModuleGraph,
) -> Result<jai_ir::ForeignLibrarySources, LocatedDiagnostic> {
    let mut records = Vec::new();
    for declaration in graph
        .declarations()
        .iter()
        .filter(|declaration| matches!(declaration.syntax().kind, FileDeclarationKind::Library(_)))
    {
        let location = declaration.location();
        let source = graph
            .sources()
            .get(location.source)
            .expect("declaration owns source");
        let identity = jai_ir::SourceProcedureIdentity::new(source, location)
            .map_err(|error| graph.diagnostic(location, error.to_string()))?;
        let environment = graph
            .module_environment_origin(declaration.file())
            .map_err(|error| graph.diagnostic(location, error.to_string()))?;
        records.push((
            ForeignLibraryId::new(declaration.id()),
            identity,
            environment,
        ));
    }
    jai_ir::ForeignLibrarySources::from_source_records(records).map_err(|error| {
        let file = graph.module(graph.root()).expect("root module").entry();
        let location = graph.locate(file, Span::default()).expect("root source");
        graph.diagnostic(location, error.to_string())
    })
}

pub(super) fn origin(
    graph: &ModuleGraph,
    file: FileInstanceId,
    prototype: &syntax::ProcedurePrototype,
) -> Result<PrototypeOrigin, LocatedDiagnostic> {
    match &prototype.binding {
        syntax::PrototypeBinding::EntryPoint => Err(graph.diagnostic(graph.locate(file, prototype.span).unwrap(), "#entry_point alias must bind the checked application entry before prototype publication")),
        syntax::PrototypeBinding::Compiler(_) => Ok(PrototypeOrigin::Compiler),
        syntax::PrototypeBinding::Intrinsic { .. } => Err(graph.diagnostic(
            jai_source::SourceSpan {
                source: graph.file(file).unwrap().source(),
                span: prototype.span,
            },
            "#intrinsic prototype must be bound by the checked runtime intrinsic binder",
        )),
        syntax::PrototypeBinding::Foreign(binding) => {
            if prototype.convention == jai_types::CallingConvention::Jai {
                if binding.library.is_some() || binding.symbol.is_some() {
                    return Err(located(graph, file, Diagnostic::new(prototype.span, "Jai source contracts cannot carry native library or symbol bindings")));
                }
                return Ok(PrototypeOrigin::SourceContract { symbol: graph.symbols().name(prototype.name).to_owned() });
            }
            let library = binding
                .library
                .as_ref()
                .map(|path| {
                    let id = declaration_id(graph, file, path, prototype.span)
                        .map_err(|error| located(graph, file, error))?;
                    declaration(graph, graph.declaration(id).unwrap())
                })
                .transpose()?;
            Ok(PrototypeOrigin::Foreign {
                symbol: binding
                    .symbol
                    .clone()
                    .unwrap_or_else(|| graph.symbols().name(prototype.name).to_owned()),
                library,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};
    use std::path::Path;

    fn graph(source: &str, dependency: &str) -> ModuleGraph {
        let mut overlay = SourceOverlay::default();
        overlay
            .insert(
                Path::new("/own-fixture/main.jai"),
                source.as_bytes().to_vec(),
            )
            .unwrap();
        overlay
            .insert(
                Path::new("/own-fixture/Lib.jai"),
                dependency.as_bytes().to_vec(),
            )
            .unwrap();
        ModuleGraph::load_with_provider(
            Path::new("/own-fixture/main.jai"),
            GraphOptions {
                import_dirs: vec!["/own-fixture".into()],
            },
            &overlay,
        )
        .unwrap()
    }

    #[test]
    fn namespace_library_resolves_declaration_metadata_and_identity() {
        let graph = graph(
            "Lib :: #import \"Lib\"; external :: () -> int #foreign Lib.alias;",
            "alias :: #library,system \"libc\";",
        );
        let library = crate::resolve_library(&graph).unwrap();
        let PrototypeOrigin::Foreign {
            library: Some(resolved),
            ..
        } = &library.prototypes()[0].origin
        else {
            panic!()
        };
        assert_eq!(
            resolved.kind,
            ForeignLibraryKind::System {
                name: "libc".into()
            }
        );
        let source = graph
            .declaration(resolved.id.declaration().unwrap())
            .unwrap();
        assert_eq!(graph.symbols().name(source.name()), "alias");
    }

    #[test]
    fn historical_library_spelling_retains_checked_identity_in_vm_programs() {
        let graph = graph(
            "Lib::#import \"Lib\"; external::(value:s32)->s32 #foreign Lib.alias \"abs\"; main::()->int{return 42;}",
            "alias::#foreign_library,system \"libc\";",
        );
        let program = crate::resolve_graph(&graph).unwrap();
        let dependency = &program.library().foreign_libraries()[0];
        assert!(matches!(&dependency.kind, ForeignLibraryKind::System { name } if name == "libc"));
        assert!(
            matches!(&program.library().prototypes()[0].origin, PrototypeOrigin::Foreign { library:Some(resolved),symbol } if resolved == dependency && symbol == "abs")
        );
        let execution = jai_vm::execute(&program, jai_vm::Limits::default());
        assert!(
            matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==42)),
            "{execution:?}"
        );
    }

    #[test]
    fn scoped_import_library_preserves_the_defining_graph_identity() {
        let graph = graph(
            "main :: () { Lib :: #import \"Lib\"; external :: () #foreign Lib.alias; external(); }",
            "alias :: #system_library \"libc\";",
        );
        let library = crate::resolve_library(&graph).unwrap();
        let PrototypeOrigin::Foreign {
            library: Some(resolved),
            ..
        } = &library.prototypes()[0].origin
        else {
            panic!()
        };
        let source = graph
            .declaration(resolved.id.declaration().unwrap())
            .unwrap();
        assert_eq!(graph.symbols().name(source.name()), "alias");
        assert!(library.foreign_libraries().contains(resolved));
        assert_ne!(graph.file(source.file()).unwrap().module(), graph.root());
    }

    #[test]
    fn unknown_nonlibrary_and_private_names_fail_before_linking() {
        for (source, dependency, expected) in [
            ("external :: () #foreign missing;", "", "unknown"),
            (
                "value :: 4; external :: () #foreign value;",
                "",
                "does not denote a foreign library",
            ),
            (
                "Lib :: #import \"Lib\"; external :: () #foreign Lib.alias;",
                "#scope_module alias :: #system_library \"libc\";",
                "private",
            ),
        ] {
            let graph = graph(source, dependency);
            let error = crate::resolve_library(&graph).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn local_metadata_resolves_relative_to_defining_source_without_reading_it() {
        let graph = graph(
            "local :: #library,no_dll \"native/own\"; external :: () #foreign local;",
            "",
        );
        let library = crate::resolve_library(&graph).unwrap();
        let PrototypeOrigin::Foreign {
            library: Some(resolved),
            ..
        } = &library.prototypes()[0].origin
        else {
            panic!()
        };
        assert_eq!(
            resolved.kind,
            ForeignLibraryKind::Local {
                path: "/own-fixture/native/own".into()
            }
        );
        assert!(resolved.options.no_dll);
    }

    #[test]
    fn active_link_always_declarations_survive_without_prototype_references() {
        let graph = graph(
            "",
        );
        let library = crate::resolve_library(&graph).unwrap();
        assert!(library.prototypes().is_empty());
        assert_eq!(library.foreign_libraries().len(), 1);
        assert!(library.foreign_libraries()[0].options.link_always);
    }

    #[test]
    fn system_paths_are_rejected_even_when_unreferenced() {
        assert!(
            crate::resolve_library(&graph)
                .unwrap_err()
                .to_string()
                .contains("plain nonempty basename")
        );
    }

    #[test]
    fn lexical_library_declarations_bind_local_foreign_prototypes() {
        let graph = graph(
            "main :: () -> int { if true { Crt :: #system_library \"libc\"; absolute :: (value:s32)->s32 #foreign Crt \"abs\"; return absolute(-42); } return 0; }",
            "",
        );
        let library = crate::resolve_library(&graph).unwrap();
        let dependency = &library.foreign_libraries()[0];
        assert!(matches!(dependency.id, ForeignLibraryId::Local { .. }));
        assert_eq!(
            dependency.kind,
            ForeignLibraryKind::System {
                name: "libc".into()
            }
        );
        assert!(
            matches!(&library.prototypes()[0].origin, PrototypeOrigin::Foreign { library: Some(binding), .. } if binding == dependency)
        );
    }

    #[test]
    fn lexical_nonlibrary_shadow_does_not_fall_back_to_file_library() {
        let graph = graph(
            "Crt :: #system_library \"libc\"; main :: () { Crt :: 4; external :: () #foreign Crt; }",
            "",
        );
        assert!(
            crate::resolve_library(&graph)
                .unwrap_err()
                .to_string()
                .contains("does not denote a foreign library")
        );
    }
}
