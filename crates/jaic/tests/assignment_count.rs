//! A list of values on the right of `a, b = ...` pairs with the targets one to one, as it does
//! for `a, b := ...`: a count that differs is an error, not a value that is dropped.
use jaic::sema::{Compiler, Options, VirtualFs};
use std::{path::Path, rc::Rc};

fn compile(body: &str) -> Result<(), String> {
    let source = format!("main :: () {{\n    a, b := 1, 2;\n    {body}\n}}\n");
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(Path::new("/main.jai"))
        .map_err(|e| e.message)
}

#[test]
fn an_extra_value_is_an_error() {
    assert_eq!(
        compile("a, b = 1, 2, 3;").unwrap_err(),
        "2 names but 3 values"
    );
}

#[test]
fn too_many_values_for_a_few_targets_is_an_error() {
    assert_eq!(
        compile("a, b = 1, 2, 3, 4;").unwrap_err(),
        "2 names but 4 values"
    );
    assert!(
        compile("a, b = 1;").is_ok(),
        "one value goes to every target"
    );
}

#[test]
fn matching_counts_still_assign() {
    compile("a, b = 3, 4;").unwrap();
    compile("a, b = b, a;").unwrap();
    compile("a, _ = 5, 6;").unwrap();
}

#[test]
fn the_declaring_form_reports_the_same_count_error() {
    assert_eq!(
        compile("c, d := 1, 2, 3;").unwrap_err(),
        "2 names but 3 values"
    );
}
