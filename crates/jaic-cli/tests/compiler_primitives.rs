//! Compiler operations must not depend on Jai implementations in the selected stdlib.
#![cfg(feature = "llvm")]
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn fixture(name: &str, text: &str) -> (PathBuf, PathBuf) {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("public-compiler-primitives")
        .join(name);
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("main.jai");
    std::fs::write(&source, text).unwrap();
    (directory, source)
}

fn success(command: &mut Command) {
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn bodyless_public_workspace_operation_runs_at_compile_time() {
    let (_, source) = fixture(
        "bodyless",
        r#"
#import "Basic";
C :: #import "Compiler";
compiler_create_workspace :: (name := "") -> Workspace #compiler;
#run,stallable {
    workspace := compiler_create_workspace("bodyless public contract");
    assert(C.get_name(workspace) == "bodyless public contract");
    options := C.get_build_options(workspace);
    options.output_type = .NO_OUTPUT;
    C.set_build_options(options, workspace);
    C.compiler_begin_intercept(workspace);
    C.add_build_string("main :: () {}", workspace);
    while true {
        message := C.compiler_wait_for_message();
        assert(message != null);
        if message.kind == .COMPLETE break;
    }
    C.compiler_end_intercept(workspace);
}
main :: () {}
"#,
    );
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source)
            .arg("-interp"),
    );
}

#[test]
fn compiler_selector_and_runtime_body_have_distinct_execution_paths() {
    let (directory, source) = fixture(
        "runtime-body",
        r#"
#import "Basic";
C :: #import "Compiler";
create :: (name := "") -> Workspace #compiler "compiler_create_workspace" {return -77;}
#run {
    workspace := create("compile-time implementation");
    assert(workspace > 0);
    C.compiler_destroy_workspace(workspace);
}
main :: () {assert(create("runtime implementation") == -77);}
"#,
    );
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&source)
            .arg("-interp"),
    );
    let executable = directory.join(if cfg!(windows) {
        "runtime.exe"
    } else {
        "runtime"
    });
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&source)
            .args(["-O2", "-exe"])
            .arg(&executable),
    );
    success(&mut Command::new(executable));
}

#[test]
fn malformed_compiler_signatures_report_an_error_instead_of_entering_a_handler() {
    for (index, declaration) in [
        "compiler_create_workspace :: (name: s64) -> Workspace #compiler;",
        "compiler_wait_for_message :: () #compiler;",
        "compiler_report :: (a: string,b: s64,c: s64,d: string) #compiler;",
        "add_global_data :: (a: [] u8,b: s64,c: *void,d: s64) -> s64 #compiler;",
    ]
    .iter()
    .enumerate()
    {
        let (_, source) = fixture(
            &format!("invalid-{index}"),
            &format!(
                "{declaration}\n#run {}();\nmain :: () {{}}",
                declaration.split_whitespace().next().unwrap()
            ),
        );
        // Referencing the procedure forces signature binding, without supplying invalid arguments.
        std::fs::write(
            &source,
            format!(
                "{declaration}\n#run {{ value := {}; }}\nmain :: () {{}}",
                declaration.split_whitespace().next().unwrap()
            ),
        )
        .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source)
            .arg("-interp")
            .output()
            .unwrap();
        assert!(!output.status.success());
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert!(
            diagnostic.contains("incompatible signature for compiler operation"),
            "{diagnostic}"
        );
    }
}

#[test]
fn bodyless_compiler_operations_remain_unavailable_at_runtime() {
    let (_, source) = fixture(
        "runtime-bodyless",
        r#"
compiler_create_workspace :: (name := "") -> Workspace #compiler;
main :: () { workspace := compiler_create_workspace(); }
"#,
    );
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .arg("-interp")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains("runs only at compile time"),
        "{diagnostic}"
    );
}

#[test]
fn aggregate_fields_use_the_selected_declarations_layout() {
    let (_, source) = fixture(
        "declaration-layout",
        r#"
#import "Basic";
C :: #import "Compiler";
Version :: struct {
    sentinel: u64 = 37;
    micro: s16;
    major: s64;
    minor: u8;
}
version :: (result: *Version) -> string #compiler "compiler_get_version_info";
#run {
    expected: C.Version_Info;
    expected_text := C.compiler_get_version_info(*expected);
    actual: Version;
    actual_text := version(*actual);
    assert(actual_text == expected_text);
    assert(actual.sentinel == 37);
    assert(actual.major == expected.major && actual.minor == expected.minor && actual.micro == expected.micro);
}
main :: () {}
"#,
    );
    success(
        Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(source)
            .arg("-interp"),
    );
}

#[test]
fn unknown_compiler_operations_do_not_run_the_runtime_body_at_compile_time() {
    let (_, source) = fixture(
        "unknown",
        r#"
unknown :: () -> s64 #compiler "operation_that_does_not_exist" {return 42;}
#assert(unknown() == 42);
main :: () {}
"#,
    );
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(source)
        .arg("-interp")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported compiler operation"));
}
