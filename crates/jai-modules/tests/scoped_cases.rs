//! Selected source case dependencies retain lexical ownership and original spans.
use jai_modules::{
    DiscoveryConditionContext, GraphDiscovery, GraphOptions, ModuleGraph, SourceOverlay,
};
use jai_syntax::CompileTimeCaseChoice;
use std::path::Path;

const ENTRY: &str = "/scoped-cases/main.jai";

fn provider(source: &str) -> SourceOverlay {
    let mut provider = SourceOverlay::new();
    for (path, bytes) in [
        (ENTRY, source.as_bytes()),
        ("/scoped-cases/dependency.jai", b"answer :: 42;".as_slice()),
    ] {
        provider.insert(Path::new(path), bytes.to_vec()).unwrap();
    }
    provider
}

fn graph(source: &str) -> ModuleGraph {
    ModuleGraph::load_with_provider(Path::new(ENTRY), GraphOptions::default(), &provider(source))
        .unwrap()
}

#[test]
fn literal_case_uses_lexical_constant_and_only_selected_fallthrough_imports() {
    let graph = graph(
        r#"
        choice :: 1;
        main :: () {
            choice :: 2;
            #if choice == {
                case 1; Missing :: #import,file "absent.jai";
                case 2; First :: #import,file "dependency.jai"; #through;
                case 3; Second :: #import,file "./dependency.jai";
                case; MissingDefault :: #import,file "absent-default.jai";
            }
        }
        "#,
    );
    assert_eq!(graph.scoped_imports().len(), 2);
    assert_eq!(
        graph.scoped_imports()[0].module(),
        graph.scoped_imports()[1].module()
    );
    assert!(graph.imports().is_empty());
    assert_eq!(graph.source_cases().len(), 1);
    assert_eq!(
        graph.source_cases()[0].choice,
        CompileTimeCaseChoice::Arm(1)
    );
}

#[test]
fn anonymous_case_promotes_only_active_fields_before_parent_method_guards() {
    let graph = graph(
        r#"
        enabled :: false;
        R :: struct {
            union {
                choice :: 2;
                #if choice == {
                    case 1; enabled:bool;
                    case 2; other:int;
                }
            }
            method :: () {
                #if enabled { Missing :: #import,file "absent.jai"; }
                else { Lib :: #import,file "dependency.jai"; }
            }
        }
        main :: () {}
        "#,
    );
    assert_eq!(graph.scoped_imports().len(), 1);
    assert_eq!(
        graph.source_cases()[0].choice,
        CompileTimeCaseChoice::Arm(1)
    );
}

#[test]
fn suspended_case_preserves_original_header_and_loads_independent_later_import() {
    let source = r#"
        choose :: () -> int { return 2; }
        main :: () {
            #if #run choose() == {
                case 1; Missing :: #import,file "absent.jai";
                case 2; Selected :: #import,file "dependency.jai";
            }
            Later :: #import,file "./dependency.jai";
        }
    "#;
    let provider = provider(source);
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
    let request = discovery.pending_cases().next().unwrap();
    assert_eq!(request.header.value.span.text(source), "#run choose()");
    assert!(
        matches!(&request.context, DiscoveryConditionContext::Lexical { scopes, .. }
        if scopes.iter().any(|scope| scope.statements.iter().any(|statement|
            matches!(statement.kind, jai_syntax::StatementKind::CompileTimeCases(_)))))
    );
    assert_eq!(request.location.span, request.header.span);
    let request = request.id;
    discovery
        .select_case(request, CompileTimeCaseChoice::Arm(1))
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 2);
    assert_eq!(
        discovery.graph().scoped_imports()[0].module(),
        discovery.graph().scoped_imports()[1].module()
    );
}

#[test]
fn suspended_case_alias_shadows_same_spelled_file_namespace_until_selection() {
    let source = r#"
        Flags :: #import,file "file-flags.jai";
        choose :: () -> int { return 1; }
        main :: () {
            #if #run choose() == {
                case 1; Flags :: #import,file "local-flags.jai";
                case 2; Missing :: #import,file "absent-case.jai";
            }
            #if Flags.enabled { Lib :: #import,file "dependency.jai"; }
            else { Missing :: #import,file "absent-guard.jai"; }
        }
    "#;
    let mut provider = provider(source);
    for (path, bytes) in [
        (
            "/scoped-cases/file-flags.jai",
            b"enabled :: false;".as_slice(),
        ),
        (
            "/scoped-cases/local-flags.jai",
            b"enabled :: true;".as_slice(),
        ),
    ] {
        provider.insert(Path::new(path), bytes.to_vec()).unwrap();
    }
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert!(discovery.graph().scoped_imports().is_empty());
    let case = discovery.pending_cases().next().unwrap().id;
    let guard = discovery.pending_conditions().next().unwrap();
    assert_eq!(guard.location.span.text(source), "Flags.enabled");
    let guard = guard.id;
    discovery
        .select_case(case, CompileTimeCaseChoice::Arm(0))
        .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
    discovery.select_condition(guard, true).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 2);
}

#[test]
fn suspended_anonymous_and_using_imports_hold_outer_constant_guards_for_real_exports() {
    for import in [
        "#import,file \"local-flags.jai\";",
        "using Flags :: #import,file \"local-flags.jai\";",
    ] {
        let source = format!(
            r#"
            enabled :: false;
            choose :: () -> int {{ return 1; }}
            main :: () {{
                #if #run choose() == {{
                    case 1; {import}
                    case 2; Missing :: #import,file "absent-case.jai";
                }}
                #if enabled {{ Lib :: #import,file "dependency.jai"; }}
                else {{ Missing :: #import,file "absent-guard.jai"; }}
            }}
            "#,
        );
        let mut provider = provider(&source);
        provider
            .insert(
                Path::new("/scoped-cases/local-flags.jai"),
                b"enabled :: true;".to_vec(),
            )
            .unwrap();
        let mut discovery =
            GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        assert!(discovery.graph().scoped_imports().is_empty());
        let case = discovery.pending_cases().next().unwrap().id;
        let guard = discovery.pending_conditions().next().unwrap();
        assert_eq!(guard.location.span.text(&source), "enabled");
        let guard = guard.id;
        discovery
            .select_case(case, CompileTimeCaseChoice::Arm(0))
            .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        assert_eq!(discovery.graph().scoped_imports().len(), 1);
        discovery.select_condition(guard, true).unwrap();
        assert!(discovery.advance().unwrap().is_complete());
        assert_eq!(discovery.graph().scoped_imports().len(), 2);
    }
}

#[test]
fn actual_same_block_constant_shadows_unresolved_using_exports() {
    let source = r#"
        choose :: () -> int { return 1; }
        main :: () {
            enabled :: false;
            #if #run choose() == {
                case 1; #import,file "dependency.jai";
                case 2; Missing :: #import,file "absent-case.jai";
            }
            #if enabled { Missing :: #import,file "absent-guard.jai"; }
            else { Later :: #import,file "./dependency.jai"; }
        }
    "#;
    let provider = provider(source);
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
    assert_eq!(discovery.pending_conditions().count(), 0);
    let case = discovery.pending_cases().next().unwrap().id;
    discovery
        .select_case(case, CompileTimeCaseChoice::Arm(0))
        .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 2);
}
