//! A struct or import that fails to resolve is not evaluated again until something changed,
//! so a program that depends on one does not repeat the failure for every lookup that meets it.
use std::cell::RefCell;
use std::rc::Rc;

use jaic::ide::{IdeFacts, IdeRoot};
use jaic::interp::{SandboxHost, SharedHost};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};

/// How many times the compiler asked whether to stop while checking `source` as an editor does.
fn polls(source: &str) -> u64 {
    let stdlib = test_support::repo::root().join("stdlib");
    let mut options = Options::host();

    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let host = Rc::new(RefCell::new(SandboxHost::default()));

    c.ct.interp.set_host(Box::new(SharedHost(host.clone())));
    c.ed.record(IdeFacts::new(vec![IdeRoot::Virtual]));
    let _ = c.begin_sources(&[ProgramSource::String(source.to_owned())]);
    let _ = c.finish_program();
    let _ = c.ide_check_all();
    let asked = host.borrow().polls();

    drop(c);
    asked
}

/// `Basic` imported with an argument that names nothing, so every type it declares is missing,
/// and `count` structs that use one and extend each other through `#as`.
fn program(count: usize) -> String {
    let mut source = String::from(
        "#import \"Basic\"()(MEMORY_DEBUGGER=NOT_DEFINED);\n\
         Base :: struct { time: Apollo_Time; }\n",
    );

    for i in 0..count {
        let parent = if i == 0 {
            "Base".to_owned()
        } else {
            format!("Child_{}", i - 1)
        };

        source += &format!(
            "Child_{i} :: struct {{ using #as base: {parent}; n: int; }}\n\
             use_{i} :: (c: *Child_{i}) -> int {{ return c.n + c.base.n; }}\n"
        );
    }

    source += "main :: () {}\n";
    source
}

#[test]
fn a_struct_that_failed_to_lay_out_is_not_laid_out_again_for_every_lookup() {
    // Four times the structs ask less than three times the questions: the failure is found once
    // and reused, not rediscovered by each struct and lookup that meets it.
    let small = polls(&program(10));
    let large = polls(&program(40));

    assert!(
        large < small * 3,
        "{small} questions for 10 structs, {large} for 40"
    );
}

#[test]
fn the_audit_of_every_body_stops_at_the_limit() {
    let stdlib = test_support::repo::root().join("stdlib");
    let mut options = Options::host();

    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let host = Rc::new(RefCell::new(SandboxHost::default()));

    c.ct.interp.set_host(Box::new(SharedHost(host.clone())));
    c.ed.record(IdeFacts::new(vec![IdeRoot::Virtual]));
    let _ = c.begin_sources(&[ProgramSource::String(program(5))]);
    let _ = c.finish_program();
    // The limit passes as the audit starts: it must not go on to check every body.
    let asked = host.borrow().polls();

    host.borrow_mut().stop_after_polls(asked);
    let errors = c.ide_check_all();

    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("checking stopped")),
        "{errors:?}"
    );
}
