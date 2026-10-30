//! Notes and help on calls whose polymorphic parameters cannot be inferred: where `$T` is
//! declared, which argument failed to bind it or bound it differently, and how to give the type
//! by name. See docs/compiler/polymorph-reporting.md.
use jaic::diagnostics::Diagnostic;
use jaic::source::Span;
use test_support::compiler::compile_source;

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

#[test]
fn a_literal_a_parameter_cannot_take_points_at_the_parameter_and_the_literal() {
    let (source, error) = failure("g :: (a: $T, count: int) {}\nmain :: () { g(1, .[1, 2]); }");
    assert!(
        error
            .message
            .contains("argument cannot be inferred as parameter type s64"),
        "{}",
        error.message
    );
    let notes = located(&source, &error);
    assert!(
        notes.iter().any(|(at, n)| at == "count: int"
            && n.contains("parameter `count` is declared here, with type `s64`")),
        "{notes:?}"
    );
    assert!(
        notes.iter().any(|(at, n)| at == ".[1, 2]"
            && n.contains("a `.[...]` literal, which has no type of its own")),
        "{notes:?}"
    );
    assert!(
        error
            .help
            .iter()
            .any(|h| h.contains("a value of type `s64`")),
        "{:?}",
        error.help
    );
}

#[test]
fn an_inferred_member_and_a_struct_literal_are_described_too() {
    let (source, error) = failure("g :: (name: string) {}\nmain :: () { g(.{1, 2}); }");
    let notes = located(&source, &error);
    assert!(
        notes.iter().any(|(at, n)| at == ".{1, 2}"
            && n.contains("a `.{...}` literal, which has no type of its own")),
        "{notes:?}"
    );
    let (source, error) = failure("g :: (x: float) {}\nmain :: () { g(.A); }");
    let notes = located(&source, &error);
    assert!(
        notes
            .iter()
            .any(|(at, n)| at == ".A" && n.contains("an inferred member")),
        "{notes:?}"
    );
    assert!(
        error
            .help
            .iter()
            .any(|h| h.contains("`float32` is not an enum")),
        "{:?}",
        error.help
    );
}

#[test]
fn arrays_of_different_lengths_name_the_length_each_has() {
    let (source, error) = failure(
        "same :: (a: [$N] int, b: [N] int) {}\n\
         main :: () { x: [3] int; y: [4] int; same(x, y); }",
    );
    assert!(error.message.contains("does not match parameter type"));
    let notes = located(&source, &error);
    assert!(
        notes
            .iter()
            .any(|(at, n)| at == "x" && n.contains("has length 3, so `$N` is 3 (parameter `a`)")),
        "{notes:?}"
    );
    assert!(
        notes
            .iter()
            .any(|(at, n)| at == "y" && n.contains("has length 4, but `N` is 3")),
        "{notes:?}"
    );
    assert!(
        error
            .help
            .iter()
            .any(|h| h.contains("arrays of the same length")),
        "{:?}",
        error.help
    );
}

#[test]
fn arrays_of_the_same_length_compile() {
    let (_, result) = compile_source(
        "same :: (a: [$N] int, b: [N] int) {}\n\
         main :: () { x: [3] int; y: [3] int; same(x, y); }",
    );
    assert!(result.is_ok());
}

#[test]
fn an_element_type_mismatch_gets_no_length_notes() {
    let (_, error) = failure(
        "same :: (a: [$N] int, b: [N] int) {}\n\
         main :: () { x: [3] int; y: [3] string; same(x, y); }",
    );
    assert!(error.notes.iter().all(|(_, n)| !n.contains("has length")));
}
