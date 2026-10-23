//! Notes, help and fixes on errors in calls, declarations and assignments: the callee's
//! signature, where a name was first declared, how to assign to a constant or an argument,
//! what a struct literal is missing, and the counts of names and values. See
//! docs/compiler/diagnostics.md.
mod common;
use common::compile_source;
use jaic::sema::Compiler;
use jaic::source::{Applicability, Diagnostic, Span};

/// Compile `source`, which must fail, and return the compiler and the error.
fn failure(source: &str) -> (Compiler, Box<Diagnostic>) {
    let (c, result) = compile_source(source);
    match result {
        Ok(()) => panic!("expected an error in:\n{source}"),
        Err(error) => (c, error),
    }
}

/// The plain rendering of the error of `source`.
fn rendered(source: &str) -> String {
    let (c, error) = failure(source);
    error.render(&c.src.sources)
}

/// The text of the notes that have no place in the source.
fn text_notes(d: &Diagnostic) -> Vec<&str> {
    d.notes
        .iter()
        .filter(|(span, _)| *span == Span::NONE)
        .map(|(_, note)| note.as_str())
        .collect()
}

/// The text of the source after applying the edits of suggestion `which` of `d`.
fn apply(c: &Compiler, d: &Diagnostic, which: usize) -> String {
    let mut text = c.src.sources.get(d.span.file).text.to_string();
    let mut edits: Vec<_> = d.suggestions()[which].replacements().collect();
    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    for (span, new) in edits {
        text.replace_range(span.start as usize..span.end as usize, new);
    }
    text
}

/// Apply the first suggestion of the error of `source` and check the result compiles.
#[track_caller]
fn fixed(source: &str) -> String {
    let (c, error) = failure(source);
    let patched = apply(&c, &error, 0);
    let (c2, result) = compile_source(&patched);
    if let Err(e) = result {
        panic!(
            "the fix left an error:\n{patched}\n{}",
            e.render(&c2.src.sources)
        );
    }
    patched
}

const SIGNATURE: &str = "f :: (a: int, b: string) { }\n";

#[test]
fn too_many_arguments_note_the_signature_and_offer_to_remove_them() {
    let source = format!("{SIGNATURE}main :: () {{ f(1, \"x\", 3); }}");
    let (c, error) = failure(&source);
    assert!(
        error.message.contains("too many arguments"),
        "{}",
        error.message
    );
    assert!(
        text_notes(&error).contains(&"`f` takes `(a: int, b: string)`"),
        "{:?}",
        error.notes
    );
    let suggestion = &error.suggestions()[0];
    assert_eq!(suggestion.title, "remove the extra argument");
    // Deleting an argument changes what the call means, so it is a guess.
    assert_eq!(suggestion.applicability, Applicability::MaybeIncorrect);
    assert!(apply(&c, &error, 0).contains("f(1, \"x\"); }"));
    fixed(&source);
}

#[test]
fn several_extra_arguments_are_removed_together() {
    let source = "f :: (a: int) { }\nmain :: () { f(1, 2, 3 + 4, 5); }";
    let (c, error) = failure(source);
    assert_eq!(error.suggestions()[0].title, "remove the extra arguments");
    assert!(apply(&c, &error, 0).contains("f(1); }"));
    fixed(source);
}

#[test]
fn extra_arguments_of_a_procedure_without_parameters_are_removed() {
    let source = "f :: () { }\nmain :: () { f(1, 2); }";
    let (c, error) = failure(source);
    assert!(apply(&c, &error, 0).contains("f(); }"));
    fixed(source);
}

#[test]
fn a_named_argument_after_the_extra_ones_is_not_removed() {
    let source = "f :: (a: int, b := 2) { }\nmain :: () { f(1, 2, 3, b = 4); }";
    let (_, error) = failure(source);
    assert!(error.suggestions().is_empty(), "{:?}", error.suggestions());
}

#[test]
fn a_missing_argument_notes_the_signature() {
    let source = format!("{SIGNATURE}main :: () {{ f(1); }}");
    let (_, error) = failure(&source);
    assert!(error.message.contains("missing argument for parameter `b`"));
    assert!(text_notes(&error).contains(&"`f` takes `(a: int, b: string)`"));
    // The located note still points at the declaration.
    assert!(error.notes.iter().any(|(_, n)| n == "`f` is declared here"));
}

#[test]
fn a_call_through_a_procedure_value_notes_its_type() {
    let head = "main :: () { g: (int, int) -> int = (x: int, y: int) -> int { return x; }; ";
    let few = format!("{head}g(1); }}");
    let (_, error) = failure(&few);
    assert!(
        error.message.contains("missing argument"),
        "{}",
        error.message
    );
    assert!(text_notes(&error).contains(&"the procedure has type `(s64, s64) -> s64`"));
    let many = format!("{head}g(1, 2, 3); }}");
    let (c, error) = failure(&many);
    assert!(
        error.message.contains("too many arguments"),
        "{}",
        error.message
    );
    assert!(text_notes(&error).contains(&"the procedure has type `(s64, s64) -> s64`"));
    assert!(apply(&c, &error, 0).contains("g(1, 2); }"));
    fixed(&many);
}

#[test]
fn a_redeclared_member_points_at_the_first() {
    let out = rendered("S :: struct { a: int; a: int; } main :: () { }");
    assert!(
        out.contains("member `a` is declared more than once"),
        "{out}"
    );
    assert!(out.contains("note: `a` is first declared here"), "{out}");
    let out = rendered("E :: enum { A; B; A; } main :: () { }");
    assert!(
        out.contains("enum member `A` is declared more than once"),
        "{out}"
    );
    assert!(out.contains("note: `A` is first declared here"), "{out}");
    let out = rendered("f :: (a: int, a: int) { } main :: () { }");
    assert!(
        out.contains("`a` is already declared in this scope"),
        "{out}"
    );
    assert!(out.contains("note: `a` is first declared here"), "{out}");
}

#[test]
fn a_declaration_reusing_a_member_from_using_names_the_variable() {
    let source = "Outer :: struct { x: s32; y: s32; }\n\
                  main :: () {\n    o: Outer;\n    using o;\n    x := 100;\n}\n";
    let (_, error) = failure(source);
    assert_eq!(error.message, "`x` is already declared in this scope");
    let (note_span, note) = &error.notes[0];
    assert_eq!(
        note,
        "`x` is a member of `o`, which `using` brings into scope"
    );
    assert_eq!(
        &source[note_span.start as usize..note_span.end as usize],
        "o"
    );
    assert_eq!(
        error.help,
        ["use another name, or write `o.x` for the member"]
    );
    // The constant declared before the `using` is the one reported, with the same advice.
    let constant = source.replace("x := 100;", "x :: 5;");
    let (_, error) = failure(&constant);
    assert_eq!(error.message, "`x` is already declared in this scope");
    assert_eq!(error.notes.len(), 1);
}

#[test]
fn a_member_two_using_variables_provide_is_ambiguous() {
    let source = "Outer :: struct { x: s32; y: s32; }\n\
                  main :: () {\n    o: Outer;\n    p: Outer;\n    using o;\n    using p;\n    x = 1;\n}\n";
    let (_, error) = failure(source);
    assert!(
        error.message.starts_with("`x` is ambiguous"),
        "{}",
        error.message
    );
    assert_eq!(error.notes.len(), 2);
    assert_eq!(error.notes[0].1, "`o` has a member `x`");
    assert_eq!(error.notes[1].1, "`p` has a member `x`");
    assert_eq!(error.help, ["say which one you mean: `o.x` or `p.x`"]);
}

#[test]
fn a_constant_is_assigned_after_declaring_it_with_a_variable_marker() {
    let source = "X :: 5;\nmain :: () {\n    X = 6;\n}\n";
    let (c, error) = failure(source);
    assert_eq!(error.message, "cannot assign to constant `X`");
    assert!(
        error
            .notes
            .iter()
            .any(|(span, n)| n == "`X` is declared as a constant here"
                && c.src.sources.snippet(*span) == "X"),
        "{:?}",
        error.notes
    );
    let suggestion = &error.suggestions()[0];
    assert_eq!(
        suggestion.title,
        "declare it with `:=` to make it a variable"
    );
    assert_eq!(suggestion.applicability, Applicability::MaybeIncorrect);
    assert_eq!(
        apply(&c, &error, 0),
        "X := 5;\nmain :: () {\n    X = 6;\n}\n"
    );
    fixed(source);
}

#[test]
fn a_local_constant_gets_the_same_help() {
    let source = "main :: () {\n    K :: 3;\n    K += 1;\n}\n";
    let (c, error) = failure(source);
    assert_eq!(error.message, "cannot assign to constant `K`");
    assert!(apply(&c, &error, 0).contains("K := 3;"));
    fixed(source);
}

#[test]
fn a_typed_constant_has_help_but_no_edit() {
    let (_, error) = failure("K : int : 5; main :: () { K = 6; }");
    assert!(error.suggestions().is_empty());
    assert_eq!(error.help, ["declare it with `:=` to make it a variable"]);
}

#[test]
fn writing_into_a_constant_array_notes_the_declaration() {
    let source = "T :: int.[1, 2];\nmain :: () { T[0] = 5; }";
    let (c, error) = failure(source);
    assert_eq!(error.message, "cannot assign into the constant `T`");
    assert!(
        error
            .notes
            .iter()
            .any(|(_, n)| n.contains("declared as a constant"))
    );
    assert!(apply(&c, &error, 0).starts_with("T := int.[1, 2];"));
    fixed(source);
}

#[test]
fn an_immutable_argument_is_copied_into_a_local_first() {
    let source = "S :: struct { a: int; b: int; c: int; }\n\
                  f :: (s: S) {\n    s.a = 3;\n}\nmain :: () { }\n";
    let (c, error) = failure(source);
    assert_eq!(error.message, "Can't assign to an immutable argument.");
    assert!(error.notes.iter().any(|(span, n)| {
        n == "`s` is an argument, which the procedure may only read"
            && c.src.sources.snippet(*span).starts_with('s')
    }));
    let suggestion = &error.suggestions()[0];
    assert_eq!(suggestion.title, "copy it into a local first: `s := s;`");
    assert_eq!(suggestion.applicability, Applicability::MaybeIncorrect);
    assert!(apply(&c, &error, 0).contains("f :: (s: S) { s := s;\n    s.a = 3;"));
    fixed(source);
}

#[test]
fn replacing_a_whole_immutable_argument_gets_the_same_fix() {
    let source = "S :: struct { a: int; b: int; c: int; }\n\
                  f :: (s: S) {\n    s = .{};\n}\nmain :: () { }\n";
    fixed(source);
}

#[test]
fn a_name_that_reaches_an_argument_through_using_only_gets_help() {
    let source = "Outer :: struct { x: s32; y: s32; z: s32; w: s32; }\n\
                  f :: (using o: Outer) {\n    x = 9;\n}\nmain :: () { }\n";
    let (_, error) = failure(source);
    assert!(error.suggestions().is_empty());
    assert_eq!(error.help, ["copy it into a local first: `x := x;`"]);
}

#[test]
fn assigning_to_something_that_is_not_a_place_says_what_it_is() {
    let cases = [
        ("1 = 2;", "cannot assign to a literal"),
        ("f() = 3;", "cannot assign to the result of a call to `f`"),
        ("x + 1 = 3;", "cannot assign to the result of an operator"),
        ("cast(s32) x = 3;", "cannot assign to the result of a cast"),
    ];
    for (statement, message) in cases {
        let source =
            format!("f :: () -> int {{ return 1; }}\nmain :: () {{ x := 1; {statement} }}");
        let (_, error) = failure(&source);
        assert_eq!(error.message, message, "{statement}");
        assert_eq!(
            error.help,
            ["only a variable, a member, an element or a dereference (`p.*`) can be assigned to"]
        );
    }
}

#[test]
fn a_discarded_must_result_can_be_assigned_or_dropped_on_purpose() {
    let source = "f :: () -> int #must { return 1; }\nmain :: () {\n    f();\n}\n";
    let (c, error) = failure(source);
    assert!(
        error
            .message
            .contains("is marked #must and cannot be discarded")
    );
    let suggestion = &error.suggestions()[0];
    assert_eq!(
        suggestion.title,
        "discard the result on purpose with `_ := ...`"
    );
    assert_eq!(suggestion.applicability, Applicability::MaybeIncorrect);
    assert_eq!(error.help, ["or use the result: `result := f(...);`"]);
    assert!(apply(&c, &error, 0).contains("    _ := f();"));
    fixed(source);
}

#[test]
fn every_result_of_a_must_call_gets_a_discard() {
    let source = "f :: () -> int, int #must { return 1, 2; }\nmain :: () {\n    f();\n}\n";
    let (c, error) = failure(source);
    assert!(apply(&c, &error, 0).contains("    _, _ := f();"));
    fixed(source);
}

#[test]
fn a_must_call_inside_another_statement_is_not_given_a_discard_edit() {
    let (_, error) = failure("f :: () -> int #must { return 1; }\nmain :: () { if true f(); }");
    assert!(error.suggestions().is_empty());
    assert_eq!(error.help.len(), 1);
}

#[test]
fn a_partly_taken_must_call_asks_for_all_results() {
    let (_, error) =
        failure("f :: () -> int, int #must { return 1, 2; }\nmain :: () { x := f(); }");
    assert!(error.suggestions().is_empty());
    assert_eq!(
        error.help,
        ["`f` returns 2 results: take all of them, using `_` for the ones you do not need"]
    );
}

const POINT: &str = "S :: struct { a: int; b: int; c: int; }\n";

#[test]
fn a_struct_literal_with_too_few_values_lists_the_members() {
    let source = format!("{POINT}main :: () {{ s := S.{{1, 2}}; }}");
    let (_, error) = failure(&source);
    assert_eq!(error.message, "not enough values in this struct literal");
    assert_eq!(error.label.as_deref(), Some("no value for member `c`"));
    assert_eq!(
        text_notes(&error),
        ["`S` needs a value for each of `a`, `b`, `c` in this order; this literal gives 2"]
    );
    assert!(error.help[0].contains("name the members you set"));
}

#[test]
fn a_struct_literal_with_too_many_values_lists_the_members_and_removes_the_extra() {
    let source = format!("{POINT}main :: () {{ t := S.{{1, 2, 3, 4, 5}}; }}");
    let (c, error) = failure(&source);
    assert_eq!(error.message, "too many values for `S`");
    assert_eq!(text_notes(&error), ["`S` has 3 members: `a`, `b`, `c`"]);
    assert_eq!(error.suggestions()[0].title, "remove the extra values");
    assert!(apply(&c, &error, 0).contains("S.{1, 2, 3}; }"));
    fixed(&source);
}

#[test]
fn an_enum_value_out_of_range_widens_the_type() {
    let source = "E :: enum u8 { A :: 300; }\nmain :: () { }";
    let (c, error) = failure(source);
    assert_eq!(
        error.message,
        "enum value 300 is out of range for the enum's type `u8`"
    );
    assert_eq!(text_notes(&error), ["`u8` holds 0 to 255"]);
    let suggestion = &error.suggestions()[0];
    assert_eq!(suggestion.title, "widen the enum's type to `u16`");
    assert_eq!(suggestion.applicability, Applicability::MaybeIncorrect);
    assert_eq!(
        apply(&c, &error, 0),
        "E :: enum u16 { A :: 300; }\nmain :: () { }"
    );
    fixed(source);
}

#[test]
fn the_enum_type_is_widened_only_as_far_as_the_value_needs() {
    let source = "E :: enum s8 { A :: 300; }\nmain :: () { }";
    let (c, error) = failure(source);
    assert!(
        error.message.contains("enum value 300"),
        "{}",
        error.message
    );
    assert!(apply(&c, &error, 0).contains("enum s16 {"));
    // A negative value in an unsigned enum needs a signed type.
    let negative = "E :: enum u8 { A :: -1; }\nmain :: () { }";
    let (c, error) = failure(negative);
    assert!(apply(&c, &error, 0).contains("enum s8 {"));
    fixed(negative);
}

#[test]
fn an_enum_that_names_no_type_gets_help_only() {
    let source = "E :: enum_flags { A; B; C; D; E; F; G; H; I; J; K; L; M; N; O; P; Q; R; S; T; U; V; W; X; Y; Z; A2; B2; C2; D2; E2; F2; G2; }\nmain :: () { }";
    let (_, error) = failure(source);
    assert!(error.message.contains("out of range"), "{}", error.message);
    assert!(error.suggestions().is_empty());
    assert!(error.help[0].starts_with("give the enum a wider backing type"));
}

#[test]
fn a_lambda_with_the_wrong_arity_notes_the_expected_signature() {
    let source = "main :: () { g: (int) -> int = (a: int, b: int) => a; }";
    let (_, error) = failure(source);
    assert_eq!(
        error.message,
        "lambda takes 2 parameters, but `(s64) -> s64` expects 1"
    );
    assert_eq!(
        text_notes(&error),
        ["the expected signature is `(s64) -> s64`"]
    );
}

#[test]
fn a_lambda_argument_with_the_wrong_arity_notes_the_expected_signature() {
    let source = "apply :: (f: (int, int) -> int) { }\nmain :: () { apply((x: int) => x); }";
    let (_, error) = failure(source);
    assert!(
        error
            .message
            .contains("lambda takes 1 parameter, but `(s64, s64) -> s64` expects 2"),
        "{}",
        error.message
    );
    assert!(text_notes(&error).contains(&"the expected signature is `(s64, s64) -> s64`"));
}

#[test]
fn extra_values_in_a_declaration_can_be_removed() {
    let source = "main :: () {\n    a, b := 1, 2, 3;\n}\n";
    let (c, error) = failure(source);
    assert_eq!(error.message, "2 names but 3 values");
    assert_eq!(error.suggestions()[0].title, "remove the extra value");
    assert_eq!(apply(&c, &error, 0), "main :: () {\n    a, b := 1, 2;\n}\n");
    fixed(source);
}

#[test]
fn extra_values_in_a_constant_declaration_can_be_removed() {
    let source = "A, B :: 1, 2, 3, 4;\nmain :: () { }\n";
    let (c, error) = failure(source);
    assert_eq!(error.message, "2 names but 4 values");
    assert_eq!(error.suggestions()[0].title, "remove the extra values");
    assert_eq!(apply(&c, &error, 0), "A, B :: 1, 2;\nmain :: () { }\n");
    fixed(source);
}

#[test]
fn too_few_values_get_help_and_no_edit() {
    let (_, error) = failure("A, B, C :: 1, 2;\nmain :: () { }\n");
    assert_eq!(error.message, "3 names but 2 values");
    assert!(error.suggestions().is_empty());
    assert_eq!(
        error.help,
        ["give each of the 3 names its own value, or use fewer names"]
    );
}

#[test]
fn a_call_that_returns_fewer_values_than_names_says_how_many() {
    let (_, error) = failure("f :: () -> int { return 1; }\nmain :: () { a, b := f(); }");
    assert_eq!(error.message, "2 names but the call returns 1 value");
    assert_eq!(
        error.help,
        ["name only the 1 value the call returns, or call something that returns 2"]
    );
}

#[test]
fn a_repeated_load_is_reported_in_the_style_of_the_other_messages() {
    let out = rendered_file("#load \"main.jai\";\nmain :: () { }\n");
    assert!(
        out.contains("`main.jai` is already loaded into this module"),
        "{out}"
    );
    assert!(out.contains("help: remove this `#load`"), "{out}");
}

fn rendered_file(source: &str) -> String {
    common::compile_main_file(source).expect_err("an error")
}
