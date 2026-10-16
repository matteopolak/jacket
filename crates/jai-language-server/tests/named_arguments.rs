//! Original callable fixtures: parameter names come from compiler metadata, not library text.
use jai_language_server::{
    CompletionItem, DocumentUri, Environment, Limits, Position, Range, Session, TextChange,
};

fn session() -> Session {
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(|_| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    )
}

fn complete(source: &str) -> Vec<CompletionItem> {
    let byte = source.find('|').unwrap();
    let text = source.replacen('|', "", 1);
    let mut session = session();
    let uri = DocumentUri::parse("file:///named-arguments/main.jai").unwrap();
    session.open(uri.clone(), 1, text.clone()).unwrap();
    session
        .completion(&uri, position(&text, byte))
        .unwrap()
        .items
}

#[test]
fn variadic_spread_defaults_and_generic_headers_use_actual_slots() {
    let decl = "f :: (first: int, rest: ..int, enabled := false) {}\nmain :: () { ";
    assert_eq!(names(&format!("{decl}f(1, 2, 3, |); }}")), ["enabled"]);
    assert_eq!(names(&format!("{decl}f(1, ..values, |); }}")), ["enabled"]);
    assert_eq!(
        names("f :: (value: $T, enabled := false) {}\nmain :: () { f(1, en|); }"),
        ["enabled"]
    );
    assert!(names("Holder :: struct { callback: (option: bool) -> int; }\nmain :: () { holder: Holder; holder.callback(op|); }").is_empty(), "no name metadata: do not invent a signature");
}

#[test]
fn resolved_overload_wins_and_member_completion_is_preserved() {
    assert_eq!(
        names(
            "f :: (value: int, shared := false, integer_only := false) {}\nf :: (value: string, shared := false, string_only := false) {}\nmain :: () { f(1, |shared = true); }"
        ),
        ["integer_only", "shared"]
    );
    assert_eq!(
        names(
            "f :: (value: int, shared := false, integer_only := false) {}\nf :: (value: string, shared := false, string_only := false) {}\nmain :: () { f(1, sh|ared = true); }"
        ),
        ["shared"]
    );
    let items = complete(
        "Record :: struct { field: int; }\nf :: (option := false) {}\nmain :: () { r: Record; f(r.fi|); }",
    );
    assert!(items.iter().any(|i| i.label == "field"));
    assert!(!items.iter().any(|i| i.detail.starts_with("parameter ")));
}

#[test]
fn known_argument_types_filter_unselected_overloads() {
    assert_eq!(
        names("f :: (byte: u8, enabled := false) {}\nmain :: () { f(\"x\", en|); }"),
        ["enabled"]
    );
    assert_eq!(
        names(
            "Record :: struct { value: int; }\nf :: (record: Record, enabled := false) {}\nmain :: () { r: Record; f(*r, en|); }"
        ),
        ["enabled"]
    );
    assert_eq!(
        names("f :: (code: Code, enabled := false) #expand {}\nmain :: () { f(1, en|); }"),
        ["enabled"]
    );
    let declarations = "f :: (value: int, option_integer := false) {}\nf :: (value: string, option_string := false) {}\nmain :: () { ";
    assert_eq!(
        names(&format!("{declarations}f(1, opt|); }}")),
        ["option_integer"]
    );
    assert_eq!(
        names(&format!("{declarations}f(\"text\", opt|); }}")),
        ["option_string"]
    );
    assert!(names("f :: (option: Missing_Type) {}\nmain :: () { f(op|); }").is_empty());
}

#[test]
fn independent_callback_signatures_do_not_leak_other_parameter_names() {
    // Shared structural-type metadata cannot prove either declaration's callable signature.
    // Preserve the no-fabrication assertion until compiler ownership binds names to values.
    assert!(names("Holder :: struct { callback: (public_option := false) -> int; }\nOther :: struct { callback: (private_option := false) -> int; }\nmain :: () { h: Holder; other: Other; h.callback(|); }").is_empty());
}

#[test]
fn utf16_incremental_edits_recompute_the_callee_without_stale_names() {
    let mut s = session();
    let uri = DocumentUri::parse("file:///named-arguments/main.jai").unwrap();
    let mut text = "f :: (first := false) {}\ng :: (other := false) {}\nmain :: () { unicode := \"😀\"; f( ); }".to_owned();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    let callee = text.rfind("f(").unwrap();
    let byte = callee + 2;
    let get = |s: &Session, text: &str| {
        s.completion(&uri, position(text, byte))
            .unwrap()
            .items
            .into_iter()
            .filter(|i| i.detail.starts_with("parameter "))
            .map(|i| i.label)
            .collect::<Vec<_>>()
    };
    assert_eq!(get(&s, &text), ["first"]);
    for (version, replacement, expected) in [(2, "g", "other"), (3, "f", "first")] {
        s.change(
            &uri,
            version,
            &[TextChange {
                range: Some(Range {
                    start: position(&text, callee),
                    end: position(&text, callee + 1),
                }),
                range_length: Some(1),
                text: replacement.into(),
            }],
        )
        .unwrap();
        text.replace_range(callee..callee + 1, replacement);
        assert_eq!(get(&s, &text), [expected]);
    }
}

fn position(text: &str, byte: usize) -> Position {
    Position {
        line: text[..byte].matches('\n').count() as u32,
        character: text[..byte]
            .rsplit('\n')
            .next()
            .unwrap()
            .encode_utf16()
            .count() as u32,
    }
}

fn names(source: &str) -> Vec<String> {
    complete(source)
        .into_iter()
        .filter(|i| i.detail.starts_with("parameter "))
        .map(|i| i.label)
        .collect()
}

#[test]
fn incomplete_call_offers_remaining_defaults_and_preserves_expressions() {
    let collisions =
        complete("f :: (enabled := false) {}\nmain :: () { enabled := true; f(en|); }");
    assert_eq!(
        collisions.iter().filter(|i| i.label == "enabled").count(),
        2
    );
    assert_eq!(
        collisions
            .iter()
            .filter(|i| i.label == "enabled" && i.detail.starts_with("parameter "))
            .count(),
        1
    );
    let items = complete(
        "emit :: (message: string, enabled := false) {}\nmain :: () { enabled_local := true; emit(\"x\", en|\n}",
    );
    let enabled = items
        .iter()
        .find(|i| i.label == "enabled")
        .expect("named parameter");
    assert_eq!(enabled.insert_text.as_deref(), Some("enabled = "));
    assert!(items.iter().any(|i| i.label == "enabled_local"));
}

#[test]
fn supplied_arguments_filter_each_overload_and_parameters_after_cursor() {
    let decl = "f :: (first: int, second := 2, third := 3) {}\n";
    assert_eq!(
        names(&format!("{decl}main :: () {{ f(1, |); }}")),
        ["second", "third"]
    );
    assert_eq!(
        names(&format!("{decl}main :: () {{ f(|, second = 2); }}")),
        ["first", "third"]
    );
    assert_eq!(
        names(&format!("{decl}main :: () {{ f(second = 2, 3, |); }}")),
        ["first"]
    );
    assert!(names(&format!("{decl}main :: () {{ f(unknown = 1, |); }}")).is_empty());
    let overloads = "f :: (first: int, integer_option := false) {}\nf :: (first: string, text_option := false) {}\nmain :: () { f(|); }";
    assert_eq!(names(overloads), ["first", "integer_option", "text_option"]);
}

#[test]
fn nested_calls_methods_aliases_and_unresolved_callback_fields() {
    let source = "Tools :: struct { emit :: (amount: int, enabled := false) {} }\nalias :: Tools.emit;\nHolder :: struct { callback: (value: int, option := false) -> int; }\nmain :: () { holder: Holder; ";
    for call in ["Tools.emit(en|)", "alias(en|)"] {
        assert_eq!(names(&format!("{source}{call}; }}")), ["enabled"]);
    }
    assert!(names(&format!("{source}holder.callback(op|); }}")).is_empty());
    assert_eq!(
        names(
            "f :: (outer: int) {}\ng :: (inner: int) -> int { return inner; }\nmain :: () { f(g(in|)); }"
        ),
        ["inner"]
    );
}

#[test]
fn nested_incomplete_calls_resolve_from_the_original_scope() {
    assert_eq!(
        names(
            "outer :: (value: int) {}\nTools :: struct { emit :: (enabled := false) -> int { return 1; } }\nmain :: () { outer(Tools.emit(en|\n}"
        ),
        ["enabled"]
    );
    assert_eq!(
        names(
            "emit :: (text: string, enabled := false) {}\nmain :: () {\n    emit(\"x\", en|\n    healthy := 42;\n}"
        ),
        ["enabled"]
    );
    assert!(names("emit :: (wrong_option := false) -> int { return 1; }\nHolder :: struct { emit: (own_option := false) -> int; }\nmain :: () { h: Holder; (h).emit(wr|\n}").is_empty(),"a complex receiver must never be truncated to an unrelated global name");
    assert_eq!(
        names(
            "outer :: (value: int) {}\ninner :: (inner_option := false) -> int { return 1; }\nmain :: () { outer(inner(in|\n}"
        ),
        ["inner_option"]
    );
    assert_eq!(
        names(
            "Tools :: struct { emit :: (enabled := false) -> int { return 1; } }\nmain :: () { Tools.emit(en|\n}"
        ),
        ["enabled"]
    );
}

#[test]
fn values_members_comments_strings_and_unresolved_callees_do_not_invent_names() {
    let decl = "f :: (first: string, enabled := false) {}\nmain :: () { ";
    for call in [
        "f(first = en|)",
        "f(\"en|\")",
        "f(/* en| */)",
        "f(thing.en|)",
        "missing(en|)",
        "f((en|))",
        "f(1,, en|)",
    ] {
        assert!(names(&format!("{decl}{call}; }}")).is_empty(), "{call}");
    }
    assert_eq!(
        names(&format!("{decl}f(\"comma, )\", /* comma, ( */ en|); }}")),
        ["enabled"]
    );
}

#[test]
fn insertion_replaces_whole_utf16_word_and_preserves_or_normalizes_separator() {
    for (tail, expected) in [
        ("en|abled", "enabled = "),
        ("en|abled = true", "enabled"),
        ("en|abled: true", "enabled = "),
        ("en|abled:= true", "enabled = "),
    ] {
        let source =
            format!("f :: (enabled := false) {{}}\nmain :: () {{ s := \"😀\"; f({tail}); }}");
        let item = complete(&source)
            .into_iter()
            .find(|i| i.label == "enabled")
            .unwrap();
        assert_eq!(item.insert_text.as_deref(), Some(expected));
        let text = source.replace('|', "");
        let start = text.find("f(en").unwrap() + 2;
        assert_eq!(item.replace.unwrap().start, position(&text, start));
    }
}
