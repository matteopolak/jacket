//! Exercise the same retained source body in the VM and newly compiled native code.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn check(source: &str, vm: Option<i128>, native: i32) {
    check_with_layout(source, vm, native, None);
}

fn check_with_layout(
    source: &str,
    vm: Option<i128>,
    native: i32,
    layout: Option<jai_types::LayoutPolicy>,
) {
    check_native(source, vm, Some(native), layout);
}

fn check_native(
    source: &str,
    vm: Option<i128>,
    native: Option<i32>,
    layout: Option<jai_types::LayoutPolicy>,
) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-execution-phase-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&scratch.0).unwrap();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
    let options = jai_sema::ResolveOptions {
        layout,
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let program =
        jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let outcome = jai_vm::execute(&program, Default::default()).outcome;
    match vm {
        Some(expected) => {
            let jai_vm::Outcome::Complete(values) = outcome else {
                panic!("{outcome:?}");
            };
            assert_eq!(values[0].integer().unwrap().value(), expected);
        }
        None => {
            assert!(matches!(outcome, jai_vm::Outcome::Failed(_)), "{outcome:?}");
            // A source #run uses the real compiler catalog, including its debug-break trap.
            let trapped = scratch.0.join("trapped.jai");
            fs::write(&trapped, format!("{source}\nTRAPPED :: #run main();")).unwrap();
            let graph = jai_modules::ModuleGraph::load(&trapped, Default::default()).unwrap();
            let options = jai_sema::ResolveOptions {
                layout,
                compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                    &graph,
                    &[],
                    jai_vm::WorkspaceId::from_raw(1).unwrap(),
                )),
                ..Default::default()
            };
            let error =
                jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects)
                    .unwrap_err();
            assert!(error.message.contains("trap"), "{error:?}");
        }
    }
    let ir = jai_codegen::emit(&program).unwrap();
    assert!(!ir.contains("compile_time_debug_break"));
    let llvm = scratch.0.join("program.ll");
    let executable = scratch.0.join("program");
    fs::write(&llvm, ir).unwrap();
    for optimization in ["-O0", "-O2"] {
        let output = native_tools::clang_command()
            .arg(&llvm)
            .arg(optimization)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if let Some(native) = native {
            assert_eq!(
                Command::new(&executable).status().unwrap().code(),
                Some(native)
            );
        } else {
            // Execute only our freshly generated program, with bounded observation.
            let mut child = Command::new(&executable).spawn().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(40));
            let stopped = child.try_wait().unwrap();
            if stopped.is_none() {
                child.kill().unwrap();
            }
            child.wait().unwrap();
            assert!(
                stopped.is_none(),
                "native phase loop unexpectedly completed: {stopped:?}"
            );
        }
    }
}

#[test]
fn retained_procedure_and_run_result_observe_their_own_execution_phases() {
    check(include_str!("fixtures/execution-phase.jai"), Some(41), 42);
}

#[test]
fn native_unselected_compiler_intrinsic_branch_is_never_lowered() {
    check(
        include_str!("fixtures/execution-phase-compiler-call.jai"),
        None,
        42,
    );
}

#[test]
fn identity_storage_cast_keeps_phase_selection_and_native_dead_intrinsics() {
    check_with_layout(
        "compile_time_debug_break :: () #compiler #no_context; main :: ()->int { if cast,force(bool) #compile_time {compile_time_debug_break();} return 42; }",
        None,
        42,
        Some(jai_types::LayoutPolicy::lp64()),
    );
}

#[test]
fn all_typed_conditional_arms_and_nested_short_circuits_prune_phase_only_calls() {
    check(
        r#"
        compile_time_debug_break :: () #compiler #no_context;
        compiler_int :: ()->int {compile_time_debug_break(); return 1;}
        compiler_bool :: ()->bool {compile_time_debug_break(); return true;}
        compiler_float :: ()->float64 {compile_time_debug_break(); return 1.0;}
        compiler_string :: ()->string {compile_time_debug_break(); return "compiler";}
        counter:int=0;
        increment :: ()->bool {counter+=1;return true;}
        main :: ()->int {
            a := ifx #compile_time then compiler_int() else 42;
            b := ifx #compile_time then compiler_bool() else true;
            c := ifx #compile_time then compiler_float() else 42.0;
            d := ifx #compile_time then compiler_string() else "native";
            if (increment() && #compile_time) && compiler_bool() return 1;
            if counter!=1 return 2;
            if (increment() || !#compile_time) || compiler_bool() {counter+=1;}
            if counter!=3 return 3;
            if (increment() && #compile_time) == false {counter+=1;} else {compiler_bool();}
            if counter!=5 return 5;
            if !b || c!=42.0 || d!="native" return 4;
            return a;
        }
    "#,
        None,
        42,
    );
}

#[test]
fn semantic_guards_evaluate_phase_in_the_vm_without_changing_runtime_phase() {
    check(
        r#"
        main :: ()->int {
            #if #compile_time {selected:=42;} else {unavailable();}
            if #compile_time return 41;
            return selected;
        }
    "#,
        Some(41),
        42,
    );
}

#[test]
fn annotated_compile_time_source_bodies_run_in_vm_and_remain_out_of_native_demand() {
    check(
        r#"
        evaluate :: (x:int)->int #compile_time { return x+1; }
        VALUE :: #run evaluate(41);
        main :: ()->int {
            if #compile_time return evaluate(40);
            return VALUE;
        }
    "#,
        Some(41),
        42,
    );
}

#[test]
fn compile_time_only_callable_aliases_cannot_enter_native_publication() {
    for source in [
        "only :: ()->int #compile_time {return 42;} main :: ()->int {return only();}",
        "only :: ()->int #compile_time {return 42;} alias :: only; main :: ()->int {return alias();}",
        "main :: ()->int {only :: ()->int #compile_time {return 42;} callback:=only; return callback();}",
    ] {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let input = std::env::temp_dir().join(format!(
            "jai-phase-reject-{}-{}.jai",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&input, source).unwrap();
        let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
        let program = jai_sema::resolve_graph(&graph).unwrap();
        let outcome = jai_vm::execute(&program, Default::default()).outcome;
        assert!(
            matches!(outcome, jai_vm::Outcome::Complete(_)),
            "{outcome:?}"
        );
        let error = jai_codegen::emit(&program).unwrap_err();
        assert!(
            matches!(error, jai_codegen::Error::Reachability(_)),
            "{error:?}"
        );
        fs::remove_file(input).unwrap();
    }
}

#[test]
fn phase_transfers_prune_later_calls_and_inactive_while_bodies() {
    check(
        r#"
        compile_time_debug_break :: () #compiler #no_context;
        main :: ()->int {
            while #compile_time {compile_time_debug_break();}
            if !#compile_time return 42;
            compile_time_debug_break();
            return 1;
        }
    "#,
        None,
        42,
    );
}

#[test]
fn case_through_targets_respect_phase_selected_terminators() {
    check(
        r#"
        compile_time_debug_break :: () #compiler #no_context;
        main :: ()->int {
            n:=1;
            if n == {
              case 1; #through;
              case 2; if !#compile_time return 42;
              case; if !#compile_time return 42;
            }
            compile_time_debug_break();
            return 1;
        }
    "#,
        None,
        42,
    );
}

#[test]
fn native_phase_infinite_loops_exclude_unreachable_compiler_dependencies() {
    for body in [
        "while !#compile_time {}",
        "while !#compile_time {if #compile_time break;}",
        "while !#compile_time {while true {break;}}",
        "while !#compile_time {while #compile_time {break;} continue;}",
        "while !#compile_time {if !#compile_time continue; break;}",
    ] {
        check_native(
            &format!(
                "compile_time_debug_break::() #compiler #no_context; main::()->int {{{body} compile_time_debug_break(); return 42;}}"
            ),
            None,
            None,
            None,
        );
    }
}

#[test]
fn reachable_loop_breaks_preserve_native_continuations_and_condition_effects() {
    check(
        r#"
        counter:int=0;
        tick::()->bool {counter+=1; return false;}
        main::()->int {
            while tick() || !#compile_time {
                while true {break;}
                if counter==3 break;
            }
            if #compile_time return counter+41;
            return counter+39;
        }
    "#,
        Some(42),
        42,
    );
    check(
        r#"
        main::()->int {
            n:=0;
            while outer:=!#compile_time {
                while true {n+=1; break outer;}
            }
            if #compile_time return 42;
            return n+41;
        }
    "#,
        Some(42),
        42,
    );
}

#[test]
fn possible_and_outer_target_breaks_keep_compiler_calls_in_native_demand() {
    for body in [
        "while !#compile_time {break;}",
        "while !#compile_time {if runtime() break;}",
        "while outer:=!#compile_time {while true {break outer;}}",
        "while !#compile_time {if !#compile_time break;}",
    ] {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let input = std::env::temp_dir().join(format!(
            "jai-phase-loop-reject-{}-{}.jai",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&input,format!("only::()->int #compile_time {{return 42;}} runtime::()->bool {{return true;}} main::()->int {{{body} return only();}}" )).unwrap();
        let graph = jai_modules::ModuleGraph::load(&input, Default::default()).unwrap();
        let program = jai_sema::resolve_graph(&graph).unwrap();
        let outcome = jai_vm::execute(&program, Default::default()).outcome;
        let jai_vm::Outcome::Complete(values) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(values[0].integer().unwrap().value(), 42);
        assert!(matches!(
            jai_codegen::emit(&program),
            Err(jai_codegen::Error::Reachability(_))
        ));
        fs::remove_file(input).unwrap();
    }
}
