use super::*;

fn overlay(main: &str, provider: &str, names: &str) -> SourceOverlay {
    let mut sources = SourceOverlay::new();
    for (path, text) in [
        ("/suspended-imports/main.jai", main),
        ("/suspended-imports/provider.jai", provider),
        ("/suspended-imports/names.jai", names),
    ] {
        sources
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    sources
}

fn lookup(
    graph: &ModuleGraph,
    file: FileInstanceId,
    names: &[&str],
) -> Result<Binding, LookupError> {
    let symbols = names
        .iter()
        .map(|name| graph.symbols().find(name).unwrap())
        .collect::<Vec<_>>();
    graph.lookup(
        file,
        &jai_syntax::NamePath {
            root: symbols[0],
            members: symbols[1..].to_vec(),
        },
    )
}

#[test]
fn suspended_imports_retain_real_namespace_declarations_and_privacy() {
    let sources = overlay(
        "Native::#import,file \"provider.jai\"; Level::Native.Width; main::(){}",
        "Width::u64; using Flags::enum_flags Width{FIRST::1;} #scope_module hidden::3;",
        "",
    );
    let mut discovery = GraphDiscovery::new(
        Path::new("/suspended-imports/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let file = discovery
        .graph()
        .module(discovery.graph().root())
        .unwrap()
        .entry();
    let namespace = lookup(discovery.graph(), file, &["Native"]).unwrap();
    let width = lookup(discovery.graph(), file, &["Native", "Width"]).unwrap();
    assert!(matches!(
        lookup(discovery.graph(), file, &["Native", "hidden"]),
        Err(LookupError::PrivateMember { .. })
    ));
    let request = discovery.pending_using_requests().remove(0);
    for _ in 0..3 {
        assert!(!discovery.advance().unwrap().is_complete());
        assert_eq!(lookup(discovery.graph(), file, &["Native"]), Ok(namespace));
        assert_eq!(
            lookup(discovery.graph(), file, &["Native", "Width"]),
            Ok(width)
        );
        assert_eq!(discovery.pending_using_requests()[0].id, request.id);
        assert_eq!(discovery.graph().imports().len(), 1);
    }
    let first = discovery.graph().symbols().find("FIRST").unwrap();
    let Binding::Declaration(flags) =
        lookup(discovery.graph(), file, &["Native", "Flags"]).unwrap()
    else {
        panic!("original enum declaration");
    };
    let member = Binding::SourceMember {
        declaration: flags,
        member: first,
    };
    discovery
        .resolve_using(
            request.id,
            FileUsingDecision {
                bindings: vec![UsingBinding {
                    name: "FIRST".into(),
                    binding: member,
                }],
                ..Default::default()
            },
        )
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().unwrap();
    assert_eq!(lookup(&graph, file, &["Native", "Width"]), Ok(width));
    assert_eq!(lookup(&graph, file, &["Native", "FIRST"]), Ok(member));
    assert_eq!(graph.imports().len(), 1);
}

#[test]
fn anonymous_suspended_imports_refresh_exports_after_real_using_publication() {
    let sources = overlay(
        "#import,file \"provider.jai\"; main::(){}",
        "Names::#import,file \"names.jai\"; Width::u64; using Names;",
        "answer::42;",
    );
    let mut discovery = GraphDiscovery::new(
        Path::new("/suspended-imports/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let file = graph.module(graph.root()).unwrap().entry();
    let width = lookup(graph, file, &["Width"]).unwrap();
    let Binding::Module(names) = lookup(graph, file, &["Names"]).unwrap() else {
        panic!("actual imported namespace");
    };
    let answer = graph.module(names).unwrap().exports()[&graph.symbols().find("answer").unwrap()];
    assert!(lookup(graph, file, &["answer"]).is_err());
    let request = discovery.pending_using_requests().remove(0);
    discovery
        .resolve_using(
            request.id,
            FileUsingDecision {
                bindings: vec![UsingBinding {
                    name: "answer".into(),
                    binding: answer,
                }],
                source_module: Some(names),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().unwrap();
    assert_eq!(lookup(&graph, file, &["Width"]), Ok(width));
    assert_eq!(lookup(&graph, file, &["answer"]), Ok(answer));
    assert_eq!(graph.imports().len(), 2);
}

#[test]
fn conditional_provider_exports_refresh_without_selecting_inactive_declarations() {
    let sources = overlay(
        "Native::#import,file \"provider.jai\"; Level::Native.Width; main::(){}",
        "Width::u64; choose::()->bool{return true;} #if choose(){selected::42;}else{discarded::0;}",
        "",
    );
    let mut discovery = GraphDiscovery::new(
        Path::new("/suspended-imports/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let file = graph.module(graph.root()).unwrap().entry();
    let width = lookup(graph, file, &["Native", "Width"]).unwrap();
    assert!(lookup(graph, file, &["Native", "selected"]).is_err());
    let condition = discovery.pending_conditions().next().unwrap().id;
    discovery.select_condition(condition, true).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().unwrap();
    assert_eq!(lookup(&graph, file, &["Native", "Width"]), Ok(width));
    assert!(lookup(&graph, file, &["Native", "selected"]).is_ok());
    assert!(lookup(&graph, file, &["Native", "discarded"]).is_err());
    assert_eq!(graph.imports().len(), 1);
}

#[test]
fn suspended_lexical_imports_never_publish_names_in_the_file_scope() {
    let sources = overlay(
        "main::(){Native::#import,file \"provider.jai\";}",
        "Width::u64; using Flags::enum_flags Width{FIRST::1;}",
        "",
    );
    let mut discovery = GraphDiscovery::new(
        Path::new("/suspended-imports/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let file = graph.module(graph.root()).unwrap().entry();
    assert!(lookup(graph, file, &["Native"]).is_err());
    assert!(lookup(graph, file, &["Width"]).is_err());
    let request = discovery.pending_using_requests().remove(0);
    let Binding::Declaration(flags) = lookup(discovery.graph(), request.file, &["Flags"]).unwrap()
    else {
        panic!("original enum declaration");
    };
    let first = discovery.graph().symbols().find("FIRST").unwrap();
    discovery
        .resolve_using(
            request.id,
            FileUsingDecision {
                bindings: vec![UsingBinding {
                    name: "FIRST".into(),
                    binding: Binding::SourceMember {
                        declaration: flags,
                        member: first,
                    },
                }],
                ..Default::default()
            },
        )
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().unwrap();
    assert!(lookup(&graph, file, &["Native"]).is_err());
    assert!(lookup(&graph, file, &["Width"]).is_err());
    assert!(graph.imports().is_empty());
    assert_eq!(graph.scoped_imports().len(), 1);
}

#[test]
fn pending_import_arguments_do_not_publish_a_module_instance() {
    let sources = overlay(
        "Native::#import,file \"provider.jai\"(Width=MISSING); main::(){}",
        "#module_parameters(Width:Type); using Flags::enum_flags Width{FIRST::1;}",
        "",
    );
    let mut discovery = GraphDiscovery::new(
        Path::new("/suspended-imports/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let file = graph.module(graph.root()).unwrap().entry();
    assert!(lookup(graph, file, &["Native"]).is_err());
    assert!(graph.imports().is_empty());
    assert!(discovery.pending_using_requests().is_empty());
}
