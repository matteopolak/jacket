//! Bootstrap fallback never unifies independent imported nominal declarations.
use jai_modules::{Binding, GraphOptions, ModuleGraph, PreludeSource, SourceOverlay};
use jai_syntax::{FileDeclarationKind, NamePath};
use std::path::Path;

fn sources(main: &str) -> SourceOverlay {
    let mut sources = SourceOverlay::new();
    for (path, text) in [
        ("/nominal-imports/main.jai", main),
        (
            "/nominal-imports/fallback/module.jai",
            "Type_Info::struct{fallback:bool;}",
        ),
        (
            "/nominal-imports/native/Basic.jai",
            "Type_Info_Tag::enum{INTEGER;}\nType_Info::struct{type:Type_Info_Tag;runtime_size:s64;}",
        ),
        (
            "/nominal-imports/native/Compiler.jai",
            "\nType_Info::struct{type:string;runtime_size:int;}",
        ),
    ] {
        sources
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    sources
}

fn load(
    sources: &SourceOverlay,
    prelude: PreludeSource,
) -> Result<ModuleGraph, jai_modules::GraphError> {
    ModuleGraph::load_with_bootstrap(
        Path::new("/nominal-imports/main.jai"),
        GraphOptions {
            import_dirs: vec![
                "/nominal-imports/native".into(),
                "/nominal-imports/fallback".into(),
            ],
        },
        prelude,
        sources,
        None,
    )
}

fn lookup(graph: &ModuleGraph, names: &[&str]) -> Binding {
    let names = names
        .iter()
        .map(|name| graph.symbols().find(name).unwrap())
        .collect::<Vec<_>>();
    graph
        .lookup(
            graph.module(graph.root()).unwrap().entry(),
            &NamePath {
                root: names[0],
                members: names[1..].to_vec(),
            },
        )
        .unwrap()
}

#[test]
fn fallback_and_qualified_imports_keep_three_original_nominal_identities() {
    let graph = load(
        &sources("Basic::#import \"Basic\";Compiler::#import \"Compiler\";"),
        PreludeSource::Search,
    )
    .unwrap();
    let bindings = [
        lookup(&graph, &["Type_Info"]),
        lookup(&graph, &["Basic", "Type_Info"]),
        lookup(&graph, &["Compiler", "Type_Info"]),
    ];
    let ids = bindings.map(|binding| match binding {
        Binding::Declaration(id) => id,
        _ => panic!("expected original nominal declaration"),
    });
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[2]);
    assert_ne!(ids[1], ids[2]);
    for id in ids {
        assert!(matches!(
            graph.declaration(id).unwrap().syntax().kind,
            FileDeclarationKind::Record(_)
        ));
    }
    let fallback = graph.prelude().unwrap();
    let file = graph.module(fallback).unwrap().entry();
    assert_eq!(
        graph
            .sources()
            .get(graph.file(file).unwrap().source())
            .unwrap()
            .path(),
        Path::new("/nominal-imports/fallback/module.jai")
    );
    assert_eq!(
        graph
            .file(graph.declaration(ids[0]).unwrap().file())
            .unwrap()
            .module(),
        fallback
    );
}

#[test]
fn imported_nominal_collision_is_independent_of_bootstrap_and_lists_both_definitions() {
    let sources = sources("#import \"Basic\";\n#import \"Compiler\";");
    for prelude in [PreludeSource::Disabled, PreludeSource::Search] {
        let diagnostic = load(&sources, prelude).unwrap_err().to_string();
        assert!(diagnostic.contains("/nominal-imports/main.jai:2:1: error: conflicting declaration or import 'Type_Info'"), "{diagnostic}");
        assert!(
            diagnostic
                .contains("/nominal-imports/native/Basic.jai:2:1: note: existing declaration"),
            "{diagnostic}"
        );
        assert!(
            diagnostic
                .contains("/nominal-imports/native/Compiler.jai:2:1: note: incoming declaration"),
            "{diagnostic}"
        );
        assert!(
            !diagnostic.contains("module.jai"),
            "fallback was incorrectly linked into the collision: {diagnostic}"
        );
    }
}

#[test]
fn imported_nominal_shadows_fallback_without_replacing_it() {
    let graph = load(&sources("#import \"Basic\";"), PreludeSource::Search).unwrap();
    let Binding::Declaration(imported) = lookup(&graph, &["Type_Info"]) else {
        panic!("expected imported record");
    };
    let prelude = graph.prelude().unwrap();
    let fallback =
        graph.module(prelude).unwrap().exports()[&graph.symbols().find("Type_Info").unwrap()];
    assert_ne!(fallback, Binding::Declaration(imported));
    assert_ne!(
        graph
            .file(graph.declaration(imported).unwrap().file())
            .unwrap()
            .module(),
        prelude
    );
}
