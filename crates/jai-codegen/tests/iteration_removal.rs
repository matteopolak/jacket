//! These programs are compiled by this Rust implementation and executed twice.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

fn program(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-iteration-source-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let input = root.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    let result = jai_sema::resolve_graph(&graph);
    fs::remove_dir_all(root).unwrap();
    result
}

fn execute(source: &str, expected: i32) {
    execute_expected(source, Expected::Exit(expected));
}

enum Expected {
    Exit(i32),
    Trap(jai_vm::Error),
}

fn execute_expected(source: &str, expected: Expected) {
    let program = program(source).unwrap();
    match (
        &expected,
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
    ) {
        (Expected::Exit(expected), jai_vm::Outcome::Complete(values)) => {
            assert_eq!(values[0].integer().unwrap().value(), i128::from(*expected))
        }
        (Expected::Trap(expected), jai_vm::Outcome::Failed(error)) => assert_eq!(&error, expected),
        (_, other) => panic!("VM failed: {other:?}"),
    }
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "jai-iteration-native-{}-{}",
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
            match &expected {
                Expected::Exit(expected) => assert_eq!(status.code(), Some(*expected)),
                Expected::Trap(_) => {
                    assert!(!status.success());
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        assert!(status.signal().is_some(), "{status:?}");
                    }
                }
            }
            break;
        }
        if Instant::now() >= deadline {
            process.kill().unwrap();
            process.wait().unwrap();
            panic!("generated iteration fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unordered_removal_revisits_swapped_elements_and_updates_dynamic_count() {
    execute(
        "main :: () -> int { backing: [6]int = .[2,4,5,6,7,8]; values: [..]int = .{data=*backing[0],count=6,allocated=6}; visited := 0; for value,index: values { visited += 1; if (value & 1) == 0 remove value; } return visited + values.count*10 + values[0] + values[1] + 4; }",
        42,
    );
}

#[test]
fn removal_then_continue_runs_user_defer_before_reusing_the_index() {
    execute(
        "main :: () -> int { backing: [4]int = .[2,4,5,7]; values: []int = backing; cleanup := 0; sum := 0; for value,index: values { defer cleanup += 1; if (value & 1) == 0 { remove value; continue; } sum += value + index; } return sum + cleanup + values.count*10 + 5; }",
        42,
    );
}

#[test]
fn reverse_removal_does_not_revisit_previously_visited_elements() {
    execute(
        "main :: () -> int { backing: [4]int = .[2,4,5,7]; values: []int = backing; visits := 0; sum := 0; for < value,index: values { visits += 1; if (value & 1) == 0 remove value; else sum += value; } return visits + sum + values.count*10 + 6; }",
        42,
    );
}

#[test]
fn descriptor_address_is_evaluated_once_and_removal_publishes_to_that_slot() {
    execute(
        "calls := 0; slot :: () -> int { calls += 1; return 0; } main :: () -> int { backing: [3]int = .[2,4,7]; descriptors: [1][]int; descriptors[0] = backing; for value: descriptors[slot()] { if (value & 1) == 0 remove value; } return calls + descriptors[0].count*10 + descriptors[0][0] + 24; }",
        42,
    );
}

#[test]
fn pointer_removal_rebinds_the_swapped_slot_for_the_next_iteration() {
    execute(
        "main :: () -> int { backing: [2]int = .[2,7]; values: []int = backing; for *pointer,index: values { if pointer.* == 2 remove pointer; else pointer.* += 35; } return values[0] + values.count - 1; }",
        42,
    );
}

#[test]
fn by_value_iterator_observes_storage_changes_and_pointer_iteration_writes() {
    execute(
        "Point :: struct { x: int; } main :: () -> int { values: [1]Point = .[.{x=1}]; result := 0; for value: values { values[0].x = 20; result = value.x; } for *value: values { value.x += 2; } return result + values[0].x; }",
        42,
    );
}

#[test]
fn record_array_fields_keep_live_storage_and_descriptor_removal_publishes_to_the_field() {
    execute(
        "Holder :: struct { values: [2]int; descriptor: []int; } main :: () -> int { backing: [3]int = .[2,20,21]; holder: Holder = .{values=.[20,0],descriptor=backing}; sum := 0; for value: holder.values { sum += value; holder.values[1] = 22; } for value: holder.descriptor if (value & 1) == 0 remove value; return sum + holder.descriptor.count - 1; }",
        42,
    );
}

#[test]
fn controlled_pointer_and_reverse_modifiers_are_compile_time_values() {
    execute(
        "REVERSE :: true; POINTER :: false; main :: () -> int { values: [2]int = .[1,2]; number := 0; for *=POINTER, <=REVERSE value: values { number = number*10 + value; } for *=true value: values { value.* += 1; } sum := 0; for <=false 1..3 sum += it; return number + values[0] + values[1] + sum + 10; }",
        42,
    );
}

#[test]
fn empty_and_negative_descriptor_loops_do_not_access_backing_storage() {
    execute(
        "main :: () -> int { empty: []int; negative: []int = .{count=-9223372036854775808}; for empty return 1; for < negative return 2; return 42; }",
        42,
    );
}

#[test]
fn pointer_string_iteration_updates_original_byte_storage_in_both_directions() {
    execute(
        "main::()->int{bytes:=u8.[19,99,21];text:string=.{count=3,data=bytes.data};order:=0;for < *byte,index:text {order=order*10+index;byte.*+=1;}if order!=210 || bytes[1]!=100 return 1;sum:=0;for *byte:text {if byte.*==100 continue;sum+=cast(int)byte.*;}return sum;}",
        42,
    );
}

#[test]
fn pointer_string_iteration_captures_source_once_and_skips_negative_counts() {
    execute(
        "calls:=0;pick::()->int{calls+=1;return 0;}main::()->int{bytes:=u8.[20,20];views:[1]string;views[0]=.{count=2,data=bytes.data};for *byte:views[pick()] byte.*+=1;negative:string=.{count=-9223372036854775808};for < *byte:negative return 1;for *byte:\"\" return 2;for *byte:string.{count=0,data=null} return 3;return cast(int)bytes[0]+cast(int)bytes[1]+calls-1;}",
        42,
    );
}

#[test]
fn pointer_string_iteration_preserves_frozen_literal_backing() {
    execute_expected(
        "main::()->int{text:=\"abc\";for *byte:text byte.*=0;return 42;}",
        Expected::Trap(jai_vm::Error::ReadOnlyStorage),
    );
}

#[test]
fn value_string_iteration_reads_the_live_byte_without_allowing_mutation() {
    execute(
        "main::()->int{bytes:=u8.[20];text:string=.{count=1,data=bytes.data};sum:=0;for byte:text {bytes[0]+=20;sum+=cast(int)byte;}return sum+2;}",
        42,
    );
}

#[test]
fn named_outer_continue_runs_only_the_target_sequence_latch() {
    execute(
        "main :: () -> int { values: [2]int = .[10,11]; result := 0; for outer: values { defer result += 1; for inner: values { result += outer; continue outer; } } return result*2 - 4; }",
        42,
    );
}

#[test]
fn unsupported_removal_and_readonly_mutations_are_located() {
    for (source, message, token) in [
        (
            "main :: () -> int { remove it; return 0; }",
            "remove must name an active enclosing array iterator",
            "remove",
        ),
        (
            "main :: () -> int { for it: 0..2 remove it; return 0; }",
            "remove requires a mutable slice or dynamic-array iteration",
            "remove",
        ),
        (
            "main :: () -> int { a: [2]int; for a remove it; return 0; }",
            "remove requires a mutable slice or dynamic-array iteration",
            "remove",
        ),
        (
            "main :: () -> int { for \"a\" remove it; return 0; }",
            "remove requires a mutable slice or dynamic-array iteration",
            "remove",
        ),
        (
            "main::()->int{for *byte:\"abc\" byte.*=0;return 0;}",
            "constant string literal backing storage is read-only",
            "\"abc\"",
        ),
        (
            "TEXT::\"abc\";main::()->int{for *byte:TEXT byte.*=0;return 0;}",
            "constant string literal backing storage is read-only",
            "TEXT",
        ),
        (
            "main::()->int{bytes:=u8.[20];text:string=.{count=1,data=bytes.data};for *byte:text remove byte;return 0;}",
            "remove requires a mutable slice or dynamic-array iteration",
            "remove",
        ),
        (
            "main::()->int{bytes:=u8.[20];text:string=.{count=1,data=bytes.data};for byte:text byte=0;return 0;}",
            "a by-value array iterator is read-only",
            "byte=0;",
        ),
        (
            "main::()->int{bytes:=u8.[20];text:string=.{count=1,data=bytes.data};for byte:text pointer:=*byte;return 0;}",
            "a by-value array iterator is read-only",
            "*byte",
        ),
        (
            "main :: () -> int { a: [1]int; for a it = 2; return 0; }",
            "a by-value array iterator is read-only",
            "it = 2;",
        ),
        (
            "Point :: struct { x: int; } main :: () -> int { a: [1]Point; for a it.x += 2; return 0; }",
            "a by-value array iterator is read-only",
            "it.x",
        ),
        (
            "main :: () -> int { a: [1]int; for a p := *it; return 0; }",
            "a by-value array iterator is read-only",
            "*it",
        ),
        (
            "Row :: struct { values: [1]int; } main :: () -> int { rows: [1]Row; for outer: rows for *inner: outer.values inner.* = 1; return 0; }",
            "a by-value array iterator is read-only",
            "outer.values",
        ),
        (
            "Row :: struct { values: [1]int; } main :: () -> int { rows: [1]Row; for outer: rows view: []int = outer.values; return 0; }",
            "a by-value array iterator is read-only",
            "view: []int = outer.values;",
        ),
        (
            "Row :: struct { values: [1]int; } main :: () -> int { rows: [1]Row; for outer: rows p := outer.values.data; return 0; }",
            "a by-value array iterator is read-only",
            "outer.values.data",
        ),
        (
            "main :: () -> int { a: [1]int; s: []int = a; for outer: s for inner: s remove outer; return 0; }",
            "remove targeting an outer loop",
            "remove",
        ),
        (
            "main :: () -> int { descriptors: [1][]int; for outer: descriptors for inner: outer remove inner; return 0; }",
            "remove requires a mutable slice or dynamic-array iteration",
            "remove",
        ),
        (
            "main :: () -> int { a: [1]int; s: []int = a; for s { defer remove it; } return 0; }",
            "a deferred body cannot remove",
            "remove",
        ),
        (
            "main :: () -> int { a: [1]int; s: []int = a; for s { remove it; remove it; } return 0; }",
            "multiple remove statements",
            "remove",
        ),
        (
            "main :: () -> int { reverse := true; a: [1]int; for <=reverse a {} return 0; }",
            "iteration modifier requires a compile-time value",
            "reverse",
        ),
    ] {
        let error = program(source).unwrap_err();
        assert!(error.message.contains(message), "{}", error.message);
        assert_eq!(error.location.span.text(source), token);
    }
}
