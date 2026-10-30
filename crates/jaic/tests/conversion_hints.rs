//! The help on conversion and cast errors is a rewrite of the code: a `cast`, a dereference,
//! an address, a comparison, a range or a flag. Each test checks the message and the help, then
//! applies the edit to the source and compiles the result to show it type-checks.
use jaic::diagnostics::Diagnostic;
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::source::Applicability;
use std::{path::Path, rc::Rc};

const PRELUDE: &str = "P :: struct { x: s64; }
Q :: struct { y: s64; }
take :: (n: s32) -> s32 { return n; }
";

fn compile(source: &str) -> (Result<(), Box<Diagnostic>>, Compiler) {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    let result = compiler.compile_program(Path::new("/main.jai"));
    (result, compiler)
}

/// A program whose `main` is `body`, after the prelude.
fn program(body: &str) -> String {
    format!(
        "{PRELUDE}main :: () {{\n    a: s64 = 3;\n    b: float = 2;\n    u: u64 = 1;\n    k: bool = true;\n    {body}\n}}\n"
    )
}

fn error_of(source: &str) -> Diagnostic {
    match compile(source).0 {
        Err(d) => *d,
        Ok(()) => panic!("expected an error in:\n{source}"),
    }
}

/// The source with the replacements of suggestion `index` applied.
fn applied(source: &str, d: &Diagnostic, index: usize) -> String {
    let mut edits: Vec<_> = d.suggestions()[index]
        .replacements()
        .map(|(span, text)| (span.start as usize, span.end as usize, text.to_string()))
        .collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = source.to_string();
    for (start, end, text) in edits {
        out.replace_range(start..end, &text);
    }
    out
}

/// Fix `body`: the message begins with `message`, the first suggestion is titled `help`, and
/// applying it leaves `main` as `fixed` and a program that compiles.
#[track_caller]
fn fixes_to(body: &str, message: &str, help: &str, fixed: &str) {
    let source = program(body);
    let d = error_of(&source);
    assert!(d.message.starts_with(message), "{}", d.message);
    let suggestion = &d.suggestions()[0];
    assert_eq!(suggestion.title, help);
    assert_eq!(suggestion.applicability, Applicability::MaybeIncorrect);
    let result = applied(&source, &d, 0);
    assert_eq!(result, program(fixed));
    let (outcome, compiler) = compile(&result);
    if let Err(e) = outcome {
        panic!(
            "the fixed program does not compile:\n{}",
            e.render(&compiler.src.sources)
        );
    }
}

#[test]
fn an_assignment_between_integers_is_wrapped_in_a_cast() {
    fixes_to(
        "c: s32 = a;",
        "type mismatch: expected `s32`, found `s64`",
        "convert with `cast(s32)` if every value fits",
        "c: s32 = cast(s32) a;",
    );
}

#[test]
fn an_expression_is_parenthesised_inside_the_cast() {
    fixes_to(
        "c: s32 = a + 1;",
        "type mismatch: expected `s32`, found `s64`",
        "convert with `cast(s32)` if every value fits",
        "c: s32 = cast(s32) (a + 1);",
    );
}

#[test]
fn a_float_assigned_to_an_integer_is_cast() {
    fixes_to(
        "c: s32 = b;",
        "type mismatch: expected `s32`, found `float32`",
        "convert with `cast(s32)`, which drops the fraction",
        "c: s32 = cast(s32) b;",
    );
}

#[test]
fn a_call_argument_is_wrapped_in_a_cast() {
    fixes_to(
        "c := take(a);",
        "in call to `take`: argument of type `s64` does not match parameter type `s32`",
        "convert with `cast(s32)` if every value fits",
        "c := take(cast(s32) a);",
    );
}

#[test]
fn a_pointer_where_its_value_is_expected_is_dereferenced() {
    fixes_to(
        "p := *a; c: s64 = p;",
        "type mismatch: expected `s64`, found `*s64`",
        "this is a pointer to the value: dereference it with `value.*`",
        "p := *a; c: s64 = p.*;",
    );
}

#[test]
fn a_value_where_its_pointer_is_expected_gets_its_address_taken() {
    fixes_to(
        "p: *s64 = a;",
        "type mismatch: expected `*s64`, found `s64`",
        "this is the value itself: take its address with `*value`",
        "p: *s64 = *a;",
    );
}

#[test]
fn an_integer_used_as_a_bool_is_compared_with_zero() {
    fixes_to(
        "c: bool = a;",
        "type mismatch: expected `bool`, found `s64`",
        "compare instead: `value != 0`",
        "c: bool = a != 0;",
    );
}

#[test]
fn an_integer_operand_of_another_signedness_is_cast_alone() {
    // The error points at the operand, so the edit leaves the other one as it is.
    fixes_to(
        "v: s64 = 2; c := u + v;",
        "type mismatch: expected `u64`, found `s64`",
        "convert with `cast(u64)` if every value fits",
        "v: s64 = 2; c := u + cast(u64) v;",
    );
}

#[test]
fn a_bool_meeting_a_number_is_cast_to_the_number() {
    fixes_to(
        "c := k + a;",
        "type mismatch: `bool` and `s64` cannot be combined",
        "convert the left operand with `cast(s64)`",
        "c := cast(s64) k + a;",
    );
    fixes_to(
        "c := b * k;",
        "type mismatch: `float32` and `bool` cannot be combined",
        "convert the right operand with `cast(float32)`",
        "c := b * cast(float32) k;",
    );
}

#[test]
fn a_binary_operand_with_no_conversion_has_no_cast_suggestion() {
    let d = error_of(&program("s := \"x\"; c := a + s;"));
    assert_eq!(
        d.message,
        "type mismatch: `s64` and `string` cannot be combined"
    );
    assert!(d.suggestions().is_empty());
}

#[test]
fn extra_return_values_are_dropped() {
    let source =
        format!("{PRELUDE}two :: () -> s64 {{\n    return 1, 2;\n}}\nmain :: () {{ two(); }}\n");
    let d = error_of(&source);
    assert_eq!(d.message, "too many return values (procedure returns 1)");
    assert_eq!(d.help, ["the procedure returns `s64`"]);
    assert_eq!(d.suggestions()[0].title, "return only the first 1 value");
    let fixed = applied(&source, &d, 0);
    assert!(fixed.contains("return 1;"), "{fixed}");
    assert!(compile(&fixed).0.is_ok());
}

#[test]
fn a_value_returned_from_a_procedure_without_results_is_dropped() {
    let source = format!("{PRELUDE}none :: () {{\n    return 5;\n}}\nmain :: () {{ none(); }}\n");
    let d = error_of(&source);
    assert_eq!(d.message, "too many return values (procedure returns 0)");
    assert_eq!(d.help, ["the procedure returns nothing"]);
    assert_eq!(d.suggestions()[0].title, "return without a value");
    let fixed = applied(&source, &d, 0);
    assert!(fixed.contains("return;"), "{fixed}");
    assert!(compile(&fixed).0.is_ok());
}

#[test]
fn a_missing_return_value_names_the_result_types() {
    let source = format!(
        "{PRELUDE}pair :: () -> s64, string {{\n    return;\n}}\nmain :: () {{ pair(); }}\n"
    );
    let d = error_of(&source);
    assert_eq!(d.message, "missing return value");
    assert_eq!(d.help, ["the procedure returns `s64`, `string`"]);
}

#[test]
fn looping_over_a_number_suggests_a_range() {
    fixes_to(
        "for a { }",
        "cannot iterate over a value of type s64",
        "to repeat a times, loop over a range: `for 0..a - 1`",
        "for 0..a - 1 { }",
    );
    fixes_to(
        "for a + 1 { }",
        "cannot iterate over a value of type s64",
        "to repeat a + 1 times, loop over a range: `for 0..(a + 1) - 1`",
        "for 0..(a + 1) - 1 { }",
    );
}

#[test]
fn looping_over_a_struct_says_where_to_define_the_expansion() {
    let d = error_of(&program("for P.{} { }"));
    assert_eq!(
        d.message,
        "no `for_expansion` is visible for iterating over P"
    );
    assert_eq!(
        d.help,
        [
            "define `for_expansion` for `P` in a visible scope, or iterate over one of its array members"
        ]
    );
    assert!(d.suggestions().is_empty());
}

#[test]
fn dereferencing_a_value_that_is_not_a_pointer_removes_the_dereference() {
    fixes_to(
        "c := a.*;",
        "cannot dereference a value of type s64",
        "`a` is not a pointer: remove the dereference",
        "c := a;",
    );
}

#[test]
fn calling_a_numeric_type_suggests_a_cast() {
    fixes_to(
        "c := s32(a);",
        "cannot call type s32",
        "to convert a value to `s32`, write `cast(s32) a`",
        "c := cast(s32) a;",
    );
    fixes_to(
        "c := float(a + 1);",
        "cannot call type float32",
        "to convert a value to `float32`, write `cast(float32) (a + 1)`",
        "c := cast(float32) (a + 1);",
    );
}

#[test]
fn indexing_and_conditions_say_what_is_accepted() {
    let d = error_of(&program("c := a[0];"));
    assert_eq!(d.message, "cannot index a value of type s64");
    assert_eq!(d.help, ["only arrays, strings and pointers can be indexed"]);
    let d = error_of(&program("if P.{} { }"));
    assert_eq!(d.message, "a value of type P cannot be used as a condition");
    assert_eq!(
        d.help,
        [
            "a condition is a bool, a number, a pointer, a string or an array: test or compare a member instead"
        ]
    );
}

#[test]
fn a_constant_out_of_range_for_a_cast_suggests_truncation() {
    let source = program("c := cast(u8) 300;");
    let d = error_of(&source);
    assert!(d.message.starts_with("constant 300 is out of range for u8"));
    assert_eq!(
        d.suggestions()[0].title,
        "to keep the low bits, write `cast,trunc(u8)`"
    );
    let fixed = applied(&source, &d, 0);
    assert!(fixed.contains("c := cast,trunc(u8) 300;"), "{fixed}");
    assert!(compile(&fixed).0.is_ok());
}

#[test]
fn a_cast_between_aggregates_of_one_size_suggests_force() {
    let source = program("p := P.{}; c := cast(Q) p;");
    let d = error_of(&source);
    assert_eq!(d.message, "cannot cast P to Q");
    assert_eq!(
        d.suggestions()[0].title,
        "both have the same size: `cast,force` reinterprets the bytes"
    );
    let fixed = applied(&source, &d, 0);
    assert!(fixed.contains("c := cast,force(Q) p;"), "{fixed}");
    assert!(compile(&fixed).0.is_ok());
}

#[test]
fn a_cast_with_no_conversion_says_what_to_do_instead() {
    let d = error_of(&program("c := cast(s64) P.{};"));
    assert_eq!(d.message, "cannot cast P to s64");
    assert_eq!(
        d.help,
        ["P holds several values: cast one of its members to s64 instead"]
    );
    let d = error_of(&program("c := cast(P) a;"));
    assert_eq!(d.message, "cannot cast s64 to P");
    assert_eq!(
        d.help,
        ["build a P from its members with a struct literal instead: `P.{...}`"]
    );
}
