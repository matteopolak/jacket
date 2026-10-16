//! Original lexical context/cleanup regressions independent of stdlib bodies.
use jaic::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

const SETUP: &str = r#"
#add_context marker: s64 = 7;
push_copy :: () #expand { copied := context; `push_context,defer_pop copied; }
"#;

fn check(source: &str) {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut c = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(error) = c.compile_sources(&[ProgramSource::String(format!("{SETUP}{source}"))]) {
        panic!("{}", error.render(&c.sources));
    }
}

#[test]
fn overload_probes_initialize_scratch_context_storage() {
    check(
        "consume :: (value: int) #expand {} consume :: (value: string) #expand {} make :: () -> int { return context.marker; } test :: () { consume(ifx true then make() else 0); } #run test();",
    );
}

#[test]
fn call_context_record_callback_override_reaches_nested_calls_and_restores() {
    check(
        r#"
        #add_context settings: Settings;
        Settings :: struct { callback:=ordinary; }
        ordinary :: ()->s64 {return 1;}
        custom :: ()->s64 {return 9;}
        nested :: ()->s64 {return context.settings.callback();}
        read :: ()->s64 {return nested();}
        test :: ()->bool {
            local:Settings=.{callback=custom};
            if local.callback()!=9 return false;
            if read()!=1 return false;
            if read(,, settings=.{callback=custom})!=9 return false;
            return read()==1;
        }
        #assert test();
    "#,
    );
}

#[test]
fn caller_scope_macro_context_lives_until_caller_block_exit() {
    check(
        "test :: () -> bool { context.marker = 55; { push_copy(); if context.marker != 55 return false; context.marker = 91; if context.marker != 91 return false; } return context.marker == 55; } #assert(test());",
    );
}

#[test]
fn nested_caller_contexts_restore_in_lexical_order() {
    check(
        "test :: () -> bool { context.marker = 55; { push_copy(); context.marker = 91; { push_copy(); context.marker = 123; } if context.marker != 91 return false; } return context.marker == 55; } #assert(test());",
    );
}

#[test]
fn defers_run_in_pushed_context_before_its_pop() {
    check(
        "observed: int; test :: () -> bool { context.marker = 55; { push_copy(); defer observed = context.marker; context.marker = 91; } return observed == 91 && context.marker == 55; } #assert(test());",
    );
    check(
        "observed: int; test :: () -> bool { context.marker = 55; { push_context,defer_pop; defer observed = context.marker; context.marker = 91; } return observed == 91 && context.marker == 55; } #assert(test());",
    );
}

#[test]
fn return_and_loop_control_restore_context_before_outer_defers() {
    check(
        "observed: int; test :: () -> int { context.marker = 55; defer observed = context.marker; { push_copy(); context.marker = 91; return context.marker; } } #assert(test() == 91); #assert(observed == 55);",
    );
    check(
        "test :: () -> bool { context.marker = 55; for i: 0..3 { if context.marker != 55 return false; push_copy(); context.marker = 91; if i < 2 continue; break; } return context.marker == 55; } #assert(test());",
    );
}

#[test]
fn macro_local_context_does_not_escape_its_expansion() {
    check(
        "local_push :: () #expand { push_context,defer_pop; context.marker = 99; } test :: () -> bool { context.marker = 55; local_push(); return context.marker == 55; } #assert(test());",
    );
}

#[test]
fn caller_owned_cleanup_survives_nested_macro_blocks() {
    check(
        "nested_push :: () #expand { { copied := context; `push_context,defer_pop copied; } } test :: () -> bool { context.marker = 55; { nested_push(); context.marker = 91; } return context.marker == 55; } #assert(test());",
    );
    check(
        "observed: int; install :: () #expand { { `defer observed = context.marker; } } test :: () -> bool { context.marker = 55; { install(); context.marker = 91; if observed != 0 return false; } return observed == 91; } #assert(test());",
    );
}

#[test]
fn conditional_macro_cleanup_only_runs_when_registered() {
    check(
        "conditional :: (enabled: bool) #expand { if enabled { copied := context; `push_context,defer_pop copied; `defer observed += context.marker; } } observed: int; test :: (enabled: bool) -> bool { context.marker = 55; { conditional(enabled); context.marker = 91; } return context.marker == (ifx enabled then 55 else 91); } #assert(test(false)); #assert(observed == 0); #assert(test(true)); #assert(observed == 91);",
    );
}

#[test]
fn explicit_context_blocks_restore_before_outer_exit_cleanup() {
    check(
        "observed: int; test :: () -> int { context.marker = 55; defer observed = context.marker; push_context { context.marker = 91; return context.marker; } } #assert(test() == 91); #assert(observed == 55);",
    );
}

#[test]
fn context_is_initialized_in_speculative_overload_checking() {
    check(
        "value :: () -> int { return context.marker; } take :: (x: int) -> int { return x; } take :: (x: bool) -> int { return 99; } test :: () -> bool { context.marker = 55; return take(value()+1) == 56; } #assert(test()); #assert(!is_constant(value()));",
    );
}
