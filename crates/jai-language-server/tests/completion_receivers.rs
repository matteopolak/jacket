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
