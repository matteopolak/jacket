//! Only independently written source and installed host system libraries execute.
#![cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "../../jai-codegen/tests/support/native_tools.rs"]
mod native_tools;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-own-foreign-library-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn command(&self, action: &str) -> std::process::Output {
        self.command_with_options(action, &[])
    }
    fn command_with_options(&self, action: &str, options: &[&str]) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jai-rs"));
        for key in [
            "JAI_RS_MODULE_PATH",
            "JAI_RS_STDLIB",
            "JAI_RS_PRELOAD",
            "JAI_RS_RUNTIME_SUPPORT",
            "JAI_RS_RUNTIME_ENTRY",
            "JAI_RS_RUNTIME_INITIALIZATION",
            "JAI_RS_RUNTIME_BACKTRACE",
            "JAI_RS_TARGET",
            "JAI_RS_CPU",
            "JAI_RS_FEATURES",
            "JAI_RS_OPT",
            "JAI_RS_DEBUG",
            "JAI_RS_CLANG",
            "JAI_RS_AR",
            "JAI_RS_NATIVE_VMA_RECEIPT",
            "JAI_RS_NATIVE_VMA_LIBRARY",
        ] {
            command.env_remove(key);
        }
        command
            .arg(action)
            .arg(self.0.join("main.jai"))
            .arg(self.0.join("program"))
            .args(options)
            .output()
            .unwrap()
    }
    fn run(&self) {
        let build = self.command("build");
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        execute(&self.0.join("program"), 0);
    }
}

fn execute(path: &Path, expected: i32) {
    let mut child = Command::new(path).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(expected));
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("authored foreign library fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn system_alias_and_renamed_symbol_execute_against_host_libc() {
    Fixture::new("Crt :: #library,system \"libc\"; absolute :: (value:s32)->s32 #foreign Crt \"abs\"; main :: () -> int { if absolute(-42) == 42 return 0; return 1; }").run();
}

#[test]
fn historical_library_spelling_resolves_nested_lexical_system_metadata() {
    Fixture::new("main :: () -> int { Crt :: #foreign_library,system \"libc\"; absolute :: (value:s32)->s32 #foreign Crt \"abs\"; if absolute(-42) == 42 return 0; return 1; }").run();
}

#[test]
fn nested_system_library_and_foreign_prototype_execute() {
    Fixture::new("main :: () -> int { if true { Crt :: #system_library \"libc\"; absolute :: (value:s32)->s32 #foreign Crt \"abs\"; if absolute(-42) == 42 return 0; } return 1; }").run();
}

#[test]
fn link_always_dependencies_are_not_dropped_without_prototypes() {
    let fixture = Fixture::new(
        "Missing :: #system_library,link_always \"jai_rs_intentionally_missing_foreign_dependency\"; main :: () -> int { return 0; }",
    );
    let build = fixture.command("build");
    assert!(!build.status.success());
    assert!(
        String::from_utf8_lossy(&build.stderr)
            .contains("jai_rs_intentionally_missing_foreign_dependency")
    );
}

#[test]
fn source_declared_local_metadata_emits_objects_but_never_loads_native_inputs() {
    let fixture = Fixture::new(
        "Local :: #library \"native-bytes-must-not-be-opened\"; external :: () #foreign Local; main :: () -> int { external(); return 0; }",
    );
    let object = fixture.command("emit-object");
    assert!(
        object.status.success(),
        "{}",
        String::from_utf8_lossy(&object.stderr)
    );
    assert!(fixture.0.join("program").is_file());
    let build = fixture.command("build");
    assert!(!build.status.success());
    assert!(
        String::from_utf8_lossy(&build.stderr)
            .contains("local native library linking is unsupported")
    );
}

#[test]
fn unused_local_library_metadata_does_not_require_native_linking() {
    Fixture::new("Local :: #library \"native-bytes-must-not-be-opened\"; external :: () #foreign Local; main :: () -> int { return 0; }").run();
}

#[test]
fn source_external_program_data_links_with_an_authored_c_provider() {
    let fixture = Fixture::new(
        "counter:s64 #elsewhere; #program_export \"increment_counter\" increment :: ()->s64 #c_call { counter+=2; return counter; }",
    );
    let consumer = fixture.0.join("consumer.c");
    fs::write(&consumer, "long long counter=40; extern long long increment_counter(void); int main(void) { long long result=increment_counter(); return result==42 && counter==42 ? 42 : 1; }").unwrap();
    for optimization in ["-O0", "-O2"] {
        let object = fixture.command_with_options("emit-object", &[optimization]);
        assert!(
            object.status.success(),
            "{}",
            String::from_utf8_lossy(&object.stderr)
        );
        let executable = fixture.0.join(format!("consumer{optimization}"));
        let linked = native_tools::clang_command()
            .arg(&consumer)
            .arg(fixture.0.join("program"))
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{}",
            String::from_utf8_lossy(&linked.stderr)
        );
        execute(&executable, 42);
    }
}

#[test]
fn source_external_data_requires_its_reached_library_without_opening_local_inputs() {
    let fixture = Fixture::new(
        "Local :: #library \"native-bytes-must-not-be-opened\"; counter:s64 #elsewhere Local \"external_counter\"; main :: ()->int { return cast(int)counter; }",
    );
    let object = fixture.command("emit-object");
    assert!(
        object.status.success(),
        "{}",
        String::from_utf8_lossy(&object.stderr)
    );
    let path = fixture.0.join("program");
    let previous = fs::read(&path).unwrap();
    let build = fixture.command("build");
    assert!(!build.status.success());
    assert!(
        String::from_utf8_lossy(&build.stderr)
            .contains("local native library linking is unsupported"),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert_eq!(fs::read(path).unwrap(), previous);
}

#[test]
fn unused_external_data_does_not_demand_its_local_library() {
    Fixture::new("Unused :: #library \"native-bytes-must-not-be-opened\"; counter:s64 #elsewhere Unused; main :: ()->int { return 0; }").run();
}

#[test]
fn compile_time_external_data_requires_a_checked_provider_at_the_run_origin() {
    for body in [
        "return counter;",
        "counter=42; return 0;",
        "pointer:=*counter; return 0;",
    ] {
        let fixture = Fixture::new(&format!(
            "counter:s64 #elsewhere;\naccess :: ()->s64 {{ {body} }}\nVALUE :: #run access();\nmain :: ()->int {{ return cast(int)VALUE; }}",
        ));
        let artifact = fixture.0.join("program");
        fs::write(&artifact, b"preserved authored output").unwrap();
        let build = fixture.command("build");
        assert!(!build.status.success());
        let stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            stderr.contains("has no checked compile-time data provider"),
            "{stderr}"
        );
        assert!(stderr.contains("main.jai:3:"), "{stderr}");
        assert_eq!(fs::read(artifact).unwrap(), b"preserved authored output");
    }
}

#[test]
fn external_data_aliases_and_incompatible_export_symbols_preserve_outputs() {
    for (source, diagnostic) in [
        (
            "#program_export \"renamed\" counter:s64 #elsewhere;",
            "requires an unsupported native data alias",
        ),
        (
            "counter:s64 #elsewhere; #program_export \"counter\" read_counter :: ()->s64 #c_call { return counter; }",
            "external data symbol \"counter\" has incompatible checked bindings",
        ),
        (
            "counter:s64 #elsewhere; #program_export \"counter\" owned:s64=7;",
            "external data symbol \"counter\" has incompatible checked bindings",
        ),
        (
            "#program_export \"counter\" owned:s64=7; counter:s64 #elsewhere;",
            "external data symbol \"counter\" has incompatible checked bindings",
        ),
    ] {
        let fixture = Fixture::new(source);
        let artifact = fixture.0.join("program");
        fs::write(&artifact, b"preserved authored output").unwrap();
        let object = fixture.command("emit-object");
        assert!(!object.status.success());
        let stderr = String::from_utf8_lossy(&object.stderr);
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert_eq!(fs::read(artifact).unwrap(), b"preserved authored output");
    }
}
