use jai_modules::{Binding, FileUsingDecision, GraphDiscovery, ModuleGraph, SourceOverlay};
use jai_syntax::{BinaryOp, FileDeclarationKind, NamePath, OperatorKind, UnaryOp};
use std::path::Path;

#[test]
fn exported_operator_groups_retain_origins_and_exclude_named_bindings() {
    let mut overlay = SourceOverlay::new();
    for (path, source) in [
        (
            "/operators/main.jai",
            "#import, file \"bridge.jai\"; main :: () {}",
        ),
        ("/operators/bridge.jai", "#import, file \"math.jai\";"),
        (
            "/operators/math.jai",
            "Box::struct {value:int;} operator +::(a:Box,b:Box)->Box{return a;} #scope_module; operator +::(a:Box,b:int)->Box{return a;}",
        ),
    ] {
        overlay
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/operators/main.jai"),
        Default::default(),
        &overlay,
    )
    .unwrap();
    let root = graph.module(graph.root()).unwrap().entry();
    let kind = OperatorKind::Binary(BinaryOp::Add);
    let exported = graph.operator_declarations(root, kind);
    assert_eq!(exported.len(), 1);
    let defining = graph.declaration(exported[0]).unwrap();
    assert_eq!(graph.operator_declarations(defining.file(), kind).len(), 2);
    assert_eq!(
        graph.exported_operator_declarations(graph.root(), kind),
        exported
    );
    let name = graph.symbols().find("operator +").unwrap();
    assert!(
        graph
            .lookup(
                defining.file(),
                &NamePath {
                    root: name,
                    members: vec![]
                }
            )
            .is_err()
    );
}

#[test]
fn loaded_file_private_operators_are_visible_only_in_their_own_file() {
    let mut overlay = SourceOverlay::new();
    overlay.insert(Path::new("/operators/main.jai"), b"#load \"hidden.jai\"; Box::struct {value:int;} operator +::(a:Box,b:Box)->Box{return a;} main::(){}".to_vec()).unwrap();
    overlay
        .insert(
            Path::new("/operators/hidden.jai"),
            b"#scope_file; operator +::(a:Box,b:int)->Box{return a;}".to_vec(),
        )
        .unwrap();
    let graph = ModuleGraph::load_with_provider(
        Path::new("/operators/main.jai"),
        Default::default(),
        &overlay,
    )
    .unwrap();
    let module = graph.module(graph.root()).unwrap();
    let kind = OperatorKind::Binary(BinaryOp::Add);
    assert_eq!(graph.operator_declarations(module.entry(), kind).len(), 1);
    let hidden = module
        .files()
        .iter()
        .copied()
        .find(|file| *file != module.entry())
        .unwrap();
    assert_eq!(graph.operator_declarations(hidden, kind).len(), 2);
}

#[test]
fn namespace_imports_do_not_publish_operators_without_using() {
    for prefix in ["library::", "using library::"] {
        let mut overlay = SourceOverlay::new();
        let source = format!("{prefix} #import,file \"math.jai\"; main::(){{}}");
        overlay
            .insert(Path::new("/operators/main.jai"), source.into_bytes())
            .unwrap();
        overlay
            .insert(
                Path::new("/operators/math.jai"),
                b"Box::struct{value:int;} operator +::(a:Box,b:Box)->Box{return a;}".to_vec(),
            )
            .unwrap();
        let graph = ModuleGraph::load_with_provider(
            Path::new("/operators/main.jai"),
            Default::default(),
            &overlay,
        )
        .unwrap();
        let file = graph.module(graph.root()).unwrap().entry();
        let operators = graph.operator_declarations(file, OperatorKind::Binary(BinaryOp::Add));
        assert_eq!(operators.len(), usize::from(prefix.starts_with("using")));
    }
}

#[test]
fn checked_using_publishes_only_selected_original_operator_ids() {
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new("/operators/main.jai"),
            b"Library::#import,file \"math.jai\"; using,only(.[\"+\"]) Library; main::(){}"
                .to_vec(),
        )
        .unwrap();
    overlay.insert(Path::new("/operators/math.jai"), b"Box::struct{value:int;} operator +::(a:Box,b:Box)->Box{return a;} operator *::(a:Box,b:Box)->Box{return a;}".to_vec()).unwrap();
    let mut discovery = GraphDiscovery::new(
        Path::new("/operators/main.jai"),
        Default::default(),
        &overlay,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let requests = discovery.pending_using_requests();
    assert_eq!(requests.len(), 1);
    let file = requests[0].file;
    let graph = discovery.graph();
    let namespace = graph.symbols().find("Library").unwrap();
    let Binding::Module(module) = graph
        .lookup(
            file,
            &NamePath {
                root: namespace,
                members: vec![],
            },
        )
        .unwrap()
    else {
        panic!("using target must retain its actual module identity");
    };
    let kind = OperatorKind::Binary(BinaryOp::Add);
    let selected = graph.exported_operator_declarations(module, kind);
    assert_eq!(selected.len(), 1);
    discovery
        .resolve_using(
            requests[0].id,
            FileUsingDecision {
                selected_operator_declarations: selected.clone(),
                source_module: Some(module),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().unwrap();
    assert_eq!(graph.operator_declarations(file, kind), selected);
    assert_eq!(
        graph.exported_operator_declarations(graph.root(), kind),
        selected
    );
    assert!(
        graph
            .operator_declarations(file, OperatorKind::Binary(BinaryOp::Multiply))
            .is_empty()
    );
}

#[test]
fn operator_aliases_reexport_original_unary_and_binary_declarations() {
    for visibility in ["", "#scope_module;"] {
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                Path::new("/operators/main.jai"),
                b"#import,file \"bridge.jai\"; main::(){}".to_vec(),
            )
            .unwrap();
        overlay
            .insert(
                Path::new("/operators/bridge.jai"),
                format!(
                    "Library::#import,file \"math.jai\"; {visibility} operator-::Library.operator-;"
                )
                .into_bytes(),
            )
            .unwrap();
        overlay.insert(Path::new("/operators/math.jai"), b"Box::struct{value:int;} operator -::(a:Box)->Box{return a;} operator -::(a:Box,b:Box)->Box{return a;} operator +::(a:Box,b:Box)->Box{return a;}".to_vec()).unwrap();
        let graph = ModuleGraph::load_with_provider(
            Path::new("/operators/main.jai"),
            Default::default(),
            &overlay,
        )
        .unwrap();
        let root = graph.module(graph.root()).unwrap().entry();
        let alias = graph
            .declarations()
            .iter()
            .find(|declaration| {
                matches!(
                    declaration.syntax().kind,
                    FileDeclarationKind::OperatorAlias(_)
                )
            })
            .unwrap();
        for kind in [
            OperatorKind::Unary(UnaryOp::Negate),
            OperatorKind::Binary(BinaryOp::Subtract),
        ] {
            let inside = graph.operator_declarations(alias.file(), kind);
            assert_eq!(inside.len(), 1);
            assert_ne!(inside[0], alias.id());
            assert!(matches!(
                graph.declaration(inside[0]).unwrap().syntax().kind,
                FileDeclarationKind::Procedure(_)
            ));
            let exposed = graph.operator_declarations(root, kind);
            assert_eq!(
                exposed,
                if visibility.is_empty() {
                    inside
                } else {
                    vec![]
                }
            );
        }
        assert!(
            graph
                .operator_declarations(root, OperatorKind::Binary(BinaryOp::Add))
                .is_empty()
        );
    }
}

#[test]
fn invalid_operator_alias_namespaces_and_private_targets_are_located_errors() {
    for (namespace, target, expected) in [
        ("", "Missing", "unknown operator alias namespace"),
        ("Library::1;", "Library", "source namespace"),
        (
            "Library::#import,file \"math.jai\";",
            "Library",
            "no exported declarations",
        ),
    ] {
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                Path::new("/operators/main.jai"),
                format!("{namespace} operator-::{target}.operator-;").into_bytes(),
            )
            .unwrap();
        overlay
            .insert(
                Path::new("/operators/math.jai"),
                b"#scope_module; Box::struct{value:int;} operator -::(a:Box)->Box{return a;}"
                    .to_vec(),
            )
            .unwrap();
        let error = ModuleGraph::load_with_provider(
            Path::new("/operators/main.jai"),
            Default::default(),
            &overlay,
        )
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
        assert!(
            matches!(error, jai_modules::GraphError::Located { .. }),
            "alias errors must retain original source locations"
        );
    }
}
