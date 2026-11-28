//! Two instances of one parameterized module declare distinct types with the same name; a
//! mismatch between them names the instance each one comes from.
use std::{path::Path, rc::Rc};

use jaic::sema::{Compiler, Options, VirtualFs};

fn message(main: &str) -> String {
    let mut fs = VirtualFs::default();

    fs.insert(
        "/lib/Shape/module.jai",
        b"#module_parameters (KIND := 0); Shape :: struct { x: s64; }".as_slice(),
    );
    fs.insert("/main.jai", main.as_bytes());
    let mut options = Options::host();

    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));

    match compiler.compile_program(Path::new("/main.jai")) {
        Err(d) => d.message.clone(),
        Ok(()) => panic!("expected a type mismatch"),
    }
}

#[test]
fn mismatch_between_module_instances_tells_them_apart() {
    let text = message(
        "A :: #import \"Shape\"; B :: #import \"Shape\"(KIND = 1);\n\
         main :: () { a: A.Shape; b: *B.Shape = *a; }",
    );

    assert!(text.contains("type mismatch"), "{text}");
    assert!(text.contains("Shape(KIND = 1)"), "{text}");
    assert!(text.contains("Shape"), "{text}");
    let (expected, found) = text.split_once(", found").expect("found part");

    assert_ne!(expected.replace("expected", ""), found, "{text}");
}

#[test]
fn mismatch_between_unrelated_types_keeps_plain_names() {
    let text = message("main :: () { a: s64 = 1; b: *s64 = a; }");

    assert!(!text.contains("module"), "{text}");
}
