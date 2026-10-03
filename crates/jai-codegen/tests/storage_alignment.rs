//! Declaration alignment from source through VM storage and new native objects.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::BitcodeOptimization;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-storage-alignment-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn graph(&self, source: &str) -> jai_modules::ModuleGraph {
        let path = self.0.join("main.jai");
        fs::write(&path, source).unwrap();
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap()
    }
    fn parity(&self, source: &str) -> String {
        let graph = self.graph(source);
        let mut ir = String::new();
        for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
            let target = NativeTarget::select(&TargetOptions {
                optimization: Optimization {
                    bitcode: optimization,
                    ..Optimization::default()
                },
                ..TargetOptions::default()
            })
            .unwrap();
            let options = ResolveOptions {
                target: Some(target.build_target().unwrap()),
                ..ResolveOptions::default()
            };
            let program = resolve_graph_with_options(&graph, &options, &mut NoEffects).unwrap();
            let outcome = jai_vm::execute(&program, Limits::default()).outcome;
            assert!(
                matches!(&outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)),
                "{outcome:?}"
            );
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            if optimization == BitcodeOptimization::O0 {
                ir = module.print_to_string().to_string();
            }
            let object = self.0.join("program.o");
            let executable = self.0.join("program");
            target.write_object(&module, &object).unwrap();
            let output = native_tools::clang_command()
                .arg(&object)
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
                    panic!("generated alignment fixture timed out");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        ir
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn global_and_local_alignment_changes_addresses_without_changing_array_stride() {
    let ir = Fixture::new().parity(
        r#"
        ALIGN :: 64;
        buffer: [7] u8 #align ALIGN;
        address_aligned :: no_inline(p: *u8, requested: u64) -> bool {
            return cast(u64) p % requested == 0;
        }
        main :: () -> int {
            ALIGN :: 128;
            local: [7] u8 #align ALIGN = ---;
            natural: [7] u8;
            word: u64 #align 1;
            if cast(u64) *buffer % 64 != 0 return 1;
            if cast(u64) *local % 128 != 0 return 2;
            if !address_aligned(cast(*u8) *buffer, 64) return 5;
            if !address_aligned(cast(*u8) *local, 128) return 6;
            if !address_aligned(cast(*u8) *word, 8) return 7;
            if size_of(type_of(buffer)) != 7 || size_of(type_of(local)) != 7 return 3;
            if *local[1] - *local[0] != 1 return 4;
            local[0] = 42;
            return cast(int) local[0];
        }
    "#,
    );
    assert!(
        ir.lines().any(|line| line.contains("@jai.g0 =")
            && line.contains("[7 x i8]")
            && line.contains("align 64")),
        "{ir}"
    );
    assert!(
        ir.lines()
            .any(|line| line.contains("alloca [7 x i8]") && line.contains("align 128")),
        "{ir}"
    );
    assert!(
        ir.lines()
            .any(|line| line.contains("alloca [7 x i8]") && line.contains("align 1")),
        "{ir}"
    );
    assert!(
        ir.lines()
            .any(|line| line.contains("alloca i64") && line.contains("align 8")),
        "{ir}"
    );
}

#[test]
fn compile_time_execution_observes_declaration_alignment() {
    Fixture::new().parity(
        r#"
        buffer: [7] u8 #align 64;
        aligned :: () -> int {
            local: [7] u8 #align 128;
            if cast(u64) *local % 128 != 0 return 1;
            if cast(u64) *buffer % 64 != 0 return 2;
            return 42;
        }
        result :: #run aligned();
        main :: () -> int { return result; }
    "#,
    );
}

#[test]
fn global_compile_time_alignment_is_ready_before_global_storage_is_observed() {
    Fixture::new().parity(
        r#"
        requested_alignment :: () -> int { return 64; }
        alignment :: #run requested_alignment();
        first: [7] u8 #align alignment;
        second: [7] u8 #align #run 128;
        third: [8] u8 #align #run size_of(type_of(third));
        observed :: #run () -> int {
            if cast(u64) *first % 64 != 0 return 1;
            if cast(u64) *second % 128 != 0 return 2;
            if cast(u64) *third % 8 != 0 return 3;
            return 42;
        };
        main :: () -> int { return observed; }
    "#,
    );
}

#[test]
fn global_alignment_cannot_depend_on_observing_its_own_pending_storage() {
    let fixture = Fixture::new();
    let graph = fixture.graph(
        r#"
        buffer: [4] u8 #align #run requested_alignment();
        requested_alignment :: () -> int {
            return ifx buffer[0] == 0 then 64 else 128;
        }
        main :: () -> int { return 42; }
    "#,
    );
    let error =
        resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects).unwrap_err();
    assert!(error.message.contains("alignment"), "{error:?}");
    assert!(
        error.message.contains("ready") || error.message.contains("cyclic"),
        "{error:?}"
    );
    assert!(error.location.span.end > error.location.span.start);
}

#[test]
fn invalid_source_alignment_is_a_located_error() {
    for alignment in [
        "0",
        "3",
        "-8",
        "4294967296",
        "true",
        "1.5",
        "#run 0",
        "#run 3",
        "#run 4294967296",
        "#run true",
    ] {
        for local in [false, true] {
            let source = if local {
                format!("main :: () -> int {{ value: u8 #align {alignment}; return 42; }}")
            } else {
                format!("value: u8 #align {alignment}; main :: () -> int {{ return 42; }}")
            };
            let fixture = Fixture::new();
            let graph = fixture.graph(&source);
            let error =
                resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects)
                    .unwrap_err();
            assert!(
                error.message.contains("storage alignment"),
                "{alignment}: {error:?}"
            );
            assert!(error.location.span.end > error.location.span.start);
        }
    }
}

#[test]
fn runtime_storage_cannot_supply_an_alignment_constant() {
    let fixture = Fixture::new();
    let graph = fixture
        .graph("main :: () -> int { requested := 64; value: u8 #align requested; return 42; }");
    let error =
        resolve_graph_with_options(&graph, &ResolveOptions::default(), &mut NoEffects).unwrap_err();
    assert!(error.message.contains("runtime storage"), "{error:?}");
}

#[test]
fn imported_alignment_uses_its_defining_scope() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("storage.jai"),
        r#"
        requested_alignment :: () -> int { return 64; }
        ALIGN :: #run requested_alignment();
        buffer: [7] u8 #align ALIGN;
        address_aligned :: () -> bool { return cast(u64) *buffer % cast(u64) ALIGN == 0; }
    "#,
    )
    .unwrap();
    fixture.parity(
        r#"
        ALIGN :: 3;
        Library :: #import, file "storage.jai";
        main :: () -> int {
            if !Library.address_aligned() return 1;
            return 42;
        }
    "#,
    );
}

#[test]
fn anonymous_compile_time_alignment_does_not_publish_temporary_local_identities() {
    Fixture::new().parity(
        r#"
        result :: #run () -> int {
            scratch: [7] u8 #align 256;
            if cast(u64) *scratch % 256 != 0 return 1;
            return 42;
        };
        main :: () -> int { return result; }
    "#,
    );
}

#[test]
fn source_alignment_obeys_a_narrow_target_address_space() {
    use jai_types::{LayoutPolicy, ScalarLayout};
    let fixture = Fixture::new();
    let graph = fixture.graph("buffer: u8 #align 2147483648; main :: () -> int { return 42; }");
    let layout = LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 4),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
        ScalarLayout::new(1, 1),
    )
    .unwrap();
    let options = ResolveOptions {
        layout: Some(layout),
        ..ResolveOptions::default()
    };
    let error = resolve_graph_with_options(&graph, &options, &mut NoEffects).unwrap_err();
    assert!(
        error.message.contains("target signed address space"),
        "{error:?}"
    );
}

#[test]
fn type_queries_and_materialized_constants_supply_local_alignment() {
    Fixture::new().parity(
        r#"
        alignment :: #run 64;
        global: [7] u8 #align size_of(u64) * 8;
        main :: () -> int {
            first: [7] u8 #align alignment;
            second: [7] u8 #align size_of(u64) * 8;
            if cast(u64) *first % 64 != 0 return 1;
            if cast(u64) *second % 64 != 0 return 2;
            if cast(u64) *global % 64 != 0 return 3;
            return 42;
        }
    "#,
    );
}

#[test]
fn specialized_procedures_keep_independent_alignment_requests() {
    let ir = Fixture::new().parity(
        r#"
        aligned :: ($ALIGN: int) -> int {
            scratch: [7] u8 #align ALIGN;
            if cast(u64) *scratch % cast(u64) ALIGN != 0 return 1;
            return 42;
        }
        main :: () -> int {
            if aligned(64) != 42 || aligned(128) != 42 return 1;
            return 42;
        }
    "#,
    );
    for alignment in [64, 128] {
        assert!(
            ir.lines().any(|line| line.contains("alloca [7 x i8]")
                && line.contains(&format!("align {alignment}"))),
            "{ir}"
        );
    }
}

#[test]
fn atomic_access_observes_the_actual_alignment_of_byte_storage() {
    Fixture::new().parity(r#"
        compare_and_swap :: (pointer:*$T, old:T, new:T) -> (success:bool, old_value:T) #intrinsic;
        global: [4] u8 #align 64;
        main :: () -> int {
            local: [4] u8 #align 128;
            global_pointer := cast(*u32) *global;
            local_pointer := cast(*u32) *local;
            changed_global, observed_global := compare_and_swap(global_pointer, cast(u32) 0, cast(u32) 42);
            changed_local, observed_local := compare_and_swap(local_pointer, cast(u32) 0, cast(u32) 42);
            if !changed_global || !changed_local || observed_global != 0 || observed_local != 0 return 1;
            if global_pointer.* != 42 || local_pointer.* != 42 return 2;
            return 42;
        }
    "#);
}
