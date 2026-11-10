//! The integer a `#run` computes by executing code is an untyped integer constant; a typed
//! constant, or a `#run` of a constant expression, keeps its type.
use test_support::compiler::compile_source;

/// Compile `consts` followed by a `main` holding `body`; the error rendered if it fails.
fn plain(consts: &str, body: &str) -> Result<(), String> {
    let source = format!("{consts}\nmain :: () {{ {body} }}\n");
    let (c, result) = compile_source(&source);
    result.map_err(|e| e.render(&c.src.sources))
}

/// `plain` with a `#run` constant `C` of `ty` whose procedure returns `value`.
fn with_run(ty: &str, value: &str, body: &str) -> Result<(), String> {
    plain(
        &format!("f :: () -> {ty} {{ return {value}; }}\nC :: #run f();"),
        body,
    )
}

#[test]
fn a_run_result_fits_the_other_operand() {
    with_run(
        "s64",
        "16",
        "slot: u64 = 3; m := (slot - 1) * C; a: u64 = C; b: u8 = C; \
         s: u32 = 3; x := C * s; y := C + s; \
         #assert(type_of(m) == u64); #assert(type_of(x) == u32); #assert(type_of(y) == u32);",
    )
    .unwrap();
    with_run("s16", "5", "a: u8 = C;").unwrap();
    with_run("u64", "5", "a: u8 = C;").unwrap();
}

#[test]
fn a_constant_derived_from_a_run_result_stays_untyped() {
    plain(
        "f :: () -> s64 { return 5; }\nC :: #run f();\nE :: C * 2;",
        "a: u8 = E; s: u32 = 1; b := s + E; #assert(type_of(b) == u32);",
    )
    .unwrap();
}

#[test]
fn a_run_result_that_does_not_fit_is_an_error_like_a_literal() {
    let too_big = with_run("s64", "300", "b: u8 = C;").unwrap_err();
    assert!(too_big.contains("error["), "{too_big}");
    let negative = with_run("s64", "-16", "slot: u64 = 3; x := slot * C;").unwrap_err();
    assert!(negative.contains("-16"), "{negative}");
}

#[test]
fn typed_constants_keep_their_type() {
    for consts in [
        "LARGEST :: cast(s64) 16;",
        "LARGEST : s64 : 16;",
        "LARGEST :: #run cast(s64) 16;",
    ] {
        let error = plain(consts, "slot: u64 = 3; x := slot * LARGEST;").expect_err(consts);
        assert!(error.contains("error["), "{consts}: {error}");
    }
    for consts in ["C :: #run cast(s64) 16;", "C :: #run cast(s32) 5;"] {
        let error = plain(consts, "a: u8 = C;").expect_err(consts);
        assert!(error.contains("error["), "{consts}: {error}");
    }
    plain("C :: #run (16 + 1);", "a: u8 = C;").unwrap();
}

#[test]
fn a_float_run_result_keeps_its_type() {
    let error = plain(
        "f :: () -> float64 { return 2.5; }\nF :: #run f();",
        "a: float32 = F;",
    )
    .unwrap_err();
    assert!(error.contains("error["), "{error}");
}

#[test]
fn a_typed_run_written_where_an_integer_is_expected_keeps_its_type() {
    for body in ["x: u64 = #run cast(s64) 16;", "x: s32 = #run cast(s64) 5;"] {
        let error = plain("", body).expect_err(body);
        assert!(error.contains("error["), "{body}: {error}");
    }
    plain("f :: () -> s64 { return 16; }", "x: u32 = #run f();").unwrap();
}
