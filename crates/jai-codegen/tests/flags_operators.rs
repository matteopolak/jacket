//! Execute only independently authored source and objects emitted by this compiler.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::target::NativeTarget;
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
        let root = std::env::temp_dir().join(format!(
            "jai-flags-operators-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn check(&self, source: &str, expected: i32) {
        let input = self.0.join("main.jai");
        fs::write(&input, source).unwrap();
        let graph =
            jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
        let target = NativeTarget::new().unwrap();
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
            panic!("VM did not complete: {execution:?}");
        };
        let [jai_vm::Value::Int(value)] = values.as_slice() else {
            panic!("expected an integer result: {values:?}");
        };
        assert_eq!(value.value(), i128::from(expected));

        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
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
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            module.print_to_string()
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected));
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated flags operator fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn nominal_flags_zero_defaults_execute_in_parameters_globals_and_fields() {
    Fixture::new().check(
        r#"
Flags :: enum_flags u32 { FIRST :: 1; SECOND :: 2; }
ZERO :: 2 - 2;
stored: Flags = ZERO;
State :: struct { flags: Flags = 0; }
read :: (flags: Flags = ZERO) -> int { if flags == 0 return 40; return 1; }
main :: () -> int {
    state: State;
    if state.flags == 0 && stored == 0 return read() + 2;
    return 1;
}
"#,
        42,
    );
}

#[test]
fn source_mask_precedence_and_weak_zero_comparisons_execute_natively() {
    Fixture::new().check(
        r#"
Flags :: enum_flags u64 { FIRST :: 1; HIGH :: 0x8000000000000000; }
ZERO :: 1 - 1;
main :: () -> int {
    flags: Flags = .FIRST;
    empty: Flags;
    high: Flags = .HIGH;
    if flags & .FIRST != 0 && flags & .HIGH == 0 && 0 != high
       && empty == ZERO && ZERO == empty && !(0 == high) return 42;
    return 1;
}
"#,
        42,
    );
}

#[test]
fn contextual_complements_preserve_width_and_projected_mutation() {
    Fixture::new().check(
        r#"
Flags :: enum_flags u8 { FIRST :: 1; SECOND :: 2; THIRD :: 4; }
State :: struct { flags: Flags; }
main :: () -> int {
    flags: Flags = Flags.FIRST | .SECOND | .THIRD;
    flags &= ~.SECOND;
    state: State = .{flags=flags};
    state.flags &= ~.THIRD;
    inverse: Flags = ~.FIRST;
    nested: Flags = ~~.THIRD;
    reverse := ~.FIRST & Flags.THIRD;
    if cast(u8)inverse == 254 && cast(u8)nested == 4
       && cast(u8)state.flags == 1 && cast(u8)reverse == 4 return 42;
    return 1;
}
"#,
        42,
    );
}

#[test]
fn pure_flags_constants_and_run_use_the_runtime_nominal_operations() {
    Fixture::new().check(
        r#"
Flags :: enum_flags u16 { FIRST :: 1; SECOND :: 2; }
MASK :: Flags.FIRST | .SECOND;
PRESENT :: MASK & .FIRST != 0;
ABSENT :: MASK & .SECOND == 0;
probe :: () -> int { if PRESENT && !ABSENT return 42; return 1; }
ANSWER :: #run probe();
main :: () -> int { return ANSWER; }
"#,
        42,
    );
}
