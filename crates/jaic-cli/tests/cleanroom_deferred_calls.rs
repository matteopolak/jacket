//! Reference-confirmed placeholder boundary and independent controller invariants.
use std::{
    path::PathBuf,
    process::{Command, Output},
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/producer-requests/plugin-registration")
        .join(name)
}

fn run(name: &str, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(fixture(name))
        .arg(mode)
        .output()
        .unwrap()
}

fn expect_success(output: Output, expected: &str) {
    assert!(
        output.status.success(),
        "status {:?}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert!(output.stderr.is_empty());
}

const POSITIVES: &[(&str, &str, &str)] = &[
    ("deferred-body.jai", "deferred body resumed once\n", ""),
    (
        "deferred-invariants.jai",
        "prefix once\ntwo bodies resumed\n",
        "deferred runtime ok\n",
    ),
    (
        "deferred-scopes.jai",
        "scoped bodies ready\n",
        "scoped bodies runtime ok\n",
    ),
    ("deferred-other-run.jai", "ready producer serviced\n", ""),
    ("deferred-generator.jai", "generator dependency ready\n", ""),
    (
        "deferred-factory.jai",
        "dynamic factory initialized before return\n",
        "",
    ),
    (
        "deferred-shared-waiters.jai",
        "inner waiter resumed\nouter waiter resumed\n",
        "",
    ),
    (
        "deferred-native-callback.jai",
        "native callback ready on exposure\n",
        "",
    ),
];

#[test]
fn indirect_calls_resume_with_local_storage_scope_and_factory_results_intact() {
    for &(name, compile_output, runtime_output) in POSITIVES {
        expect_success(
            run(name, "-interp"),
            &format!("{compile_output}{runtime_output}"),
        );
    }
}

#[test]
fn a_child_workspace_services_its_own_sources_while_its_first_run_is_active() {
    expect_success(
        run("deferred-child.jai", "-check"),
        "prefix once\ntwo bodies resumed\nchild dependency complete\n",
    );
}

#[cfg(not(target_os = "windows"))]
#[test]
fn ordinary_native_callbacks_shared_inside_records_are_ready() {
    let compiler = PathBuf::from(env!("CARGO_BIN_EXE_jaic"));
    let scratch = compiler
        .parent()
        .unwrap()
        .join(format!("callback-record-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let c = scratch.join("record.c");
    let library = scratch.join(if cfg!(target_os = "macos") {
        "librecord.dylib"
    } else {
        "librecord.so"
    });
    std::fs::write(&c, "struct Record { int (*callback)(int); }; int invoke_record(struct Record *r) { return r->callback(1); }\n").unwrap();
    let output = Command::new("clang")
        .arg(if cfg!(target_os = "macos") {
            "-dynamiclib"
        } else {
            "-shared"
        })
        .args(["-fPIC", "-Wall", "-Wextra", "-Werror"])
        .arg(&c)
        .arg("-o")
        .arg(&library)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let source = scratch.join("record.jai");
    std::fs::write(
        &source,
        format!(
            r#"
using #import "Basic";
C :: #import "Compiler";
native :: #library "{}";
#placeholder Body;
body_scope :: #code Body;
Callback :: #type (value: s32) -> s32 #c_call;
Record :: struct {{ callback: Callback; }}
invoke_record :: (record: *Record) -> s32 #foreign native;
abs :: (value: s32) -> s32 #foreign;
callback :: (value: s32) -> s32 #c_call {{ return value + 41; }}
deferred :: (value: s32) -> s32 #c_call {{ #insert Body; }}
force :: (value: s32) -> s32 {{ #insert Body; }}
#run {{
    record: Record;
    record.callback = callback;
    assert(invoke_record(*record) == 42);
    record.callback = deferred;
    assert(abs(-7) == 7);
    C.add_build_string("Body :: \"return value + 41;\";", -1, body_scope);
    assert(force(1) == 42);
    assert(invoke_record(*record) == 42);
    print("record callback ready\n");
}}
main :: () {{}}
"#,
            "record"
        ),
    )
    .unwrap();
    expect_success(
        Command::new(&compiler)
            .arg(&source)
            .arg("-interp")
            .env("JAIC_NATIVE_LIBS", &scratch)
            .output()
            .unwrap(),
        "record callback ready\n",
    );
    std::fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn unresolved_cycles_invalid_bodies_duplicates_and_cross_workspace_code_fail() {
    for (name, diagnostic) in [
        ("deferred-deadlock.jai", "compile-time dependency deadlock"),
        ("deferred-cycle.jai", "cyclic compile-time body dependency"),
        ("deferred-invalid-body.jai", "unknown identifier `absent`"),
        ("deferred-duplicate.jai", "already declared"),
        (
            "deferred-wrong-workspace.jai",
            "Code scope must belong to the running workspace",
        ),
        (
            "deferred-active-destroy.jai",
            "cannot destroy an active compiler workspace",
        ),
    ] {
        let output = run(name, "-check");
        assert!(!output.status.success(), "{name} unexpectedly succeeded");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
}

#[test]
fn a_resumed_body_reports_its_real_failure_without_replaying_the_prefix() {
    let output = run("deferred-failure.jai", "-check");
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "failure prefix once\n"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("assertion failed"), "{error}");
    assert!(!error.contains("never defined"), "{error}");
}

#[test]
fn failed_insert_and_transitive_generators_are_terminal_without_prefix_replay() {
    for (name, expected, diagnostic) in [
        (
            "deferred-insert-failure.jai",
            "insert prefix once\n",
            "dependency deadlock",
        ),
        (
            "deferred-transitive-failure.jai",
            "generator prefix once\n",
            "assertion failed",
        ),
        (
            "deferred-after-generator.jai",
            "completed generator once\n",
            "Body",
        ),
    ] {
        let output = run(name, "-check");
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
}

#[cfg(feature = "llvm")]
#[test]
fn generated_bodies_execute_in_actual_native_o0_and_o2_programs() {
    let compiler = PathBuf::from(env!("CARGO_BIN_EXE_jaic"));
    let scratch = compiler
        .parent()
        .unwrap()
        .join(format!("deferred-calls-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    for optimization in ["-O0", "-O2"] {
        for &(name, compile_output, runtime_output) in POSITIVES {
            let executable = scratch.join(format!("{name}{optimization}"));
            let output = Command::new(&compiler)
                .arg(fixture(name))
                .args([optimization, "-exe"])
                .arg(&executable)
                .output()
                .unwrap();
            expect_success(output, compile_output);
            expect_success(Command::new(&executable).output().unwrap(), runtime_output);
        }
    }
    std::fs::remove_dir_all(scratch).unwrap();
}
