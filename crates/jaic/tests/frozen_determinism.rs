//! Values frozen out of compile-time memory do not depend on where the interpreter put it.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

/// Every global of a build of `source`: name, initial bytes and relocation offsets.
fn globals(source: &str) -> Vec<(String, Vec<u8>, Vec<u64>)> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.src.sources));
    }
    c.out
        .program
        .globals
        .iter()
        .map(|g| {
            (
                g.name.to_string(),
                g.init.clone(),
                g.relocs.iter().map(|r| r.offset).collect(),
            )
        })
        .collect()
}

fn assert_same_twice(source: &str) {
    let first = globals(source);
    // Allocations in between move where the interpreter's next blocks land.
    let _ballast: Vec<Vec<u8>> = (1..64).map(|n| vec![1; n * 997]).collect();
    let second = globals(source);
    assert_eq!(first.len(), second.len(), "different globals");
    for (a, b) in first.iter().zip(&second) {
        assert_eq!(a, b, "global `{}` differs between builds", a.0);
    }
}

#[test]
fn opaque_pointers_in_a_frozen_value_do_not_carry_addresses() {
    assert_same_twice(
        r#"
        #import "Basic";
        Holder :: struct { count: int; token: *void; }
        make :: () -> Holder { h: Holder; h.count = 3; h.token = New(int); return h; }
        VALUE :: #run make();
        main :: () { x := VALUE; }
        "#,
    );
}

#[test]
fn unions_in_a_frozen_value_do_not_carry_addresses() {
    assert_same_twice(
        r#"
        #import "Basic";
        Either :: union { number: int; pointer: *u8; }
        Holder :: struct { count: int; either: Either; }
        make :: () -> Holder { h: Holder; h.count = 3; h.either.pointer = cast(*u8) New(int); return h; }
        VALUE :: #run make();
        main :: () { x := VALUE; }
        "#,
    );
}
