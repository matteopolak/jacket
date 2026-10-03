//! Custom iteration follows user-defined source macro bodies, never name intrinsics.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const FLAGS: &str = "For_Flags :: enum_flags u8 { POINTER :: 1; REVERSE :: 2; } ";

fn compile(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    compile_with_files(source, &[])
}

fn compile_with_files(
    source: &str,
    files: &[(&str, &str)],
) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-custom-for-source-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let input = root.join("main.jai");
    fs::write(&input, format!("{FLAGS}{source}")).unwrap();
    for (name, text) in files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let graph = jai_modules::ModuleGraph::load(
        &input,
        jai_modules::GraphOptions {
            import_dirs: vec![root.clone()],
        },
    )
    .unwrap();
    let result = jai_sema::resolve_graph(&graph);
    fs::remove_dir_all(root).unwrap();
    result
}

fn execute(source: &str, expected: i32) {
    execute_with_files(source, &[], expected);
}

fn execute_with_files(source: &str, files: &[(&str, &str)], expected: i32) {
    let program = compile_with_files(source, files).unwrap();
    match jai_vm::execute(&program, jai_vm::Limits::default()).outcome {
        jai_vm::Outcome::Complete(values) => {
            assert_eq!(values[0].integer().unwrap().value(), i128::from(expected))
        }
        other => panic!("VM failed: {other:?}"),
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-custom-for-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let context = jai_codegen::Context::create();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    let object = root.join("program.o");
    target.write_object(&module, &object).unwrap();
    let executable = root.join("program");
    let result = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stderr),
        module.print_to_string()
    );
    let mut process = Command::new(&executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            assert_eq!(status.code(), Some(expected));
            break;
        }
        if Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated custom iteration fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sparse_record_custom_loop_uses_source_body_and_remaps_both_iterators() {
    execute(
        "Holder :: struct { occupied: [4]bool; values: [4]int; } for_expansion :: (holder: Holder, body: Code, flags: For_Flags) #expand { for ok,slot: holder.occupied { if !ok continue; `it := holder.values[slot]; `it_index := slot; #insert body; } } main :: () -> int { holder: Holder = .{occupied=.[false,true,false,true],values=.[1,20,3,20]}; sum := 0; for value,index: holder { sum += value + index; } return sum - 2; }",
        42,
    );
}

#[test]
fn named_expansion_override_and_extra_export_preserve_caller_lexical_storage() {
    execute(
        "Container :: struct { value: int; } positive :: (source: Container, body: Code, flags: For_Flags) #expand { `visits := 0; for slot: 0..1 { `it := source.value; `it_index := slot; defer visits += 1; #insert body; } } main :: () -> int { source: Container = .{value=20}; sum := 0; for :positive value,index: source { sum += value + index + visits; } return sum; }",
        42,
    );
}

#[test]
fn pointer_source_parameter_mutates_original_record_and_flags_are_typed() {
    execute(
        "Container :: struct { value: int; } for_expansion :: (source: *Container, body: Code, flags: For_Flags) #expand { source.value += 2; `it := source.value; `it_index := 0; if (flags & .POINTER) != 0 it += 1; if (flags & .REVERSE) != 0 it += 1; #insert body; } main :: () -> int { source: Container = .{value=18}; sum := 0; for < * source { sum += it; } return source.value + sum; }",
        42,
    );
}

#[test]
fn temporary_source_runs_once_and_nested_body_defers_keep_their_caller_binding() {
    execute(
        "Container :: struct { value: int; } calls := 0; make :: () -> Container { calls += 1; return .{value=20}; } for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { for slot: 0..1 { `it := source.value; `it_index := slot; #insert body; } } main :: () -> int { sum := 0; for make() { defer sum += 1; sum += it; } return sum + calls - 1; }",
        42,
    );
}

#[test]
fn source_expansion_forwards_real_flags_to_controlled_array_modifiers() {
    execute(
        "Container :: struct { values: [2]int; } for_expansion :: (source: *Container, body: Code, flags: For_Flags) #expand { `it := 0; `it_index := 0; for *=(flags & .POINTER != 0) <=(flags & .REVERSE != 0) value,index: source.values { it_index = index; it = value.*; value.* += 1; #insert body; } } main :: () -> int { source: Container = .{values=.[1,2]}; number := 0; for < * source { number = number*10 + it; } return number + source.values[0] + source.values[1] + 16; }",
        42,
    );
}

#[test]
fn direct_exported_array_iterators_preserve_aliases_and_remap_names() {
    execute(
        "Container :: struct { values: [2]int; } for_expansion :: (source: *Container, body: Code, flags: For_Flags) #expand { for `it, `it_index: source.values { #insert body; } } main :: () -> int { source: Container = .{values=.[20,21]}; sum := 0; for value,index: source { sum += value + index; } return sum; }",
        42,
    );
}

#[test]
fn direct_exported_range_iterator_preserves_the_named_caller_binding() {
    execute(
        "Container :: struct {} for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { for `it: 20..21 { `it_index := it - 20; #insert body; } } main :: () -> int { source: Container; sum := 0; for value,index: source sum += value + index; return sum; }",
        42,
    );
}

#[test]
fn explicit_selector_overrides_builtin_iteration_with_real_source_body() {
    execute(
        "walk :: (source: [2]int, body: Code, flags: For_Flags) #expand { for slot: 0..1 { `it := source[slot]; `it_index := 1-slot; #insert body; } } main :: () -> int { source: [2]int = .[20,21]; sum := 0; for :walk value,index: source sum += value + index; return sum; }",
        42,
    );
}

#[test]
fn pointer_sources_auto_dereference_for_a_value_parameter_without_copying() {
    execute(
        "Container :: struct { value: int; } for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { source.value += 2; `it := source.value; `it_index := 0; #insert body; } main :: () -> int { source: Container = .{value=18}; pointer := *source; sum := 0; for pointer sum += it; return sum + source.value + 2; }",
        42,
    );
}

#[test]
fn inactive_macro_returns_are_skipped_and_inserted_caller_returns_are_preserved() {
    execute(
        "Container :: struct { value: int; } for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { #if false { return; } else { `it := source.value; `it_index := 0; #insert body; } } main :: () -> int { source: Container = .{value=42}; for source return it; }",
        42,
    );
}

#[test]
fn namespaced_expansion_keeps_definition_and_caller_file_bindings_distinct() {
    execute_with_files(
        "API :: #import \"iterator\"; OFFSET :: 22; main :: () -> int { source: API.Container = .{value=20}; sum := 0; for :API.walk value,index: source { sum += value + OFFSET; } return sum - 1; }",
        &[(
            "iterator.jai",
            "For_Flags :: enum_flags u8 { POINTER :: 1; REVERSE :: 2; } Container :: struct { value: int; } OFFSET :: 1; walk :: (source: Container, body: Code, flags: For_Flags) #expand { `it := source.value + OFFSET; `it_index := 0; #insert body; }",
        )],
        42,
    );
}

#[test]
fn scoped_namespace_selects_an_imported_source_expansion() {
    execute_with_files(
        "main :: () -> int { API :: #import \"iterator\"; source: API.Container = .{value=40}; sum := 0; for :API.walk value,index: source { sum += value + index; } return sum; }",
        &[(
            "iterator.jai",
            "For_Flags :: enum_flags u8 { POINTER :: 1; REVERSE :: 2; } Container :: struct { value: int; } walk :: (source: Container, body: Code, flags: For_Flags) #expand { `it := source.value; `it_index := 2; #insert body; }",
        )],
        42,
    );
}

#[test]
fn lexical_expansion_uses_local_nominal_types_and_direct_iterator_exports() {
    execute(
        "main :: () -> int { Container :: struct { values: [2]int; } for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { for `it, `it_index: source.values { #insert body; } } source: Container = .{values=.[20,21]}; sum := 0; for value,index: source sum += value + index; return sum; }",
        42,
    );
}

#[test]
fn lexical_expansion_retains_its_definition_scope_while_body_keeps_caller_shadow() {
    execute(
        "Container :: struct { value: int; } main :: () -> int { OFFSET :: 2; walk :: (source: Container, body: Code, flags: For_Flags) #expand { `it := source.value + OFFSET; `it_index := 0; #insert body; } source: Container = .{value=20}; sum := 0; { OFFSET :: 20; for :walk source sum += it + OFFSET; } return sum; }",
        42,
    );
}

#[test]
fn readonly_sequence_elements_can_be_read_through_custom_value_aliases() {
    execute(
        "Container :: struct { value: int; } for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { `it := source.value; `it_index := 0; #insert body; } main :: () -> int { sources: [2]Container = .[.{value=20},.{value=22}]; sum := 0; for outer: sources { for value,index: outer { sum += value; } } return sum; }",
        42,
    );
}

#[test]
fn readonly_source_aliases_cannot_be_modified_or_passed_as_mutable_pointers() {
    for source_parameter in ["Container", "*Container"] {
        let source = format!(
            "Container :: struct {{ value: int; }} for_expansion :: (source: {source_parameter}, body: Code, flags: For_Flags) #expand {{ source.value = 1; `it := source.value; `it_index := 0; #insert body; }} main :: () -> int {{ sources: [1]Container; for outer: sources {{ for outer {{}} }} return 0; }}"
        );
        let error = compile(&source).unwrap_err();
        assert!(
            error
                .message
                .contains("a by-value array iterator is read-only"),
            "{}",
            error.message
        );
    }
}

#[test]
fn custom_break_and_continue_follow_inserted_source_loop_control() {
    execute(
        "Container :: struct { value: int; } for_expansion :: (source: Container, body: Code, flags: For_Flags) #expand { for slot: 0..3 { `it := source.value; `it_index := slot; #insert body; } } main :: () -> int { source: Container = .{value=20}; sum := 0; for source { if it_index == 1 continue; if it_index == 3 break; sum += it; } return sum + 2; }",
        42,
    );
}

#[test]
fn missing_or_malformed_source_expansions_report_the_correct_source_location() {
    for (source, message, token) in [
        (
            "Container :: struct {} main :: () -> int { source: Container; for source {} return 0; }",
            "no source for_expansion",
            "source",
        ),
        (
            "Container :: struct {} walk :: (source: Container) #expand {} main :: () -> int { source: Container; for :walk source {} return 0; }",
            "requires three parameters",
            "source",
        ),
        (
            "Container :: struct {} walk :: (source: Container, body: int, flags: For_Flags) #expand {} main :: () -> int { source: Container; for :walk source {} return 0; }",
            "second for expansion parameter must be Code",
            "source",
        ),
        (
            "Container :: struct {} walk :: (source: Missing, body: Code, flags: For_Flags) #expand {} main :: () -> int { source: Container; for :walk source {} return 0; }",
            "Missing",
            "source: Missing",
        ),
        (
            "Container :: struct {} walk :: (source: Container, body: Code, flags: For_Flags) #expand { `it := 1; #insert body; } main :: () -> int { source: Container; for :walk source {} return 0; }",
            "must export it and it_index",
            "source",
        ),
    ] {
        let full = format!("{FLAGS}{source}");
        let error = compile(source).unwrap_err();
        assert!(error.message.contains(message), "{}", error.message);
        assert_eq!(error.location.span.text(&full), token);
    }
}
