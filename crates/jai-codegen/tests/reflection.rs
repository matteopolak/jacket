//! Compare independent VM storage with only this compiler's generated native code.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn check(source: &str, expected: i32) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-native-reflection-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&scratch.0).unwrap();
    let path = scratch.0.join("main.jai");
    fs::write(&path, source).unwrap();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let options = jai_sema::ResolveOptions {
        target: Some(target.build_target().unwrap()),
        layout: Some(target.layout_policy().unwrap()),
        ..jai_sema::ResolveOptions::default()
    };
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    let program =
        jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("VM reflection outcome: {outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), i128::from(expected));
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let object = scratch.0.join("program.o");
    target.write_object(&module, &object).unwrap();
    let executable = scratch.0.join("program");
    let compiled = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&compiled.stderr),
        module.print_to_string()
    );
    let mut child = Command::new(&executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(expected));
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated reflection fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn canonical_descriptor_relocations_match_direct_and_nested_queries() {
    for queries in [
        "direct:=type_info(Pair); pointer:=type_info(*Pair); outer:=type_info(Wrap);",
        "pointer:=type_info(*Pair); outer:=type_info(Wrap); direct:=type_info(Pair);",
    ] {
        check(
            &format!(
                "Pair::struct{{value:int;}} Wrap::struct{{pair:Pair;}} main::()->int{{ {queries} if cast(*void)pointer.pointer_to != cast(*void)direct return 1; if cast(*void)outer.members[0].type != cast(*void)direct return 2; return 42; }}"
            ),
            42,
        );
    }
}

#[test]
fn cyclic_descriptors_and_nonzero_array_relocations_are_readable() {
    check(
        "Node::struct { next:*Node; value:int; } main::()->int { info:=type_info(Node); pointer:=cast(*Type_Info_Pointer)info.members[0].type; if cast(*void)pointer.pointer_to != cast(*void)info return 1; if info.members[1].name[0] != #char \"v\" return 2; return info.members[1].offset_in_bytes+34; }",
        42,
    );
}

#[test]
fn source_notes_and_initializer_defaults_survive_native_lowering() {
    check(
        "Pair::struct { value:int=42; @JsonIgnore } main::()->int { info:=type_info(Pair); if info.members[0].notes.count != 1 return 1; if info.members[0].notes[0].count != 10 return 2; if info.members[0].notes[0][4] != #char \"I\" return 3; pair:=initializer_of(Pair); return pair.value; }",
        42,
    );
}

#[test]
fn captured_code_insertion_preserves_lexical_storage_in_native_code() {
    check(
        "main::()->int { x:=40; slot::#code x; code::#code x+2; { x:=100; (#insert slot)=40; return #insert code; } }",
        42,
    );
}

#[test]
fn expanded_code_places_and_caller_exports_share_native_storage() {
    check(
        "total:int; swap::(a:Code,b:Code)#expand { t:=(#insert a); (#insert a)=(#insert b); (#insert b)=t; } apply::(body:Code)#expand { `answer:=2; #insert body; } main::()->int { a:=2; b:=4; t:=100; swap(a,b); apply(#code { total=a*10+answer; }); return total+(t-100); }",
        42,
    );
}

#[test]
fn runtime_types_are_values_in_locals_arguments_and_results() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-basics.jai"),
        42,
    );
}

#[test]
fn runtime_types_survive_record_and_array_storage() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-storage.jai"),
        42,
    );
}

#[test]
fn runtime_type_descriptor_casts_retain_canonical_identity() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-descriptors.jai"),
        42,
    );
}

#[test]
fn runtime_type_conditionals_choose_real_descriptor_pointer_values() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-conditionals.jai"),
        42,
    );
}

#[test]
fn runtime_type_values_follow_c_calling_convention() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-c-calls.jai"),
        42,
    );
}

#[test]
fn compile_time_type_results_publish_canonical_runtime_cells() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-run.jai"),
        42,
    );
}

#[test]
fn initialized_type_globals_keep_canonical_descriptor_relocations() {
    check(
        include_str!("../../jai-sema/tests/fixtures/runtime-type-globals.jai"),
        42,
    );
}

#[test]
fn field_type_annotations_keep_inline_enum_identity_in_native_calls() {
    check(
        include_str!("../../jai-sema/tests/fixtures/type-of-field-annotations.jai"),
        42,
    );
}

#[test]
fn discarded_macro_inputs_have_no_native_argument_evaluations() {
    check(
        include_str!("../../jai-sema/tests/fixtures/discarded-macro-inputs.jai"),
        42,
    );
}

#[test]
fn local_macro_formals_and_captured_storage_keep_their_defining_scope() {
    check(
        "main::()->int { Pair::struct { value:s32; } total:=0; OFFSET::2; apply::(pair:Pair)#expand { total=cast(int)pair.value+OFFSET; } value:Pair; value.value=40; { OFFSET::100; Pair::struct { other:int; } apply(value); } return total; }",
        42,
    );
}

#[test]
fn caller_defer_restores_allocator_procedure_values_after_macro_scope() {
    check(
        include_str!("../../jai-sema/tests/fixtures/caller-defer-allocator.jai"),
        42,
    );
}

#[test]
fn caller_defer_runs_in_reverse_order_on_return() {
    check(
        include_str!("../../jai-sema/tests/fixtures/caller-defer-lifecycle.jai"),
        42,
    );
}

#[test]
fn caller_defer_restores_the_pushed_context_before_context_exit() {
    check(
        include_str!("../../jai-sema/tests/fixtures/caller-defer-context.jai"),
        42,
    );
}
