//! Notes and help on calls whose polymorphic parameters cannot be inferred: where `$T` is
//! declared, which argument failed to bind it or bound it differently, and how to give the type
//! by name. See docs/compiler/polymorph-reporting.md.
mod common;
use common::compile_source;
use jaic::source::{Diagnostic, Span};

/// Compile `source`, which must fail, and return its source text and the error.
fn failure(source: &str) -> (String, Box<Diagnostic>) {
    let (_, result) = compile_source(source);
    match result {
        Ok(()) => panic!("expected an error in:\n{source}"),
        Err(error) => (source.to_string(), error),
    }
}

/// The source text a located note points at, paired with the note.
fn located(source: &str, d: &Diagnostic) -> Vec<(String, String)> {
    d.notes
        .iter()
        .filter(|(span, _)| *span != Span::NONE)
        .map(|(span, note)| {
            (
                source[span.start as usize..span.end as usize].to_string(),
                note.clone(),
            )
        })
        .collect()
}

#[test]
fn unbound_variable_points_at_its_declaration() {
    let (source, error) = failure("none :: (n: int, rest: ..$T) {}\nmain :: () { none(1); }");
    assert!(
        error
            .message
            .contains("could not infer polymorphic type `$T`")
    );
    let notes = located(&source, &error);
    assert!(
        notes.iter().any(|(at, note)| at == "$T"
            && note.contains("`$T` is declared here, in the type of `rest`")),
        "{notes:?}"
    );
    // No argument was passed for `rest`, so none is blamed.
    assert!(
        !notes.iter().any(|(_, n)| n.contains("argument")),
        "{notes:?}"
    );
    assert!(
        error.help.iter().any(|h| h.contains("`T = s64`")),
        "{:?}",
        error.help
    );
}

#[test]
fn unbound_variable_names_the_argument_that_gave_nothing() {
    let (source, error) =
        failure("k :: (xs: [$N] $T) -> int { return N; }\nmain :: () { k(.[1, 2, 3]); }");
    let notes = located(&source, &error);
    assert!(
        notes
            .iter()
            .any(|(at, note)| at == ".[1, 2, 3]"
                && note.contains("gave nothing for `$T` in `[$N] $T`")),
        "{notes:?}"
    );
}

#[test]
fn naming_the_type_makes_the_suggested_call_compile() {
    let (_, result) = compile_source(
        "none :: (n: int, rest: ..$T) {}\nk :: (xs: [$N] $T) -> int { return N; }\n\
         main :: () { none(1, T = s64); k(.[1, 2, 3], T = s64); }",
    );
    assert!(result.is_ok());
}

#[test]
fn conflicting_arguments_each_get_a_note_with_their_type() {
    let (source, error) = failure(
        "Box :: struct($T: Type) { v: T; }\nboth :: (a: $T, b: Box($T)) {}\n\
         main :: () { s: Box(string); both(1, s); }",
    );
    assert!(error.message.contains("conflicting types for `$T`"));
    let notes = located(&source, &error);
    assert!(
        notes
            .iter()
            .any(|(at, n)| at == "1" && n.contains("comes from this argument, of type `s64`")),
        "{notes:?}"
    );
    assert!(
        notes.iter().any(|(at, n)| at == "s"
            && n.contains("of type `Box(string)`, gives a different `$T`")),
        "{notes:?}"
    );
    assert!(
        error.help.iter().any(|h| h.contains("cast(")),
        "{:?}",
        error.help
    );
}

#[test]
fn casting_an_argument_resolves_a_conflict() {
    let (_, result) = compile_source(
        "f :: (a: $T, b: T) -> T { return a; }\n\
         main :: () { y: float = 1; z: s64 = 2; w := f(y, cast(float) z); }",
    );
    assert!(result.is_ok());
}

#[test]
fn slice_pattern_mismatch_names_the_argument_type_and_pattern() {
    let (source, error) = failure("v :: (a: [] $T) {}\nmain :: () { x: int = 3; v(x); }");
    let notes = located(&source, &error);
    assert!(
        notes.iter().any(|(at, n)| at == "x"
            && n.contains("argument has type `s64`, which does not match `[] $T`")),
        "{notes:?}"
    );
}

#[test]
fn array_pattern_mismatch_names_the_argument_type_and_pattern() {
    let (source, error) = failure("r :: (a: [..] $T) {}\nmain :: () { arr: [3] int; r(arr); }");
    let notes = located(&source, &error);
    assert!(
        notes
            .iter()
            .any(|(at, n)| at == "arr" && n.contains("which does not match `[..] $T`")),
        "{notes:?}"
    );
}
