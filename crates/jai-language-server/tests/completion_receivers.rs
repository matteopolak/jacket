//! Member completion after receivers no name lookup leads to.
mod common;
use common::{cursor, file_uri, lean_session};

fn labels(source: &str) -> Vec<String> {
    let (text, at) = cursor(source);
    let mut session = lean_session();
    let uri = file_uri("/receivers/main.jai");
    session.open(uri.clone(), 1, text).unwrap();
    let list = session.completion(&uri, at).unwrap();
    list.items.into_iter().map(|i| i.label).collect()
}

const COUNTER: &str = "Counter :: struct { from: int; step: int; }\n";

#[test]
fn a_macro_parameter() {
    let labels = labels(&format!(
        "{COUNTER}peek :: (c: Counter) -> int #expand {{ return c.|from; }}\nmain :: () {{ c: Counter; x := peek(c); }}"
    ));
    assert!(labels.contains(&"step".to_string()), "{labels:?}");
}

#[test]
fn a_call_result() {
    let labels = labels(&format!(
        "{COUNTER}make :: () -> Counter {{ c: Counter; return c; }}\nmain :: () {{ x := make().|from; }}"
    ));
    assert!(labels.contains(&"step".to_string()), "{labels:?}");
}

#[test]
fn an_indexed_element() {
    let labels = labels(&format!(
        "{COUNTER}main :: () {{ list: [3] Counter; x := list[0].|from; }}"
    ));
    assert!(labels.contains(&"step".to_string()), "{labels:?}");
}

/// A local declared further down the block does not yet hide the global it shadows.
#[test]
fn a_name_shadowed_only_below_the_cursor() {
    let labels = labels(&format!(
        "{COUNTER}Other :: struct {{ a: int; b: int; }}\nc: Counter;\nmain :: () {{ x := c.|from; c: Other; y := c.a; }}"
    ));
    assert!(labels.contains(&"step".to_string()), "{labels:?}");
    assert!(!labels.contains(&"b".to_string()), "{labels:?}");
}

/// A polymorphic body is checked once per instance: the members are those of every instance, so a
/// branch only one instance takes still completes.
#[test]
fn a_parameter_of_a_polymorphic_procedure_offers_every_instance() {
    let labels = labels(
        "Pair :: struct { first: int; second: int; }\n\
         Named :: struct { label: string; }\n\
         pick :: (a: $T) {\n    #if T == Pair { x := a.|first; } else { y := a; }\n}\n\
         main :: () { p: Pair; n: Named; pick(p); pick(n); }",
    );
    assert!(labels.contains(&"second".to_string()), "{labels:?}");
}
