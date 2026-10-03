//! End-to-end checks for our own generated anonymous field schemas.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-inline-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check(source: &str, expected: i32) {
    let scratch = Scratch::new();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode: optimization,
                ..Optimization::default()
            },
            ..TargetOptions::default()
        })
        .unwrap();
        let program = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(target.layout_policy().unwrap()),
                ..jai_sema::ResolveOptions::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        let execution = jai_vm::execute(&program, jai_vm::Limits::default());
        let jai_vm::Outcome::Complete(values) = execution.outcome else {
            panic!("VM failed: {:?}", execution.outcome);
        };
        let [jai_vm::Value::Int(value)] = values.as_slice() else {
            panic!("integer result required");
        };
        assert_eq!(value.value(), i128::from(expected));
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command.arg(&object);
        if cfg!(target_os = "macos") {
            command.arg("-Wl,-no_fixup_chains");
        }
        let output = command.arg("-o").arg(&executable).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected), "{optimization:?}\n{source}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated anonymous field fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn inline_nested_using_fields_and_contextual_enum_defaults_execute() {
    check(
        r#"
        Options :: struct {
            output: enum u8 { OMIT; EXECUTABLE; OBJECT; } = .EXECUTABLE;
            using common: struct {
                support: enum u8 { AUTO; INIT; OMIT; } = .INIT;
                using inner: struct { amount: int = 40; }
            }
        }
        main :: () -> int { options: Options; return options.amount + cast(int)options.output + cast(int)options.support; }
    "#,
        42,
    );
}

#[test]
fn inline_specializations_reuse_identity_and_keep_baked_defaults() {
    check(
        r#"
        Box :: struct(T:Type, Fill:T=19) {
            entry: struct { value:T=Fill; next:*Box(T,Fill); }
            mode: enum u8 { OFF; ON; } = .ON;
        }
        main :: () -> int {
            first:Box(int); second:Box(int)=first;
            second.entry=first.entry;
            if second.entry.next != null return 1;
            small:Box(u8,3);
            return second.entry.value+cast(int)small.entry.value+cast(int)small.mode+19;
        }
    "#,
        42,
    );
}

#[test]
fn inline_enum_defaults_apply_contextual_checked_and_wrapping_casts() {
    check(
        r#"
        Options :: struct {
            bits:enum_flags u8 { READ; WRITE; } = xx 3;
            mode:enum u8 { ZERO; ONE; } = xx 1;
            wrapped:enum u8 { ZERO; } = xx,no_check 256;
        }
        main :: () -> int { options:Options;
            return cast(int)options.bits + cast(int)options.mode
                 + cast(int)options.wrapped + 38; }
        "#,
        42,
    );
}

#[test]
fn inline_enums_keep_flags_aliases_and_explicit_width() {
    check(
        r#"
        Options :: struct {
            bits: enum_flags u16 { READ; WRITE; BOTH :: READ | WRITE; } = .BOTH;
            mode: enum u8 #specified { ZERO :: 0; LAST :: 39; ALIAS :: LAST; } = .ALIAS;
        }
        main :: () -> int { value:Options; return cast(int)value.bits+cast(int)value.mode; }
    "#,
        42,
    );
}

#[test]
fn inline_array_elements_keep_recursive_pointers_and_defaults() {
    check(
        r#"
        Holder :: struct { values:[2] struct { amount:int=21; next:*Holder; }; }
        main :: () -> int { holder:Holder;
            if holder.values[1].next != null return 1;
            return holder.values[0].amount+holder.values[1].amount; }
    "#,
        42,
    );
}

#[test]
fn inline_union_fields_use_real_overlapping_storage() {
    check(
        r#"
        Holder :: struct { value:union { small:u8; wide:int; }; }
        main :: () -> int { holder:Holder=.{value=.{wide=42}}; return holder.value.wide; }
    "#,
        42,
    );
}
