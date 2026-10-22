//! Declaration lookup must not request offsets from a record being laid out.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

fn check(source: &str) -> Result<(), String> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.src.sources))
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
