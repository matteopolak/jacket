//! Diagnostics for calls whose polymorphic parameters cannot be inferred from the arguments.
mod common;
use common::compile_source;

/// The rendered error of compiling `declarations` followed by `main :: () { body }`.
fn error_of(declarations: &str, body: &str) -> String {
    let source = format!("{declarations}\nmain :: () {{ {body} }}");
    let (c, result) = compile_source(&source);
    match result {
        Ok(()) => panic!("expected an error from `{body}`"),
        Err(error) => error.render(&c.src.sources),
    }
}

#[track_caller]
fn rejects(declarations: &str, body: &str, expected: &str) {
    let rendered = error_of(declarations, body);
    assert!(
        rendered.contains(expected),
        "`{body}` should report `{expected}`:\n{rendered}"
    );
}

const PAIR: &str = "Pair :: struct($A: Type, $B: Type) { a: A; b: B; }";

#[test]
fn explicit_type_variable_must_be_a_type() {
    rejects(
        "ident :: (x: $T) -> T { return x; }",
        "ident(5, T=3);",
        "`$T` must be given a type",
    );
}

#[test]
fn baked_parameter_without_default_needs_an_argument() {
    rejects(
        "Info :: struct { n: int; }\nbaked :: ($q: Info) -> int { return q.n; }",
        "baked();",
        "missing argument for baked parameter `q`",
    );
}

#[test]
fn conflicting_bindings_are_reported_with_both_types() {
    rejects(
        "Box :: struct($T: Type) { v: T; }\nboth :: (a: $T, b: Box($T)) {}",
        "s: Box(string); both(1, s);",
        "conflicting types for `$T`",
    );
}

#[test]
fn unbound_type_variable_is_named() {
    rejects(
        "none :: (n: int, rest: ..$T) {}",
        "none(1);",
        "could not infer polymorphic type `$T`",
    );
}

#[test]
fn array_lengths_must_agree() {
    let declarations = "same :: (a: [$N] int, b: [N] int) {}\nlit :: (a: [$N] int = .[1, 2]) {}";
    rejects(
        declarations,
        "a: [2] int; b: [3] int; same(a, b);",
        "does not match parameter type",
    );
    rejects(declarations, "lit(.[7, 8, 9]);", "unequal array lengths");
}

#[test]
fn patterns_report_what_the_argument_was() {
    rejects("p :: (a: *$T) {}", "p(5);", "expected a pointer, found");
    rejects("v :: (a: [] $T) {}", "v(5);", "expected an array, found");
    rejects(
        "r :: (a: [..] $T) {}",
        "arr: [3] int; r(arr);",
        "array kind mismatch",
    );
    rejects(
        "f :: (cb: ($A) -> $R) {}",
        "f(5);",
        "expected a procedure, found",
    );
}

#[test]
fn polymorphic_struct_patterns_reject_other_types() {
    let pairs = format!(
        "{PAIR}\nbare :: (p: Pair) {{}}\nargs :: (p: Pair($X, $Y)) {{}}\nrestricted :: (p: $P/Pair) {{}}"
    );
    rejects(&pairs, "bare(5);", "is not an instance of `Pair`");
    rejects(&pairs, "args(5);", "is not an instance of this struct");
    rejects(&pairs, "restricted(5);", "is not an instance of `Pair`");
}

#[test]
fn restriction_to_a_struct_requires_it_as_a_base() {
    rejects(
        "Base :: struct { id: int; }\nOther :: struct { id: int; }\nneeds :: (e: $E/Base) {}",
        "o: Other; needs(o);",
        "does not satisfy the restriction `Base`",
    );
}

#[test]
fn procedure_arguments_must_match_an_overload() {
    rejects(
        "apply :: (f: (T) -> $R, x: $T) -> R { return f(x); }\n\
         dbl :: (x: int) -> int { return x; }\n\
         dbl :: (x: float) -> float { return x; }",
        "apply(dbl, \"s\");",
        "no overload matches the procedure parameter",
    );
}
