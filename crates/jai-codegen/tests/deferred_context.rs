//! Source-scoped context suffixes execute through genuine cleanup/context IR.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn check(source: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-deferred-context-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let cleanup = Cleanup(directory);
    let input = cleanup.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph = ModuleGraph::load(&input, GraphOptions::default()).unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    let llvm = jai_codegen::emit(&program).unwrap();
    let ir = cleanup.0.join("program.ll");
    fs::write(&ir, llvm).unwrap();
    for optimization in ["-O0", "-O2"] {
        let executable = cleanup.0.join(optimization);
        let output = native_tools::clang_command()
            .arg(optimization)
            .arg(&ir)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(42));
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("context fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn deferred_context_cleanups_capture_their_actual_activation_context() {
    check(
        "#add_context number:int=1; trace:int;
        helper::(){context.number=2; defer trace=trace*10+context.number;
            push_context,defer_pop; context.number=4; defer trace=trace*10+context.number;}
        main::()->int{helper();return trace;}",
    );
}

#[test]
fn deferred_context_keeps_original_local_declaration_scope_and_forward_aliases() {
    check("#add_context number:int=2;
        helper::()->int #no_context {value:T=40; push_context,defer_pop; T::int; return value+context.number;}
        main::()->int{return helper();}");
}

#[test]
fn deferred_context_does_not_redirect_a_captured_outer_field_pointer() {
    check(
        "#add_context number:int=2;
        helper::()->int{context.number=40; p:=*context.number; push_context,defer_pop;
            context.number=9; p.*+=2; defer context.number=0; return p.*;}
        main::()->int{return helper();}",
    );
}

#[test]
fn deferred_context_restores_on_loop_continue_and_break_after_cleanups() {
    check(
        "#add_context number:int=1;
        main::()->int{context.number=40; total:int;
            for i:0..3 {push_context,defer_pop; context.number=i;
                defer total+=context.number; if i<2 continue; break;}
            return context.number+total-1;}",
    );
}

#[test]
fn nested_deferred_context_records_are_copies_and_outer_cleanup_runs_after_inner() {
    check("#add_context number:int=1; trace:int;
        helper::(){saved:=context; saved.number=40; push_context,defer_pop saved;
            defer trace+=context.number; push_context,defer_pop; context.number=2; defer trace+=context.number;}
        main::()->int{helper();return trace;}");
}
