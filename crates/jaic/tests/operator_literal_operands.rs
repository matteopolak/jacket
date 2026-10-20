//! An untyped struct literal as an operand of an overloaded binary operator is typed by the
//! operator overloads, as a literal argument of a call is.
mod common;
use common::compile_source;

const TYPES: &str = "
S :: struct { a, b: float; }
T :: struct { a, b, c: float; }
";

/// The rendered error `source` fails with.
fn failure(source: &str) -> String {
    let (c, result) = compile_source(source);
    match result {
        Ok(()) => panic!("compiled: {source}"),
        Err(error) => error.render(&c.src.sources),
    }
}

fn accepts(source: &str) {
    let (c, result) = compile_source(source);
    if let Err(error) = result {
        panic!("{}", error.render(&c.src.sources));
    }
}

#[test]
fn a_literal_operand_takes_the_type_of_the_only_matching_overload() {
    for body in [
        // The literal on the left, then on the right.
        "x := .{ 1.0, 2.0 } * 2.0; #assert(type_of(x) == S);",
        "operator + :: (a: S, b: S) -> S { return a; } v := S.{}; x := v + .{ 1.0, 2.0 }; #assert(type_of(x) == S);",
        "operator + :: (a: S, b: S) -> S { return a; } v := S.{}; x := .{ 1.0, 2.0 } + v; #assert(type_of(x) == S);",
        // Compound assignment goes through the same operator.
        "operator + :: (a: S, b: S) -> S { return a; } v := S.{}; v += .{ 1.0, 2.0 };",
        // Comparisons, and `!=` through `==`.
        "operator == :: (a: S, b: S) -> bool { return a.a == b.a; } v := S.{}; c := v == .{ 1.0, 2.0 }; d := .{ 1.0, 2.0 } != v; #assert(type_of(c) == bool && type_of(d) == bool);",
    ] {
        accepts(&format!(
            "{TYPES} operator * :: (x: S, f: float) -> S {{ return x; }} main :: () {{ {body} }}"
        ));
    }
}

#[test]
fn a_literal_with_too_few_values_does_not_match_the_larger_struct() {
    // `T` has three fields without initializers, so two values pick `S`; with three values the
    // literal picks `T`.
    accepts(&format!(
        "{TYPES}
        operator * :: (x: S, f: float) -> S {{ return x; }}
        operator * :: (x: T, f: float) -> T {{ return x; }}
        main :: () {{
            x := .{{ 1.0, 2.0 }} * 2.0;
            y := .{{ 1.0, 2.0, 3.0 }} * 2.0;
            #assert(type_of(x) == S);
            #assert(type_of(y) == T);
        }}"
    ));
}

#[test]
fn equally_good_overloads_for_a_literal_operand_are_ambiguous() {
    let error = failure(
        "S :: struct { a, b: float; }
         T :: struct { a, b: float; }
         operator * :: (x: S, f: float) -> S { return x; }
         operator * :: (x: T, f: float) -> T { return x; }
         main :: () { x := .{ 1.0, 2.0 } * 2.0; }",
    );
    assert!(
        error.contains("Procedure call matches 2 possible overloads"),
        "{error}"
    );
    assert!(
        error.contains("is ambiguous for these arguments"),
        "{error}"
    );
    assert!(error.contains("operator *"), "{error}");
    // Both candidates are named, at their own declarations.
    assert!(error.matches("operator * ::").count() >= 2, "{error}");
}

#[test]
fn a_literal_operand_without_any_overload_is_still_an_error() {
    let error = failure("main :: () { x := .{ 1, 2 } * 2; }");
    assert!(
        error.contains("cannot infer the type of `.{...}`"),
        "{error}"
    );
}

#[test]
fn a_literal_that_fits_no_overload_is_an_error() {
    let error = failure(&format!(
        "{TYPES} operator * :: (x: T, f: float) -> T {{ return x; }}
         main :: () {{ x := .{{ 1.0, 2.0 }} * 2.0; }}"
    ));
    assert!(error.contains("operator"), "{error}");
}

#[test]
fn a_type_is_not_an_operand_of_an_overloaded_operator() {
    let error = failure(&format!(
        "{TYPES} operator * :: (x: S, f: float) -> S {{ return x; }}
         main :: () {{ x := .{{ 1.0, 2.0 }} * S; }}"
    ));
    assert!(
        error.contains("a type cannot be an operand of an overloaded operator"),
        "{error}"
    );
}
