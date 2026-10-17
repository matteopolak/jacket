//! Conditional exports, import cycles and configured runtime identity.
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::{intern::Sym, sema::scope::EntityState};
use std::{path::Path, rc::Rc};

fn compile(files: &[(&str, &str)], runtime: bool) -> Compiler {
    let mut fs = VirtualFs::default();
    for (path, text) in files {
        fs.insert(*path, text.as_bytes());
    }
    let mut options = Options::host();
    options.runtime_support = runtime;
    options.runtime_entry_point = false;
    options.runtime_initialization = false;
    options.backtrace_on_crash = false;
    options.import_paths = vec!["/lib".into()];
    Compiler::new(options, Rc::new(fs))
}

#[test]
fn readiness_probe_does_not_discard_an_imported_invalid_condition() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/error-preservation/invalid/main.jai"
                ),
            ),
            (
                "/app/flags.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/error-preservation/invalid/flags.jai"
                ),
            ),
        ],
        false,
    );
    let error = compiler.compile_program(Path::new("/app/main.jai")).expect_err("a readiness probe must not discard unknown-name errors from imported pending conditions");
    assert!(
        error.message.contains("unknown identifier `missing_flag`"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn acyclic_imported_conditional_flag_becomes_ready() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/error-preservation/valid-exported/main.jai"
                ),
            ),
            (
                "/app/flags.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/error-preservation/valid-exported/flags.jai"
                ),
            ),
            (
                "/app/config.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/error-preservation/valid-exported/config.jai"
                ),
            ),
        ],
        false,
    );
    if let Err(error) = compiler.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn readiness_probe_does_not_discard_an_imported_undeclared_call() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                "Flags :: #import \"Flags\"; #if Flags.ready { selected :: true; } #assert selected; main :: () {}",
            ),
            (
                "/lib/Flags/module.jai",
                "ready :: true; #if undeclared_predicate() { unused :: 42; }",
            ),
        ],
        false,
    );
    let error = compiler
        .compile_program(Path::new("/app/main.jai"))
        .expect_err("undeclared calls cannot disappear inside readiness probes");
    assert!(
        error
            .message
            .contains("unknown identifier `undeclared_predicate`"),
        "{}",
        error.render(&compiler.sources)
    );
}

#[test]
fn readiness_constant_run_effect_is_not_replayed() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                "Flags :: #import \"Flags\"; #if Flags.ready { selected :: 42; } #assert selected==42; #assert #run Flags.called_once(); main :: () {}",
            ),
            (
                "/lib/Flags/module.jai",
                "calls:int; ready :: #run compute_ready(); compute_ready :: ()->bool {calls+=1; return true;} called_once :: ()->bool {return calls==1;}",
            ),
        ],
        false,
    );
    if let Err(error) = compiler.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn readiness_constant_run_can_depend_on_pending_declarations() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                "Flags :: #import \"Flags\"; #if Flags.ready { selected :: true; } #assert selected; main :: () {}",
            ),
            (
                "/lib/Flags/module.jai",
                "ready :: #run compute_ready(); compute_ready :: ()->bool {return helper;} #if true {helper :: true;}",
            ),
        ],
        false,
    );
    if let Err(error) = compiler.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn readiness_constant_run_failure_is_terminal() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                "Flags :: #import \"Flags\"; #if Flags.ready { selected :: true; } main :: () {}",
            ),
            (
                "/lib/Flags/module.jai",
                "ready :: #run compute_ready(); compute_ready :: ()->bool {while true {} return true;}",
            ),
        ],
        false,
    );
    compiler.interp.block_budget = Some(32);
    let error = compiler
        .compile_program(Path::new("/app/main.jai"))
        .expect_err("executed constant failure must be retained");
    assert_eq!(compiler.ct_traps, 1);
    let flags = compiler
        .modules
        .iter()
        .find(|module| module.name == "Flags")
        .unwrap();
    let ready = compiler.scope(flags.scope).names[&Sym::intern("ready")][0];
    assert!(matches!(
        compiler.entity(ready).state,
        EntityState::Failed(_)
    ));
    let repeated = compiler.resolve_entity(ready).unwrap_err();
    assert_eq!(repeated.message, error.message);
    assert_eq!(
        compiler.ct_traps, 1,
        "diagnostic reuse must not execute the failing constant again"
    );
}

#[test]
fn pending_insert_fulfills_placeholder_before_conditional_commit() {
    let mut compiler = compile(
        &[(
            "/app/main.jai",
            "#placeholder ready; #insert \"ready :: true;\"; #if ready { selected :: 42; } #assert selected==42; main :: () {}",
        )],
        false,
    );
    if let Err(error) = compiler.compile_program(Path::new("/app/main.jai")) {
        panic!("{}", error.render(&compiler.sources));
    }
}

#[test]
fn officially_rejected_conditional_cycle_remains_an_error() {
    let mut compiler = compile(
        &[
            (
                "/app/main.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/conditional-module-dependency/main.jai"
                ),
            ),
            (
                "/app/consumer.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/conditional-module-dependency/consumer.jai"
                ),
            ),
            (
                "/app/config.jai",
                include_str!(
                    "../../../tests/fixtures/readiness/conditional-module-dependency/config.jai"
                ),
            ),
        ],
        false,
    );
    assert!(
        compiler
            .compile_program(Path::new("/app/main.jai"))
            .is_err(),
        "officially rejected cycle must not become a positive"
    );
}
