//! Exercise lifecycle recipes with independently authored source and fresh native output.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const API: &str = r#"
compiler_create_workspace :: (name: string) -> s64 #compiler;
compiler_destroy_workspace :: (w: s64) #compiler;
add_build_string :: (data: string, w: s64) #compiler;
Build_Options :: struct { output_path: string; }
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
"#;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-cli-workspace-lifecycle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn command(&self, action: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_jai-rs"));
        for variable in [
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
            command.env_remove(variable);
        }
        command.arg(action).arg(self.0.join("main.jai"));
        command
    }
    fn build(&self) -> Output {
        self.command("build")
            .arg(self.0.join("root-program"))
            .output()
            .unwrap()
    }
}

#[test]
fn stallable_source_waits_for_real_no_output_child_and_builds_native_root() {
    let messages = include_str!("../../../tests/fixtures/compiler-message-api.jai");
    let fixture = Fixture::new(&format!(
        r#"{messages}
Output_Type :: enum u8 {{ NO_OUTPUT; EXECUTABLE; DYNAMIC_LIBRARY; STATIC_LIBRARY; OBJECT_FILE; }}
Build_Options :: struct {{ output_type: Output_Type = .EXECUTABLE; output_path: string; }}
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
#run,stallable {{
    write_string("before wait");
    child := compiler_create_workspace("checked child");
    add_build_string("child :: () -> int {{ return 42; }}", child);
    set_build_options(Build_Options.{{output_type = .NO_OUTPUT, output_path = "never-created-child"}}, child);
    compiler_begin_intercept(child, Intercept_Flags.SKIP_ALL);
    while true {{
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {{ break; }}
    }}
    write_string("after wait");
    compiler_end_intercept(child);
}}
main :: () -> int {{ return 42; }}
"#
    ));
    let output = fixture.build();
    succeeded(&output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("before wait").count(), 1);
    assert_eq!(stdout.matches("after wait").count(), 1);
    assert!(!fixture.0.join("never-created-child").exists());
    assert_eq!(execute(&fixture.0.join("root-program")), 42);
}

#[test]
fn pending_record_field_runs_complete_before_native_globals_and_header_defaults() {
    for tail in [
        "global:Record; main::()->int{return global.value;}",
        "read::(value:Record=Record.{}) -> int{return value.value;} main::()->int{return read();}",
        "global:Record; read::(value:Record=Record.{}) -> int{return value.value;} main::()->int{return global.value+read()-42;}",
    ] {
        let fixture = Fixture::new(&format!(
            "Record::struct{{seed::()->int{{return 42;}} value:int=#run seed();}} {tail}"
        ));
        succeeded(&fixture.build());
        assert_eq!(execute(&fixture.0.join("root-program")), 42);
    }
}

#[test]
fn retained_stallable_source_guard_loads_selected_file_and_builds_native_root() {
    let messages = include_str!("../../../tests/fixtures/compiler-message-api.jai");
    let fixture = Fixture::new(&format!(
        r#"{messages}
Output_Type::enum u8 {{NO_OUTPUT; EXECUTABLE; DYNAMIC_LIBRARY; STATIC_LIBRARY; OBJECT_FILE;}}
Build_Options::struct {{output_type:Output_Type=.EXECUTABLE;}}
set_build_options::(options:Build_Options,w:s64=-1) #compiler;
calls:int;
choose::()->bool {{
    calls+=1;
    write_string("guard before");
    child:=compiler_create_workspace("guard child");
    add_build_string("child::()->int{{return 42;}}",child);
    set_build_options(Build_Options.{{output_type=.NO_OUTPUT}},child);
    compiler_begin_intercept(child,Intercept_Flags.SKIP_ALL);
    message:=compiler_wait_for_message();
    phase:=cast(*Message_Phase) message;
    compiler_end_intercept(child);
    write_string("guard after");
    return calls==1 && phase.phase==.ALL_SOURCE_CODE_PARSED;
}}
#if #run,stallable choose() {{#load "selected.jai";}} else {{#load "absent.jai";}}
main::()->int {{return ANSWER;}}
"#
    ));
    fs::write(fixture.0.join("selected.jai"), "ANSWER::42;").unwrap();
    let output = fixture.build();
    succeeded(&output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("guard before").count(), 1);
    assert_eq!(stdout.matches("guard after").count(), 1);
    assert_eq!(execute(&fixture.0.join("root-program")), 42);
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn execute(path: &Path) -> i32 {
    let mut child = Command::new(path).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status.code().unwrap();
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("freshly authored fixture exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn retiring_every_workspace_succeeds_without_new_or_overwritten_artifacts() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    child := compiler_create_workspace("cancelled");
    add_build_string("invalid source must never be compiled", child);
    compiler_destroy_workspace(child);
    compiler_destroy_workspace(-1);
}}
#run recipe();"#
    ));
    for action in ["build", "emit-object", "emit-llvm"] {
        let destination = fixture.0.join(format!("preserved-{action}"));
        fs::write(&destination, b"previous artifact").unwrap();
        let output = fixture
            .command(action)
            .arg(&destination)
            .env("JAI_RS_CLANG", fixture.0.join("missing-clang"))
            .output()
            .unwrap();
        succeeded(&output);
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read(&destination).unwrap(), b"previous artifact");

        let absent = fixture.0.join(format!("absent-{action}"));
        succeeded(&fixture.command(action).arg(&absent).output().unwrap());
        assert!(!absent.exists());
        assert!(
            !fixture
                .0
                .join(format!("absent-{action}.cancelled"))
                .exists()
        );
    }
    let output = fixture.command("emit-llvm").output().unwrap();
    succeeded(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn retiring_root_builds_only_the_surviving_child() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    child := compiler_create_workspace("surviving");
    compiler_destroy_workspace(-1);
    add_build_string("main :: () -> int {{ return 42; }}", child);
    set_build_options(Build_Options.{{output_path = "child-program"}}, child);
}}
#run recipe();
main :: () -> int {{ return retired_root_dependency; }}"#
    ));
    let root_output = fixture.0.join("root-program");
    fs::write(&root_output, b"previous root").unwrap();
    let output = fixture.build();
    succeeded(&output);
    assert_eq!(fs::read(&root_output).unwrap(), b"previous root");
    assert_eq!(execute(&fixture.0.join("child-program")), 42);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("child-program"), "{stdout}");
    assert!(!stdout.contains("root-program"), "{stdout}");
}

#[test]
fn retired_invalid_and_empty_children_do_not_block_root_native_output() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    invalid := compiler_create_workspace("invalid");
    add_build_string("this is invalid source", invalid);
    set_build_options(Build_Options.{{output_path = "retired-child"}}, invalid);
    compiler_destroy_workspace(invalid);
    empty := compiler_create_workspace("empty");
    compiler_destroy_workspace(empty);
}}
#run recipe();
main :: () -> int {{ return 43; }}"#
    ));
    let child_output = fixture.0.join("retired-child");
    fs::write(&child_output, b"previous child").unwrap();
    succeeded(&fixture.build());
    assert_eq!(execute(&fixture.0.join("root-program")), 43);
    assert_eq!(fs::read(&child_output).unwrap(), b"previous child");
    assert!(!fixture.0.join("root-program.empty").exists());
}

#[test]
fn self_retiring_child_preserves_its_prior_artifact_and_drops_invalid_entry() {
    let fixture = Fixture::new(&format!(
        r#"{API}
recipe :: () {{
    child := compiler_create_workspace("self-retiring");
    add_build_string("compiler_destroy_workspace :: (w: s64) #compiler; #run compiler_destroy_workspace(-1); main :: () -> int {{ return retired_dependency; }}", child);
}}
#run recipe();
main :: () -> int {{ return 44; }}"#
    ));
    let child_output = fixture.0.join("root-program.self-retiring");
    fs::write(&child_output, b"previous child").unwrap();
    succeeded(&fixture.build());
    assert_eq!(execute(&fixture.0.join("root-program")), 44);
    assert_eq!(fs::read(&child_output).unwrap(), b"previous child");
}

#[test]
fn stale_handle_request_fails_transaction_and_preserves_previous_outputs() {
    let fixture = Fixture::new(&format!(
        r#"{API}
write_string :: (text: string, error: bool = false) #no_context #compiler;
recipe :: () {{
    child := compiler_create_workspace("cancelled");
    compiler_destroy_workspace(child);
    write_string("uncommitted output");
    add_build_string("main :: () {{}}", child);
}}
#run recipe();
main :: () {{}}"#
    ));
    let root_output = fixture.0.join("root-program");
    fs::write(&root_output, b"previous root").unwrap();
    let output = fixture.build();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("destroyed"));
    assert!(output.stdout.is_empty());
    assert_eq!(fs::read(&root_output).unwrap(), b"previous root");
    assert!(!fixture.0.join("root-program.cancelled").exists());
}
