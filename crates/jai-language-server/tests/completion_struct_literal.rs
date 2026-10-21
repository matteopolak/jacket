//! Completion offers the fields of the struct a literal builds where a field name goes.
use jai_language_server::CompletionItem;
mod common;
use common::{cursor, file_uri, lean_session};

fn items(source: &str) -> Vec<CompletionItem> {
    let (text, at) = cursor(source);
    let mut session = lean_session();
    let uri = file_uri("/struct-literal/main.jai");
    session.open(uri.clone(), 1, text).unwrap();
    session.completion(&uri, at).unwrap().items
}

fn labels(source: &str) -> Vec<String> {
    items(source).into_iter().map(|i| i.label).collect()
}

fn has(labels: &[String], name: &str) -> bool {
    labels.iter().any(|l| l == name)
}

const FOO: &str = "Foo :: struct { a1: s32; b2: string; c3: bool; }\n";

#[test]
fn a_typed_literal_offers_the_fields_with_their_types() {
    let source = format!("{FOO}main :: () {{ x := Foo.{{ | }}; }}");
    let items = items(&source);
    let field = |n: &str| items.iter().find(|i| i.label == n).unwrap();
    assert_eq!(field("a1").detail, "s32");
    assert_eq!(field("b2").detail, "string");
    assert_eq!(field("c3").insert_text.as_deref(), Some("c3 = "));
}

#[test]
fn assigned_fields_are_skipped() {
    let l = labels(&format!("{FOO}main :: () {{ x := Foo.{{ a1 = 1, | }}; }}"));
    assert!(has(&l, "b2") && has(&l, "c3") && !has(&l, "a1"), "{l:?}");
    let l = labels(&format!("{FOO}main :: () {{ x := Foo.{{ | , a1 = 1 }}; }}"));
    assert!(has(&l, "b2") && !has(&l, "a1"), "{l:?}");
}

#[test]
fn after_a_named_field_only_fields_are_offered() {
    let l = labels(&format!("{FOO}main :: () {{ x := Foo.{{ a1 = 1, | }}; }}"));
    assert!(!has(&l, "main"), "{l:?}");
}

#[test]
fn a_partial_name_filters() {
    let l = labels(&format!("{FOO}main :: () {{ x := Foo.{{ b| }}; }}"));
    assert!(has(&l, "b2") && !has(&l, "a1"), "{l:?}");
}

#[test]
fn the_type_comes_from_the_context() {
    for body in [
        "x: Foo = .{ | };",
        "take(.{ | });",
        "x: Foo; x = .{ | };",
        "x: Foo = .{ a1 = 1, | };",
    ] {
        let l = labels(&format!(
            "{FOO}take :: (f: Foo) {{}}\nmain :: () {{ {body} }}"
        ));
        assert!(has(&l, "b2") && has(&l, "c3"), "{body}: {l:?}");
    }
    let l = labels(&format!("{FOO}make :: () -> Foo {{ return .{{ | }}; }}"));
    assert!(has(&l, "b2"), "{l:?}");
}

#[test]
fn a_literal_nested_in_a_field_uses_the_field_type() {
    let l = labels(&format!(
        "{FOO}Bar :: struct {{ inner: Foo; n: s32; }}\nmain :: () {{ x := Bar.{{ inner = .{{ | }} }}; }}"
    ));
    assert!(has(&l, "a1") && !has(&l, "inner"), "{l:?}");
}

#[test]
fn using_fields_are_offered() {
    let l = labels(&format!(
        "{FOO}Bar :: struct {{ using foo: Foo; n: s32; }}\nmain :: () {{ x := Bar.{{ | }}; }}"
    ));
    assert!(has(&l, "a1") && has(&l, "n"), "{l:?}");
}

#[test]
fn polymorphic_instances_are_offered() {
    let l = labels(
        "Pair :: struct ($T: Type) { first: T; second: T; }\nmain :: () { x := Pair(s32).{ | }; }",
    );
    assert!(has(&l, "first") && has(&l, "second"), "{l:?}");
}

#[test]
fn positional_values_are_expressions() {
    let l = labels(&format!(
        "{FOO}main :: () {{ local := 1; x := Foo.{{ 1, | }}; }}"
    ));
    assert!(has(&l, "local") && !has(&l, "b2"), "{l:?}");
}

#[test]
fn a_value_after_equals_is_an_expression() {
    let l = labels(&format!(
        "{FOO}main :: () {{ local := 1; x := Foo.{{ a1 = | }}; }}"
    ));
    assert!(has(&l, "local") && !has(&l, "b2"), "{l:?}");
}

#[test]
fn the_first_position_keeps_expressions() {
    let l = labels(&format!(
        "{FOO}main :: () {{ local := 1; x := Foo.{{ | }}; }}"
    ));
    assert!(has(&l, "a1") && has(&l, "local"), "{l:?}");
}

#[test]
fn a_value_still_ranks_by_the_expected_type() {
    let l = items(&format!(
        "{FOO}main :: () {{ a_text := \"\"; z_n: s32 = 1; x := Foo.{{ a1 = | }}; }}"
    ));
    let key = |n: &str| l.iter().find(|i| i.label == n).unwrap().sort_text.clone();
    assert!(key("z_n") < key("a_text"));
}
