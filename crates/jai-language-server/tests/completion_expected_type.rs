//! Completion ranks values whose type fits the expected type before the others.
use jai_language_server::CompletionItem;
use test_support::lsp::{cursor, file_uri, lean_session};

fn sorted(source: &str) -> Vec<String> {
    let (text, at) = cursor(source);
    let mut session = lean_session();
    let uri = file_uri("/expected-type/main.jai");
    session.open(uri.clone(), 1, text).unwrap();
    let mut items: Vec<CompletionItem> = session.completion(&uri, at).unwrap().items;
    items.sort_by(|a, b| {
        let key = |i: &CompletionItem| i.sort_text.clone().unwrap_or_else(|| i.label.clone());
        key(a).cmp(&key(b))
    });
    items.into_iter().map(|i| i.label).collect()
}

fn before(order: &[String], first: &str, second: &str) -> bool {
    let at = |n: &str| order.iter().position(|l| l == n).unwrap();
    at(first) < at(second)
}

const LOCALS: &str = "a_name: string = \"\"; z_flag := true;";

#[test]
fn a_bool_parameter_ranks_a_bool_local_first() {
    let order = sorted(&format!(
        "f :: (on: bool) {{}}\nmain :: () {{ {LOCALS} f(|); }}"
    ));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
}

#[test]
fn a_named_argument_ranks_by_its_parameter() {
    let order = sorted(&format!(
        "f :: (text: string, on: bool) {{}}\nmain :: () {{ {LOCALS} f(\"x\", on = |); }}"
    ));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
    let order = sorted(
        "f :: (text: string, on: bool) {}\nmain :: () { a_flag := true; z_name: string = \"\"; f(text = |); }",
    );
    assert!(before(&order, "z_name", "a_flag"), "{order:?}");
}

#[test]
fn a_typed_declaration_ranks_by_its_type() {
    let order = sorted(&format!("main :: () {{ {LOCALS} x: bool = |; }}"));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
}

#[test]
fn a_return_value_and_a_condition_rank_by_their_types() {
    let order = sorted(&format!("g :: () -> bool {{ {LOCALS} return |; }}"));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
    let order = sorted(&format!("main :: () {{ {LOCALS} if | {{}} }}"));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
    let order = sorted(&format!("main :: () {{ {LOCALS} while | {{}} }}"));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
}

#[test]
fn an_assignment_ranks_by_the_target_type() {
    let order = sorted(&format!("main :: () {{ {LOCALS} v := false; v = |; }}"));
    assert!(before(&order, "z_flag", "a_name"), "{order:?}");
}

#[test]
fn a_convertible_type_fits() {
    let order = sorted("main :: () { a_text: string = \"\"; z_small: s8 = 1; x: s64 = |; }");
    assert!(before(&order, "z_small", "a_text"), "{order:?}");
}

#[test]
fn no_expected_type_keeps_the_usual_order() {
    let order = sorted(&format!("main :: () {{ {LOCALS} |; }}"));
    assert!(before(&order, "a_name", "z_flag"), "{order:?}");
}

#[test]
fn nothing_is_hidden() {
    let order = sorted(&format!(
        "f :: (on: bool) {{}}\nmain :: () {{ {LOCALS} f(|); }}"
    ));
    assert!(order.contains(&"a_name".to_string()));
}

#[test]
fn an_enum_value_for_a_baked_parameter_lists_the_enum() {
    let order = sorted(
        "Mode :: enum { slow; fast; }\nspeed :: ($mode: Mode = .fast) -> int { return 1; }\nmain :: () { x := speed(.|slow) == 1; }",
    );
    assert!(order.contains(&"slow".to_string()), "{order:?}");
    assert!(order.contains(&"fast".to_string()), "{order:?}");
}

mod deferred {
    use super::*;

    const SOURCE: &str = "main :: () { a_name: string = \"\"; z_flag := true; x: bool = |; }";

    fn order(mut items: Vec<CompletionItem>) -> Vec<String> {
        items.sort_by(|a, b| {
            let key = |i: &CompletionItem| i.sort_text.clone().unwrap_or_else(|| i.label.clone());
            key(a).cmp(&key(b))
        });
        items.into_iter().map(|i| i.label).collect()
    }

    #[test]
    fn a_request_does_not_compile_for_ranking_and_the_next_one_is_ranked() {
        let (text, at) = cursor(SOURCE);
        let mut session = lean_session();
        session.defer_compiles(true);
        let uri = file_uri("/expected-type/deferred.jai");
        session.open(uri.clone(), 1, text).unwrap();
        // The diagnostics compile the program; the names come from it.
        session.publications();
        let compiles = session.compile_counts().0;
        let first = session.completion(&uri, at).unwrap();
        assert_eq!(
            session.compile_counts().0,
            compiles,
            "no compile in the request"
        );
        assert!(first.is_incomplete);
        assert!(session.compile_pending());
        let unranked = order(first.items);
        assert!(before(&unranked, "a_name", "z_flag"), "{unranked:?}");
        session.warm_pending();
        assert!(!session.compile_pending());
        let compiles = session.compile_counts().0;
        let second = session.completion(&uri, at).unwrap();
        assert_eq!(
            session.compile_counts().0,
            compiles,
            "ranked from the cache"
        );
        let order = order(second.items);
        assert!(before(&order, "z_flag", "a_name"), "{order:?}");
    }

    #[test]
    fn without_deferring_the_first_request_is_ranked() {
        let (text, at) = cursor(SOURCE);
        let mut session = lean_session();
        let uri = file_uri("/expected-type/immediate.jai");
        session.open(uri.clone(), 1, text).unwrap();
        let answer = session.completion(&uri, at).unwrap();
        let order = order(answer.items);
        assert!(before(&order, "z_flag", "a_name"), "{order:?}");
    }
}
