//! Declaration lookup must not request offsets from a record being laid out.
use test_support::compiler::compile_with_stdlib;

fn check(source: &str) -> Result<(), String> {
    let (compiler, result) = compile_with_stdlib(source);

    result.map_err(|error| error.render(&compiler.src.sources))
}

#[test]
fn promoted_nested_array_fields_do_not_form_storage_cycles() {
    for aggregate in ["struct", "union"] {
        for member in ["using bar: struct", "struct"] {
            for count in ["size_of(u32)", "#run size_of(u32)"] {
                let source = format!(
                    "Foo :: {aggregate} {{ {member} {{ baz: [{count}] s64; }} }}\n\
                     #assert size_of(Foo) == 32;\n\
                     #assert offset_of(Foo, \"baz\") == 0;\n\
                     main :: () {{ object: Foo; object.baz[3] = 7; }}"
                );

                check(&source).unwrap_or_else(|error| panic!("{source}\n{error}"));
            }
        }
    }
}

#[test]
fn recursive_storage_is_still_rejected() {
    let error = check("Foo :: struct { value: Foo; } main :: () {}").unwrap_err();

    assert!(error.contains("contains itself"), "{error}");
}
