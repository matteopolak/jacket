//! A `#type` alias written with type variables is a pattern: using it as a value is an error at
//! compile time, not a failure when the program runs.
use test_support::compiler::compile_source;

const ALIAS: &str = "Iter :: #type (value: *$T, n: int);\n";

fn rejection(body: &str) -> String {
    let (c, result) = compile_source(&format!("{ALIAS}{body}"));
    let error = result.expect_err("a polymorphic type alias is not a value");
    error.render(&c.src.sources)
}

#[test]
fn polymorphic_alias_as_a_value_is_rejected() {
    for body in [
        "main :: () { t: Type = Iter; }",
        "main :: () { t := Iter; }",
        "show :: (a: Any) {}\nmain :: () { show(Iter); }",
    ] {
        let message = rejection(body);
        assert!(
            message.contains("is a pattern, not a value"),
            "{body}\n{message}"
        );
    }
}
