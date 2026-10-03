use jai_modules::{Binding, GraphOptions, ModuleGraph, PreludeSource, SourceOverlay};
use jai_syntax::{
    CodeBody, ContextFieldDeclaration, ExpressionKind, FieldConversion, FileDeclarationKind,
    NamePath, ResultUsage, StatementKind,
};
use std::path::Path;

#[test]
fn complete_compiler_prelude_builds_a_source_graph_with_its_public_protocol() {
    let source = jai_modules::compiler_prelude_source();
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            Path::new("/jai-full-preload/main.jai"),
            b"main :: () {}".to_vec(),
        )
        .unwrap();
    provider
        .insert(
            Path::new("/jai-full-preload/modules/module.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    let graph = ModuleGraph::load_with_bootstrap(
        Path::new("/jai-full-preload/main.jai"),
        GraphOptions {
            import_dirs: vec!["/jai-full-preload/modules".into()],
        },
        PreludeSource::Search,
        &provider,
        None,
    )
    .unwrap();
    let module = graph.module(graph.prelude().unwrap()).unwrap();
    let file = module.entry();
    assert_eq!(
        graph
            .sources()
            .get(graph.file(file).unwrap().source())
            .unwrap()
            .text(),
        source
    );
    let find = |name| {
        let Binding::Declaration(declaration) = graph
            .lookup(
                file,
                &NamePath {
                    root: graph.symbols().find(name).unwrap(),
                    members: vec![],
                },
            )
            .unwrap()
        else {
            panic!("source declaration expected");
        };
        graph.declaration(declaration).unwrap()
    };
    let FileDeclarationKind::Procedure(workspace) = &find("get_current_workspace").syntax().kind
    else {
        panic!("source compiler body expected");
    };
    assert!(workspace.compiler.is_some());
    assert!(!workspace.body.is_empty());
    let FileDeclarationKind::ProcedurePrototype(memcmp) = &find("memcmp").syntax().kind else {
        panic!("intrinsic prototype expected");
    };
    assert_eq!(memcmp.results[0].usage, ResultUsage::Required);
    let FileDeclarationKind::Constant(context) = &find("FIRST_ADD_CONTEXT").syntax().kind else {
        panic!("quoted constant expected");
    };
    let ExpressionKind::Code(CodeBody::Statement(statement)) = &context.initializer.kind else {
        panic!("quoted source statement expected");
    };
    let StatementKind::ContextField(ContextFieldDeclaration::Field(field)) = &statement.kind else {
        panic!("quoted context field expected");
    };
    assert_eq!(graph.symbols().name(field.name), "base");
    assert!(field.using);
    assert_eq!(field.conversion, FieldConversion::Implicit);
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.sources().records().len(), 2);
}

#[test]
fn physical_prelude_fragments_share_one_module_and_match_composed_exports() {
    let entry = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(jai_modules::COMPILER_PRELUDE_ENTRY)
        .canonicalize()
        .unwrap();
    let mut provider = SourceOverlay::new();
    let main = Path::new("/jai-physical-preload/main.jai");
    provider.insert(main, b"main :: () {}".to_vec()).unwrap();
    let physical = ModuleGraph::load_with_bootstrap(
        main,
        GraphOptions::default(),
        PreludeSource::File(entry),
        &provider,
        None,
    )
    .unwrap();
    let composed_path = Path::new("/jai-physical-preload/module.jai");
    provider
        .insert(
            composed_path,
            jai_modules::compiler_prelude_source().as_bytes().to_vec(),
        )
        .unwrap();
    let composed = ModuleGraph::load_with_bootstrap(
        main,
        GraphOptions::default(),
        PreludeSource::File(composed_path.into()),
        &provider,
        None,
    )
    .unwrap();
    let exports = |graph: &ModuleGraph| {
        let mut names = graph
            .module(graph.prelude().unwrap())
            .unwrap()
            .exports()
            .keys()
            .map(|name| graph.symbols().name(*name).to_owned())
            .collect::<Vec<_>>();
        names.sort();
        names
    };
    assert_eq!(exports(&physical), exports(&composed));
    assert_eq!(physical.modules().len(), 2);
    assert_eq!(
        physical
            .module(physical.prelude().unwrap())
            .unwrap()
            .files()
            .len(),
        8
    );
    assert_eq!(physical.sources().records().len(), 9);
    assert_eq!(composed.sources().records().len(), 2);
}

#[test]
fn physical_bootstrap_and_library_preload_import_have_identical_declarations() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entry = repository
        .join(jai_modules::COMPILER_PRELUDE_ENTRY)
        .canonicalize()
        .unwrap();
    let library = repository.join("stdlib").canonicalize().unwrap();
    let mut provider = SourceOverlay::new();
    let main = Path::new("/jai-physical-preload/main.jai");
    provider
        .insert(main, b"P :: #import \"Preload\"; main :: () {}".to_vec())
        .unwrap();
    let graph = ModuleGraph::load_with_bootstrap(
        main,
        GraphOptions {
            import_dirs: vec![library],
        },
        PreludeSource::File(entry),
        &provider,
        None,
    )
    .unwrap();
    let root = graph.module(graph.root()).unwrap().entry();
    let lookup = |names: &[&str]| {
        graph
            .lookup(
                root,
                &NamePath {
                    root: graph.symbols().find(names[0]).unwrap(),
                    members: names[1..]
                        .iter()
                        .map(|name| graph.symbols().find(name).unwrap())
                        .collect(),
                },
            )
            .unwrap()
    };
    assert_eq!(lookup(&["P"]), Binding::Module(graph.prelude().unwrap()));
    for name in [
        "Type_Info",
        "Type_Info_Struct",
        "Allocator",
        "Operating_System_Tag",
    ] {
        assert_eq!(lookup(&[name]), lookup(&["P", name]));
        assert_eq!(
            graph
                .declarations()
                .iter()
                .filter(|declaration| graph.symbols().name(declaration.name()) == name)
                .count(),
            1
        );
    }
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.sources().records().len(), 9);
}
