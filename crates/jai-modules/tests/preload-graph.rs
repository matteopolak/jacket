use jai_modules::{
    Binding, GraphError, GraphOptions, LookupError, ModuleGraph, PreludeSource, SourceOverlay,
};
use jai_syntax::NamePath;
use std::path::Path;

fn graph(files: &[(&str, &str)]) -> ModuleGraph {
    selected_graph(files, PreludeSource::Search).unwrap()
}

fn selected_graph(
    files: &[(&str, &str)],
    prelude: PreludeSource,
) -> Result<ModuleGraph, GraphError> {
    let mut provider = SourceOverlay::new();
    for (path, source) in files {
        provider
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_bootstrap(
        Path::new("/jai-prelude/main.jai"),
        GraphOptions {
            import_dirs: vec!["/jai-prelude/modules".into()],
        },
        prelude,
        &provider,
        None,
    )
}

fn lookup(
    graph: &ModuleGraph,
    file: jai_modules::FileInstanceId,
    names: &[&str],
) -> Result<Binding, LookupError> {
    graph.lookup(
        file,
        &NamePath {
            root: graph.symbols().find(names[0]).unwrap(),
            members: names[1..]
                .iter()
                .map(|name| graph.symbols().find(name).unwrap())
                .collect(),
        },
    )
}

#[test]
fn one_preload_identity_is_shared_by_modules_and_explicit_imports() {
    let graph = graph(&[
        (
            "/jai-prelude/main.jai",
            "A :: #import \"A\"; B :: #import \"B\"; P :: #import \"Preload\"; main :: () {}",
        ),
        (
            "/jai-prelude/modules/module.jai",
            "Shared :: struct { value: int; } shared_value :: 42;",
        ),
        ("/jai-prelude/modules/A.jai", "value :: shared_value;"),
        ("/jai-prelude/modules/B.jai", "value :: shared_value;"),
    ]);
    let root = graph.module(graph.root()).unwrap().entry();
    let prelude = graph.prelude().unwrap();
    assert_eq!(lookup(&graph, root, &["P"]), Ok(Binding::Module(prelude)));
    let canonical = lookup(&graph, root, &["Shared"]).unwrap();
    for module in graph.modules() {
        assert_eq!(lookup(&graph, module.entry(), &["Shared"]), Ok(canonical));
    }
    assert_eq!(graph.modules().len(), 4);
    assert_eq!(graph.sources().records().len(), 4);
    assert_eq!(
        graph
            .declarations()
            .iter()
            .filter(|declaration| graph.symbols().name(declaration.name()) == "Shared")
            .count(),
        1
    );
}

#[test]
fn file_and_module_names_shadow_the_shared_fallback() {
    let graph = graph(&[
        (
            "/jai-prelude/main.jai",
            "#load \"other.jai\"; Shared :: 2; #scope_file; Shared :: 3; main :: () {}",
        ),
        ("/jai-prelude/other.jai", "other :: Shared;"),
        ("/jai-prelude/modules/module.jai", "Shared :: 1;"),
    ]);
    let root = graph.module(graph.root()).unwrap();
    let private = lookup(&graph, root.entry(), &["Shared"]).unwrap();
    let module = lookup(&graph, root.files()[1], &["Shared"]).unwrap();
    let prelude = lookup(
        &graph,
        graph.module(graph.prelude().unwrap()).unwrap().entry(),
        &["Shared"],
    )
    .unwrap();
    assert_ne!(private, module);
    assert_ne!(module, prelude);
    assert_eq!(
        root.exports()
            .get(&graph.symbols().find("Shared").unwrap())
            .copied(),
        Some(module)
    );
}

#[test]
fn prelude_fallback_never_becomes_a_namespace_export_or_application_leak() {
    let graph = graph(&[
        (
            "/jai-prelude/main.jai",
            "A :: #import \"A\"; application_only :: 99; main :: () {}",
        ),
        (
            "/jai-prelude/modules/module.jai",
            "Shared :: 42; #scope_module; preload_private :: 7; #scope_file; preload_file :: 8;",
        ),
        ("/jai-prelude/modules/A.jai", "local :: Shared;"),
    ]);
    let root = graph.module(graph.root()).unwrap().entry();
    let Binding::Module(module) = lookup(&graph, root, &["A"]).unwrap() else {
        panic!("module expected");
    };
    let file = graph.module(module).unwrap().entry();
    assert!(lookup(&graph, file, &["Shared"]).is_ok());
    assert!(matches!(
        lookup(&graph, root, &["A", "Shared"]),
        Err(LookupError::UnknownMember { .. })
    ));
    for name in ["application_only", "preload_private", "preload_file"] {
        assert!(matches!(
            lookup(&graph, file, &[name]),
            Err(LookupError::UnknownName(_))
        ));
    }
    assert!(
        !graph
            .module(module)
            .unwrap()
            .exports()
            .contains_key(&graph.symbols().find("Shared").unwrap())
    );
}

#[test]
fn absent_required_preload_is_an_error_and_disabled_keeps_low_level_loading() {
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            Path::new("/jai-prelude/main.jai"),
            b"main :: () {}".to_vec(),
        )
        .unwrap();
    let options = GraphOptions {
        import_dirs: vec!["/jai-prelude/missing".into()],
    };
    assert!(matches!(
        ModuleGraph::load_with_bootstrap(
            Path::new("/jai-prelude/main.jai"),
            options.clone(),
            PreludeSource::Search,
            &provider,
            None
        ),
        Err(GraphError::Prelude(_))
    ));
    let graph = ModuleGraph::load_with_bootstrap(
        Path::new("/jai-prelude/main.jai"),
        options,
        PreludeSource::Disabled,
        &provider,
        None,
    )
    .unwrap();
    assert_eq!(graph.prelude(), None);
    assert_eq!(graph.modules().len(), 1);
}

#[test]
fn checking_preload_itself_reserves_its_source_once() {
    let mut provider = SourceOverlay::new();
    let path = Path::new("/jai-prelude/modules/module.jai");
    provider
        .insert(path, b"Shared :: struct { value: int; }".to_vec())
        .unwrap();
    let graph = ModuleGraph::load_with_bootstrap(
        path,
        GraphOptions {
            import_dirs: vec!["/jai-prelude/modules".into()],
        },
        PreludeSource::Search,
        &provider,
        None,
    )
    .unwrap();
    assert_eq!(graph.prelude(), Some(graph.root()));
    assert_eq!(graph.modules().len(), 1);
    assert_eq!(graph.sources().records().len(), 1);
    assert_eq!(graph.declarations().len(), 1);
}

#[test]
fn explicit_bootstrap_selection_owns_named_preload_imports() {
    let graph = selected_graph(
        &[
            (
                "/jai-prelude/main.jai",
                "P :: #import \"Preload\"; A :: #import \"A\"; main :: () {}",
            ),
            (
                "/jai-prelude/bootstrap.jai",
                "Shared :: struct { value: int; }",
            ),
            (
                "/jai-prelude/modules/module.jai",
                "this source must not be parsed",
            ),
            (
                "/jai-prelude/modules/A.jai",
                "P :: #import \"Preload\"; item: P.Shared;",
            ),
        ],
        PreludeSource::File("/jai-prelude/bootstrap.jai".into()),
    )
    .unwrap();
    let root = graph.module(graph.root()).unwrap().entry();
    assert_eq!(
        lookup(&graph, root, &["P"]),
        Ok(Binding::Module(graph.prelude().unwrap()))
    );
    assert_eq!(
        lookup(&graph, root, &["Shared"]),
        lookup(&graph, root, &["P", "Shared"])
    );
    let Binding::Module(imported) = lookup(&graph, root, &["A"]).unwrap() else {
        panic!("module expected");
    };
    assert_eq!(
        lookup(&graph, graph.module(imported).unwrap().entry(), &["P"]),
        Ok(Binding::Module(graph.prelude().unwrap()))
    );
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(graph.sources().records().len(), 3);
}

#[test]
fn empty_named_import_arguments_reuse_the_shared_prelude() {
    let graph = graph(&[
        (
            "/jai-prelude/main.jai",
            "P :: #import \"Preload\"(); Q :: #import \"Preload\"()(); main :: () {}",
        ),
        (
            "/jai-prelude/modules/module.jai",
            "Shared :: struct { value: int; }",
        ),
    ]);
    let root = graph.module(graph.root()).unwrap().entry();
    assert_eq!(lookup(&graph, root, &["P"]), lookup(&graph, root, &["Q"]));
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.declarations().len(), 2);
}

#[test]
fn nonempty_named_import_arguments_cannot_create_another_prelude() {
    for arguments in ["(value=1)", "()(value=1)"] {
        let source = format!("P :: #import \"Preload\"{arguments}; main :: () {{}}");
        let error = selected_graph(
            &[
                ("/jai-prelude/main.jai", &source),
                ("/jai-prelude/modules/module.jai", "Shared :: struct {};"),
            ],
            PreludeSource::Search,
        )
        .unwrap_err();
        let GraphError::Located {
            diagnostic,
            rendered,
        } = error
        else {
            panic!("located import diagnostic expected");
        };
        assert!(rendered.contains("main.jai:1:"));
        assert!(diagnostic.message.contains("cannot be specialized"));
    }
}

#[test]
fn disabled_bootstrap_keeps_named_preload_an_ordinary_import() {
    let graph = selected_graph(
        &[
            (
                "/jai-prelude/main.jai",
                "P :: #import \"Preload\"; main :: () {}",
            ),
            ("/jai-prelude/modules/module.jai", "Shared :: struct {};"),
        ],
        PreludeSource::Disabled,
    )
    .unwrap();
    assert_eq!(graph.prelude(), None);
    let root = graph.module(graph.root()).unwrap().entry();
    assert!(lookup(&graph, root, &["P", "Shared"]).is_ok());
    assert!(matches!(
        lookup(&graph, root, &["Shared"]),
        Err(LookupError::UnknownName(_))
    ));
}

#[test]
fn explicit_file_imports_keep_their_actual_source_identity() {
    let graph = selected_graph(
        &[
            (
                "/jai-prelude/main.jai",
                "P :: #import \"Preload\"; Q :: #import,file \"other.jai\"; main :: () {}",
            ),
            ("/jai-prelude/bootstrap.jai", "Shared :: struct {};"),
            ("/jai-prelude/other.jai", "Shared :: struct {};"),
        ],
        PreludeSource::File("/jai-prelude/bootstrap.jai".into()),
    )
    .unwrap();
    let root = graph.module(graph.root()).unwrap().entry();
    assert_eq!(
        lookup(&graph, root, &["P", "Shared"]),
        lookup(&graph, root, &["Shared"])
    );
    assert_ne!(
        lookup(&graph, root, &["P", "Shared"]),
        lookup(&graph, root, &["Q", "Shared"])
    );
}
