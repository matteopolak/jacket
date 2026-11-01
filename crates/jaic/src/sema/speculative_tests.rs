//! Speculative bodies whose first step is the first use of something (`speculative.rs`).
use super::speculative::{Counts, counts, force};
use super::{Compiler, Options, VirtualFs};
use std::rc::Rc;

/// What a compile produced in the order it numbered things, and how the bodies went.
fn compile(source: &str, speculate: bool) -> (String, Counts) {
    let mut fs = VirtualFs::default();
    fs.insert("/spec/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    force(speculate);
    let mut compiler = Compiler::new(options, Rc::new(fs));
    // `Compiler::new` reads the switch again.
    force(speculate);
    compiler
        .compile_program(std::path::Path::new("/spec/main.jai"))
        .unwrap();
    let program = &compiler.out.program;
    let shape = format!(
        "{:?}\n{:?}\n{:?}",
        program.func_names, program.funcs, program.globals
    );
    (shape, counts())
}

/// A fan of procedures, each calling two that nothing has used yet, a constant and a struct:
/// every body is the first to name something, which used to stop it.
fn first_uses() -> String {
    let mut source = String::from(
        "LIMIT :: 10;\nPair :: struct { a: s64; b: s64; }\n\
         leaf :: (p: Pair) -> s64 { return p.a + p.b + LIMIT; }\n",
    );
    for i in 0..24 {
        source += &format!(
            "mid{i} :: (x: s64) -> s64 {{ return leaf{i}(Pair.{{x, LIMIT}}) + leaf(Pair.{{x, {i}}}); }}\n\
             leaf{i} :: (p: Pair) -> s64 {{ return p.a * {i} + LIMIT; }}\n"
        );
    }
    source += "main :: () {\n";
    for i in 0..24 {
        source += &format!("    v{i} := mid{i}({i});\n");
    }
    source + "}\n"
}

#[test]
fn first_use_bodies_commit_with_identical_output() {
    let source = first_uses();
    let (serial, _) = compile(&source, false);
    let (speculative, counts) = compile(&source, true);
    assert_eq!(serial, speculative);
    assert!(counts.batched >= 24, "batched {}", counts.batched);
    assert!(
        counts.recovered >= 10,
        "few bodies committed after running a first use ahead: {} steps, {} recovered, {} committed of {}",
        counts.resolved_ahead,
        counts.recovered,
        counts.committed,
        counts.batched
    );
    assert!(
        counts.committed * 2 > counts.batched,
        "committed {} of {}",
        counts.committed,
        counts.batched
    );
}

#[test]
fn constants_resolved_in_a_body_are_kept_in_order() {
    let source = "
        A :: 3;
        B :: A * 2;
        C :: B + A;
        f :: () -> s64 { return C; }
        g :: () -> s64 { return B + f(); }
        main :: () { x := g(); y := f(); }
    ";
    let (serial, _) = compile(source, false);
    let (speculative, counts) = compile(source, true);
    assert_eq!(serial, speculative);
    assert!(counts.committed >= 1);
}

#[test]
fn thunk_scope_made_by_two_bodies_is_made_once() {
    let source = "
        f :: () -> s64 { return #run 1 + 2; }
        g :: () -> s64 { return #run 3 + 4; }
        main :: () { x := f() + g(); }
    ";
    let (serial, _) = compile(source, false);
    let (speculative, _) = compile(source, true);
    assert_eq!(serial, speculative);
}
