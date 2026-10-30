//! Completion offers the fields of the struct a literal builds where a field name goes.
use jai_language_server::CompletionItem;
use test_support::lsp::{cursor, file_uri, lean_session};

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

#[test]
fn bare_braces_after_a_typed_declaration_are_a_literal() {
    for body in [
        "x: Foo = { | };",
        "x: Foo = {\n    |\n};",
        "x: Foo = { a1 = 1, | };",
        "x: Foo; x = { | };",
    ] {
        let l = labels(&format!("{FOO}main :: () {{ {body} }}"));
        assert!(has(&l, "b2") && has(&l, "c3"), "{body}: {l:?}");
    }
    let l = labels(&format!(
        "{FOO}Bar :: struct {{ using foo: Foo; n: s32; }}\nmain :: () {{ x: Bar = {{\n    |\n}}; }}"
    ));
    assert!(has(&l, "a1") && has(&l, "n"), "{l:?}");
}

#[test]
fn a_block_is_not_a_literal() {
    let l = labels(&format!("{FOO}main :: () {{ f: Foo; if true {{ | }} }}"));
    assert!(!has(&l, "a1") && has(&l, "f"), "{l:?}");
}

#[test]
fn generic_arguments_supply_literal_fields_from_other_arguments() {
    for parameters in ["array: *[..] $T, item: T", "array: *[..] $T, items: ..T"] {
        for arguments in ["*array, .{ | }", "array = *array, .{ | }"] {
            let source = format!(
                "{FOO}take :: ({parameters}) {{}}\nmain :: () {{ array: [..] Foo; take({arguments}); }}"
            );
            let l = labels(&source);
            assert!(has(&l, "a1") && has(&l, "b2"), "{source}: {l:?}");
        }
    }
}

#[test]
fn generic_literal_can_precede_the_binding_argument() {
    let l = labels(&format!(
        "{FOO}take :: (item: T, array: *[..] $T) {{}}\nmain :: () {{ array: [..] Foo; take(.{{ | }}, *array); }}"
    ));
    assert!(has(&l, "a1") && has(&l, "b2"), "{l:?}");
}

#[test]
fn array_add_offers_element_fields() {
    let (text, at) = cursor(&format!(
        "#import \"Basic\";\n{FOO}main :: () {{ array: [..] Foo; array_add(*array, .{{ | }}); }}"
    ));
    let mut session = jai_language_server::Session::with_environment(
        jai_language_server::Limits::default(),
        test_support::lsp::environment(),
    );
    let uri = file_uri("/struct-literal/array-add.jai");
    session.open(uri.clone(), 1, text).unwrap();
    let items = session.completion(&uri, at).unwrap().items;
    assert!(
        items
            .iter()
            .any(|item| item.label == "a1" && item.detail == "s32")
    );
}

#[test]
fn inline_and_multiline_fields_have_equivalent_sort_priority() {
    for literal in ["Foo.{|}", "Foo.{ | }", "Foo.{\n |\n}"] {
        let list = items(&format!("{FOO}main :: () {{ value := {literal}; }}"));
        for (index, name) in ["a1", "b2", "c3"].into_iter().enumerate() {
            let item = list.iter().find(|item| item.label == name).unwrap();
            assert_eq!(item.sort_text, Some(format!("!{index:05}")), "{literal}");
            assert_eq!(item.preselect, index == 0, "{literal}");
        }
    }
}

#[test]
fn unresolved_generic_bindings_do_not_invent_literal_fields() {
    let l = labels(&format!(
        "{FOO}take :: (item: $T) {{}}\nmain :: () {{ take(.{{ | }}); }}"
    ));
    assert!(!has(&l, "a1"), "{l:?}");
}

#[test]
fn inline_and_multiline_wire_sort_keys_survive_edits() {
    use serde_json::json;
    let mut session = jai_language_server::JsonSession::with_environment(
        jai_language_server::Limits::default(),
        test_support::lsp::lean_environment(),
    );
    session
        .handle_json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}).to_string())
        .unwrap();
    session
        .handle_json(&json!({"jsonrpc":"2.0","method":"initialized","params":{}}).to_string())
        .unwrap();
    let uri = file_uri("/struct-literal/edits.jai");
    for (version, literal) in ["Foo.{|}", "Foo.{\n |\n}", "Foo.{ | }"].iter().enumerate() {
        let (text, at) = cursor(&format!("{FOO}main :: () {{ value := {literal}; }}"));
        let method = if version == 0 {
            "textDocument/didOpen"
        } else {
            "textDocument/didChange"
        };
        let params = if version == 0 {
            json!({"textDocument":{"uri":uri.as_str(),"languageId":"jai","version":1,"text":text}})
        } else {
            json!({"textDocument":{"uri":uri.as_str(),"version":version+1},"contentChanges":[{"text":text}]})
        };
        session
            .handle_json(&json!({"jsonrpc":"2.0","method":method,"params":params}).to_string())
            .unwrap();
        let responses = session.handle_json(&json!({"jsonrpc":"2.0","id":version+10,"method":"textDocument/completion","params":{"textDocument":{"uri":uri.as_str()},"position":at,"context":{"triggerKind":1}}}).to_string()).unwrap();
        let responses: Vec<serde_json::Value> = responses
            .iter()
            .map(|text| serde_json::from_str(text).unwrap())
            .collect();
        let items = responses
            .iter()
            .find_map(|response| response.get("result")?.get("items")?.as_array())
            .unwrap();
        let item = items.iter().find(|item| item["label"] == "a1").unwrap();
        assert_eq!(item["sortText"], "!00000");
        assert_eq!(item["preselect"], true);
        assert_eq!(item["textEdit"]["newText"], "a1 = ");
    }
}

/// The fields' order and selection, as the client sorts them (`sortText`, then the label).
fn sorted_fields(source: &str) -> Vec<(String, bool)> {
    let mut items: Vec<_> = items(source)
        .into_iter()
        .filter(|i| i.kind == jai_language_server::CompletionKind::Field)
        .collect();
    items.sort_by(|a, b| {
        let key = |i: &CompletionItem| i.sort_text.clone().unwrap_or_else(|| i.label.clone());
        key(a).cmp(&key(b)).then(a.label.cmp(&b.label))
    });
    items.into_iter().map(|i| (i.label, i.preselect)).collect()
}

#[test]
fn fields_keep_declaration_order_and_the_first_left_is_preselected() {
    let zyx = "Zyx :: struct { z: int; y: int; x: int; }\n";
    let order = sorted_fields(&format!("{zyx}main :: () {{ v := Zyx.{{|}}; }}"));
    assert_eq!(
        order,
        [("z".into(), true), ("y".into(), false), ("x".into(), false)]
    );
    let order = sorted_fields(&format!("{zyx}main :: () {{ v := Zyx.{{ z = 1, | }}; }}"));
    assert_eq!(order, [("y".into(), true), ("x".into(), false)]);
}

#[test]
fn a_literal_of_a_type_parameter_inside_type_of() {
    let l = labels(
        "Inner :: struct { x: int; }\nOuter :: struct { inner: Inner; y: int; }\n\
         pick :: ($T: Type) -> Type { return type_of(T.{ | }.inner); }\n\
         main :: () { t: pick(Outer); }",
    );
    assert!(has(&l, "inner") && has(&l, "y"), "{l:?}");
}

#[test]
fn fields_after_assignments_through_paths() {
    let l = labels(
        "Handlers :: struct { on_start: int; count: int; }\n\
         Widget :: struct { #as base: Handlers; extra: int; }\n\
         main :: () { w := Widget.{ base.on_start = 1, base.count = 3, | }; }",
    );
    assert!(has(&l, "extra") && has(&l, "base"), "{l:?}");
}

#[test]
fn the_return_types_of_a_procedure_value_do_not_count_as_positional_values() {
    let l = labels(
        "Handler :: struct { run: (n: int) -> bool, int; other: int; }\n\
         make :: () -> Handler {\n\
             return Handler.{ run = (n: int) -> bool, int { return true, 1; }, | };\n\
         }",
    );
    assert!(has(&l, "other") && !has(&l, "run"), "{l:?}");
}
