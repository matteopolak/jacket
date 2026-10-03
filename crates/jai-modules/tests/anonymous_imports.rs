//! Source graph acceptance; no procedures or supplied native assets are executed.
use jai_modules::{
    DiscoveryConditionContext, GraphDiscovery, GraphOptions, ModuleGraph, SourceOverlay,
};
use jai_syntax::{FileDeclarationKind, RecordMember};
use std::path::Path;

const ENTRY: &str = "/anonymous-imports/main.jai";

fn provider(source: &str) -> SourceOverlay {
    let mut provider = SourceOverlay::new();
    provider
        .insert(Path::new(ENTRY), source.as_bytes().to_vec())
        .unwrap();
    provider
        .insert(
            Path::new("/anonymous-imports/dependency.jai"),
            b"value :: 42; #scope_module; private_value :: 9;".to_vec(),
        )
        .unwrap();
    provider
}

fn graph(source: &str) -> ModuleGraph {
    ModuleGraph::load_with_provider(Path::new(ENTRY), GraphOptions::default(), &provider(source))
        .unwrap()
}

#[test]
fn anonymous_child_constants_shadow_locally_without_changing_parent_method_guards() {
    let graph = graph(
        r#"
        R :: struct {
            enabled :: false;
            union {
                enabled :: true;
                value:int;
                method :: () {
                    #if enabled { Lib :: #import,file "dependency.jai"; }
                    else { Missing :: #import,file "absent.jai"; }
                }
            }
            sibling :: () {
                #if enabled { Missing :: #import,file "absent.jai"; }
                else { Lib :: #import,file "dependency.jai"; }
            }
        }
        main :: () {}
        "#,
    );
    assert_eq!(graph.scoped_imports().len(), 2);
    assert_eq!(
        graph.scoped_imports()[0].module(),
        graph.scoped_imports()[1].module()
    );
    let root = graph.module(graph.root()).unwrap().entry();
    assert_eq!(
        graph
            .declarations()
            .iter()
            .filter(|declaration| declaration.file() == root)
            .count(),
        2,
        "child declarations must not become fabricated file declarations"
    );
    assert!(graph.imports().is_empty());
}

#[test]
fn selected_anonymous_child_methods_keep_nested_constants_and_skip_missing_dependencies() {
    let graph = graph(
        r#"
        R :: struct {
            union {
                #if true {
                    enabled :: false;
                    struct {
                        value:int;
                        method :: () {
                            #if enabled { Missing :: #import,file "absent.jai"; }
                            else { Lib :: #import,file "dependency.jai"; }
                        }
                    }
                } else {
                    wrong :: () { Missing :: #import,file "absent.jai"; }
                }
            }
        }
        main :: () {}
        "#,
    );
    assert_eq!(graph.scoped_imports().len(), 1);
}

#[test]
fn deferred_anonymous_method_guard_keeps_original_tree_runtime_names_and_source_span() {
    let source = r#"
        enabled :: false;
        R :: struct {
            union {
                enabled:bool;
                value:int;
                method :: () { #if enabled { Lib :: #import,file "dependency.jai"; } }
            }
        }
        main :: () {}
    "#;
    let provider = provider(source);
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let pending = discovery.pending_conditions().next().unwrap();
    assert_eq!(pending.location.span.text(source), "enabled");
    let owner = pending.location.source;
    assert_eq!(
        discovery.graph().sources().get(owner).unwrap().path(),
        Path::new(ENTRY)
    );
    let DiscoveryConditionContext::Lexical {
        declaration,
        scopes,
    } = &pending.context
    else {
        panic!("anonymous method guard must retain its lexical defining environment")
    };
    let graph = discovery.graph();
    let enabled = graph.symbols().find("enabled").unwrap();
    assert!(
        scopes
            .iter()
            .any(|scope| scope.runtime_names.contains(&enabled))
    );
    assert!(
        scopes
            .iter()
            .any(|scope| scope.record_members.iter().any(|member| {
                matches!(member, RecordMember::AnonymousRecord(record)
            if record.members.iter().any(|member| matches!(member, RecordMember::Procedure(_))))
            }))
    );
    assert!(matches!(
        graph.declaration(*declaration).unwrap().syntax().kind,
        FileDeclarationKind::Record(_)
    ));
    let request = pending.id;
    discovery.select_condition(request, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert!(discovery.graph().scoped_imports().is_empty());
}

#[test]
fn promoted_anonymous_field_shadows_file_constant_in_parent_method_guard() {
    let source = r#"
        enabled :: false;
        R :: struct {
            union { enabled:bool; other:int; }
            method :: () { #if enabled { Lib :: #import,file "dependency.jai"; } }
        }
        main :: () {}
    "#;
    let provider = provider(source);
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let pending = discovery.pending_conditions().next().unwrap();
    let DiscoveryConditionContext::Lexical { scopes, .. } = &pending.context else {
        panic!("promoted physical field must retain a runtime lexical shadow")
    };
    let enabled = discovery.graph().symbols().find("enabled").unwrap();
    assert!(
        scopes
            .iter()
            .any(|scope| scope.runtime_names.contains(&enabled))
    );
    assert_eq!(pending.location.span.text(source), "enabled");
    let request = pending.id;
    discovery.select_condition(request, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert!(discovery.graph().scoped_imports().is_empty());
}

#[test]
fn inactive_anonymous_field_does_not_shadow_file_constant_in_parent_method_guard() {
    let graph = graph(
        r#"
        enabled :: false;
        R :: struct {
            union {
                #if false { enabled:bool; }
                else { other:int; }
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
}

#[test]
fn deferred_anonymous_layout_preserves_child_context_and_loads_independent_parent_import() {
    let source = r#"
        enabled :: false;
        R :: struct {
            union {
                #if #run true { enabled:bool; }
                else { other:int; }
            }
            guarded :: () { #if enabled { Missing :: #import,file "absent.jai"; } }
            independent :: () { Lib :: #import,file "dependency.jai"; }
        }
        main :: () {}
    "#;
    let provider = provider(source);
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
    let pending = discovery
        .pending_conditions()
        .find(|condition| condition.location.span.text(source).contains("#run"))
        .unwrap();
    let DiscoveryConditionContext::Lexical { scopes, .. } = &pending.context else {
        panic!("child layout guard must retain its actual defining record context")
    };
    let enabled = discovery.graph().symbols().find("enabled").unwrap();
    assert!(
        scopes
            .iter()
            .all(|scope| !scope.runtime_names.contains(&enabled))
    );
    assert!(
        scopes
            .iter()
            .any(|scope| scope.record_members.iter().any(|member| {
                matches!(member, RecordMember::Conditional { condition, .. }
            if condition.span.text(source).contains("#run"))
            }))
    );
    let request = pending.id;
    discovery.select_condition(request, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
}

#[test]
fn deferred_record_method_selection_still_loads_later_independent_method_import() {
    let source = r#"
        R :: struct {
            #if #run true { unresolved :: () { Missing :: #import,file "absent.jai"; } }
            independent :: () { Lib :: #import,file "dependency.jai"; }
        }
        main :: () {}
    "#;
    let provider = provider(source);
    let mut discovery =
        GraphDiscovery::new(Path::new(ENTRY), GraphOptions::default(), &provider).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
    assert_eq!(discovery.pending_conditions().count(), 1);
    let pending = discovery.pending_conditions().next().unwrap();
    assert!(pending.location.span.text(source).contains("#run"));
    let request = pending.id;
    discovery.select_condition(request, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
}
