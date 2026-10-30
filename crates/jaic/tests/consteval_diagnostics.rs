//! Messages of compile-time failures: `#assert` formatting and `#insert` of unsuitable values.
use test_support::compiler::compile_source;

fn error_of(source: &str) -> String {
    let (c, result) = compile_source(source);
    match result {
        Ok(()) => panic!("expected an error from:\n{source}"),
        Err(error) => error.render(&c.src.sources),
    }
}

#[test]
fn assert_message_fills_each_percent_with_the_next_argument() {
    let rendered = error_of(
        "main :: () {}\n\
         #assert(false, \"int % float % bool % string % type % null % other %\", 3, 1.5, true, \"s\", float, null, .[1]);",
    );
    assert!(
        rendered.contains("int 3 float 1.5 bool true string s type float32 null null other ?"),
        "{rendered}"
    );
}

#[test]
fn assert_message_keeps_an_escaped_percent_and_blanks_missing_arguments() {
    let rendered = error_of("main :: () {}\n#assert(false, \"100\\\\% of % and %\", 7);");
    assert!(rendered.contains("100% of 7 and "), "{rendered}");
}

#[test]
fn assert_without_a_string_message_reports_nothing_extra() {
    let rendered = error_of("main :: () {}\n#assert(false);");
    assert!(rendered.contains("error"), "{rendered}");
}

#[test]
fn insert_rejects_values_that_are_not_text_or_code() {
    let rendered = error_of("main :: () { #insert 5; }");
    assert!(
        rendered.contains("#insert needs a string or Code"),
        "{rendered}"
    );
    let rendered = error_of("main :: () { x := #insert 5; }");
    assert!(
        rendered.contains("#insert needs a string or Code"),
        "{rendered}"
    );
}

#[test]
fn inserted_strings_that_insert_themselves_stop_at_a_depth_limit() {
    let rendered = error_of("S :: \"#insert S;\";\nmain :: () { #insert S; }");
    assert!(
        rendered.contains("#insert strings are nested more than"),
        "{rendered}"
    );
}

#[test]
fn code_in_expression_position_must_be_one_expression() {
    let rendered = error_of("main :: () { x := #insert #code { a := 1; b := 2; }; }");
    assert!(
        rendered.contains("statement block in expression position"),
        "{rendered}"
    );
    let rendered = error_of("main :: () { x := #insert \"1 +\"; }");
    assert!(rendered.contains("error"), "{rendered}");
}
