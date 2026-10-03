//! Multi-result declarations retain real source names before typed execution.
use jai_modules::{DiscoveryConditionContext, GraphDiscovery, GraphOptions, SourceOverlay};
use jai_syntax::StatementKind;
use std::path::Path;

#[test]
fn local_result_name_holds_file_constant_selection_until_its_real_source_decision() {
    let source = r#"
enabled :: false;
choose :: ()->(bool,int) { return true,9; }
main :: () {
    enabled,_ :: #run choose();
    #if enabled { Lib :: #import,file "dependency.jai"; }
    else { Missing :: #import,file "absent.jai"; }
}
"#;
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(
            Path::new("/result-shadow/main.jai"),
            source.as_bytes().to_vec(),
        )
        .unwrap();
    overlay
        .insert(
            Path::new("/result-shadow/dependency.jai"),
            b"ANSWER::42;".to_vec(),
        )
        .unwrap();
    let mut discovery = GraphDiscovery::new(
        Path::new("/result-shadow/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert!(discovery.graph().scoped_imports().is_empty());
    let request = discovery.pending_conditions().next().unwrap().clone();
    assert_eq!(request.location.span.text(source), "enabled");
    let DiscoveryConditionContext::Lexical {
        scopes, ..
    } = &request.context
    else {
        panic!("result source lost its lexical context");
    };
    let group = scopes
        .iter()
        .flat_map(|scope| &scope.statements)
        .find_map(|statement| {
            if let StatementKind::ConstantResults(group) = &statement.kind {
                Some(group)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(group.names.len(), 2);
    assert_eq!(group.initializer.span.text(source), "#run choose()");
    for scope in scopes {
        assert!(
            scope
                .runtime_names
                .iter()
                .all(|&name| discovery.graph().symbols().name(name) != "enabled")
        );
    }
    discovery.select_condition(request.id, true).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.into_graph().unwrap().scoped_imports().len(), 1);
}
