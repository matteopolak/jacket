//! Inlay hints and hovers inside polymorphic procedures nothing instantiates.
use jai_language_server::{Position, Range};
use test_support::lsp::{at_nth, open, uri};

const PROGRAM: &str = r"find :: (array: [] $T, item: T) -> bool, s64 {
    return false, -1;
}
plain :: (x: int) -> bool, s64 {
    return x > 0, x;
}
add_unique :: (array: *[..] $T, item: T) -> bool, s64 {
    found, index := find(array.*, item);
    ok, at := plain(3);
    return found, index;
}
same :: (v: $U) -> U { return v; }
wrap :: (v: $U) -> [] U { return .[]; }
ptr :: (v: $U) -> *U { return null; }
cb :: (v: $U) -> ((U, U) -> bool) { return null; }
first_match :: (a: [] $T, v: T) -> T { return v; }
rename :: (xs: [] $R, x: R) {
    renamed := first_match(xs, x);
}
forward :: (x: $T) {
    id := same(x);
    list := wrap(x);
    p := ptr(x);
    f := cb(x);
}
";

fn everything() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: PROGRAM.lines().count() as u32,
            character: 0,
        },
    }
}

#[test]
fn uninstantiated_polymorphic_body_hints_use_the_generic_signature() {
    let s = open(PROGRAM);
    let hints = s.inlay_hints(&uri(), everything()).unwrap();
    assert!(
        !hints.iter().any(|h| h.label.contains('?')),
        "no hint may be `?`: {hints:?}"
    );
    let at = |needle: &str, offset| at_nth(PROGRAM, needle, 0, offset);
    let label_at = |p| {
        hints
            .iter()
            .find(|h| h.position == p)
            .map(|h| h.label.clone())
    };
    assert_eq!(label_at(at("found,", 5)).as_deref(), Some(": bool"));
    assert_eq!(label_at(at("index :=", 5)).as_deref(), Some(": s64"));
    assert_eq!(label_at(at("ok,", 2)).as_deref(), Some(": bool"));
    assert_eq!(label_at(at("at :=", 2)).as_deref(), Some(": s64"));
    assert_eq!(label_at(at("id :=", 2)).as_deref(), Some(": T"));
    assert_eq!(label_at(at("list :=", 4)).as_deref(), Some(": [] T"));
    assert_eq!(label_at(at("p :=", 1)).as_deref(), Some(": *T"));
    assert_eq!(
        label_at(at("    f := cb", 5)).as_deref(),
        Some(": (T, T) -> bool")
    );
    assert_eq!(label_at(at("renamed :=", 7)).as_deref(), Some(": R"));
    for (marker, want) in [("found,", "bool"), ("index :=", "s64"), ("id :=", "T")] {
        let hover = s
            .hover(&uri(), at_nth(PROGRAM, marker, 0, 0))
            .unwrap()
            .expect("hover")
            .contents
            .value;
        assert!(hover.contains(want) && !hover.contains('?'), "{hover}");
    }
}

#[test]
fn an_unparenthesised_procedure_type_result_still_gets_a_hint() {
    const TEXT: &str = r"cmp :: (a: int, b: int) -> bool { return a < b; }
pick :: () -> (int, int) -> bool { return cmp; }
main :: () {
    f := pick();
}
";
    let s = open(TEXT);
    let all = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 5,
            character: 0,
        },
    };
    let hints = s.inlay_hints(&uri(), all).unwrap();
    let at = at_nth(TEXT, "f :=", 0, 1);
    let label = hints
        .iter()
        .find(|h| h.position == at)
        .map(|h| h.label.clone());
    assert_eq!(label.as_deref(), Some(": (s64, s64) -> bool"), "{hints:?}");
    // The compiler rejects the form; the editor still says so.
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.message.contains("must be parenthesised")),
        "{diagnostics:?}"
    );
}
