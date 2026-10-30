//! A declaration may not reuse the name of a member brought into the same scope by `using`;
//! a nested scope still shadows it.
use test_support::compiler::compile_source;

const TYPES: &str = "Outer :: struct { x: s32; y: s32; }";

fn outcome(body: &str) -> Result<(), String> {
    let source = format!("{TYPES} main :: () {{ o: Outer; {body} }}");
    let (c, result) = compile_source(&source);
    result.map_err(|error| error.render(&c.src.sources))
}

#[test]
fn a_declaration_reusing_a_used_member_is_rejected() {
    for body in [
        "using o; o.x = 1; x := 100;",
        "using o; x: s32;",
        "using o; x: s32 = 3;",
        "using o; x :: 5;",
        "using o; x, z := 1, 2;",
        "using o; y := 1;",
    ] {
        let message = outcome(body).expect_err(body);
        assert!(
            message.contains("is already declared in this scope"),
            "{body}: {message}"
        );
    }
}

#[test]
fn shadowing_and_unrelated_names_stay_allowed() {
    for body in [
        "using o; z := 100; z += x;",
        "using o; { x := 100; x += 1; }",
        "using o; if true { y: s32 = 2; y += 1; }",
        "x := 100; { using o; z := x; z += 1; }",
        "using o; for 0..2 { x := it; x += 1; }",
    ] {
        if let Err(message) = outcome(body) {
            panic!("{body}: {message}");
        }
    }
}
