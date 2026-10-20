//! The argument of a number-typed constant parameter (`$N: s32`) is checked against the declared
//! type as for a normal parameter.
mod common;
use common::compile_source;

const DECLS: &str = "S :: struct { a: s32; } f :: ($N: s32) -> s32 { return N * 2; }";

fn rejected(argument: &str) -> String {
    let source = format!("{DECLS} main :: () {{ x := f({argument}); }}");
    let (c, result) = compile_source(&source);
    match result {
        Ok(()) => panic!("compiled: f({argument})"),
        Err(error) => error.render(&c.src.sources),
    }
}

#[test]
fn an_argument_of_another_type_is_rejected() {
    for argument in ["true", "S.{1}", ".[1, 2]", "null", "\"text\"", "2.5"] {
        let message = rejected(argument);
        assert!(message.contains("in call to `f`"), "{argument}: {message}");
    }
}

#[test]
fn a_matching_argument_is_accepted() {
    for argument in ["4", "-3"] {
        let source =
            format!("{DECLS} main :: () {{ x := f({argument}); #assert(type_of(x) == s32); }}");
        let (c, result) = compile_source(&source);
        if let Err(error) = result {
            panic!("f({argument}): {}", error.render(&c.src.sources));
        }
    }
}
