//! Original callable-provenance fixtures. Structural type equality is never a name oracle.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Session};

fn names(source: &str) -> Vec<String> {
    let byte = source.find('|').unwrap();
    let text = source.replacen('|', "", 1);
    let mut s = Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(|_| {
                let mut o = jaic::sema::Options::host();
                o.runtime_support = false;
                o.stack_trace = false;
                o
            }),
        },
    );
    let uri = DocumentUri::parse("file:///callback-arguments/main.jai").unwrap();
    s.open(uri.clone(), 1, text.clone()).unwrap();
    let prefix = &text[..byte];
    let at = Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    };
    s.completion(&uri, at)
        .unwrap()
        .items
        .into_iter()
        .filter(|i| i.detail.starts_with("parameter "))
        .map(|i| i.label)
        .collect()
}

#[test]
fn same_shaped_callback_fields_keep_independent_headers() {
    let declarations = "Holder :: struct { callback: (public_option := false) -> int; }\nOther :: struct { callback: (private_option := true) -> int; }\nmain :: () { h: Holder; other: Other; ";
    assert_eq!(
        names(&format!("{declarations}h.callback(|); }}")),
        ["public_option"]
    );
    assert_eq!(
        names(&format!("{declarations}other.callback(|); }}")),
        ["private_option"]
    );
    assert!(names(&format!("{declarations}h.callback(private_|); }}")).is_empty());
}

#[test]
fn declared_callback_header_wins_over_initializer_and_same_type_alias() {
    assert_eq!(
        names(
            "implementation :: (implementation_name: bool) -> int { return 1; }\nmain :: () { value: (declared_name: bool) -> int = implementation; value(|); }"
        ),
        ["declared_name"]
    );
    assert_eq!(
        names(
            "implementation :: (implementation_name: bool) -> int { return 1; }\nmain :: () { value: (declared_name: bool) -> int = implementation; alias := value; alias(|); }"
        ),
        ["declared_name"]
    );
    assert_eq!(
        names("main :: (callback: (argument: int, option := false) -> int) { callback(1, op|); }"),
        ["option"]
    );
    assert_eq!(
        names(
            "implementation :: (implementation_name: bool) -> int { return 1; }\nmain :: () { value: (declared_name: bool) -> int = implementation; value(declared_na|me = true); }"
        ),
        ["declared_name"]
    );
}

#[test]
fn grouped_nested_and_unclosed_callback_calls_use_actual_callee() {
    let prefix = "Holder :: struct { callback: (argument: int, option := false) -> int; }\nmain :: () { h: Holder; ";
    assert_eq!(
        names(&format!("{prefix}(h.callback)(1, op|); }}")),
        ["option"]
    );
    assert_eq!(
        names(&format!("{prefix}h.callback(h.callback(1, op|")),
        ["option"]
    );
    assert_eq!(
        names(&format!("{prefix}h.callback(1, op|\n}}\nhealthy :: 1;")),
        ["option"]
    );
}

#[test]
fn callback_slots_filter_both_sides_and_positional_arguments() {
    let prefix = "main :: (callback: (first: int, second := false, third := true) -> int) { ";
    assert_eq!(
        names(&format!("{prefix}callback(1, |); }}")),
        ["second", "third"]
    );
    assert_eq!(
        names(&format!(
            "{prefix}callback(first = 1, se|, third = true); }}"
        )),
        ["second"]
    );
    assert!(
        names(&format!(
            "{prefix}callback(1, second = false, third = true, |); }}"
        ))
        .is_empty()
    );
}

#[test]
fn pointer_promoted_and_generic_callback_fields_use_selected_member() {
    assert_eq!(
        names(
            "Holder :: struct { callback: (selected_option := false) -> int; }\nOuter :: struct { using inner: Holder; }\nmain :: () { p: *Outer; p.callback(sel|); }"
        ),
        ["selected_option"]
    );
    assert_eq!(
        names(
            "Holder :: struct (T: Type) { callback: (value: T, selected_option := false) -> T; }\nmain :: () { h: Holder(int); h.callback(1, sel|); }"
        ),
        ["selected_option"]
    );
}

#[test]
fn factory_result_retains_checked_header_without_borrowing_a_type_match() {
    assert_eq!(
        names(
            "implementation :: (implementation_name: bool) -> int { return 1; }\nCallback :: (returned_name: bool) -> int;\nfactory :: () -> Callback { return implementation; }\nmain :: () { factory()(ret|); }"
        ),
        ["returned_name"]
    );
}

#[test]
fn erased_array_element_and_unresolved_headers_do_not_borrow_names() {
    assert!(names("Other :: struct { callback: (borrowed_name: bool) -> int; }\nmain :: () { other: Other; callbacks: [2](erased_name: bool) -> int; callbacks[0](|); }").is_empty());
    assert!(names("Holder :: struct { callback: (unknown_name: Missing_Type) -> int; }\nmain :: () { h: Holder; h.callback(unk|); }").is_empty());
}

#[test]
fn variadic_callback_uses_declared_runtime_slots() {
    assert_eq!(
        names(
            "main :: (callback: (first: int, rest: ..int, enabled := false) -> int) { callback(1, 2, en|); }"
        ),
        ["enabled"]
    );
}
