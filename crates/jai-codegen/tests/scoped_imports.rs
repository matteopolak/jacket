//! Independently written source modules execute through the VM and native code.
#[path = "support/native_tools.rs"]
mod native_tools;
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
            "jai-scoped-import-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn execute(&self, application: &str, library: &str) {
        fs::write(self.0.join("main.jai"), application).unwrap();
        fs::write(self.0.join("library.jai"), library).unwrap();
        let graph = jai_modules::ModuleGraph::load(
            &self.0.join("main.jai"),
            jai_modules::GraphOptions::default(),
        )
        .unwrap();
        let program = jai_sema::resolve_graph(&graph).unwrap();
        let execution = jai_vm::execute(&program, jai_vm::Limits::default());
        assert!(
            matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
            "{execution:?}"
        );
        let context = jai_codegen::Context::create();
        let target = jai_codegen::target::NativeTarget::new().unwrap();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = self.0.join("fixture.o");
        let executable = self.0.join("fixture");
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
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(42));
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("self-written scoped import fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn local_import_keeps_exported_callback_bound_to_private_module_procedure() {
    Fixture::new().execute("main :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.callback() + Lib.answer(); }", "#scope_module; helper :: () -> int { return 21; } #scope_export; callback :: helper; answer :: () -> int { return helper(); }");
}

#[test]
fn canonical_local_requests_share_storage_across_procedure_scopes() {
    Fixture::new().execute("first :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.next(); } main :: () -> int { Lib :: #import,file \"./library.jai\"; return first() + Lib.next(); }", "value: int = 19; next :: () -> int { value += 2; return value - 1; }");
}

#[test]
fn selected_using_import_binds_generic_and_overloaded_calls() {
    Fixture::new().execute("main :: () -> int { #if false { Missing :: #import \"Absent\"; } #import,file \"library.jai\"; return identity(21) + choose(21); }", "identity :: (value: $T) -> T { return value; } choose :: (value: int) -> int { return value; } choose :: (value: bool) -> int { return 0; }");
}
