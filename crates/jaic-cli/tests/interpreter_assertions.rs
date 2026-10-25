//! Assertion callbacks must not abort the compiler host; custom callbacks still execute.
use std::{path::Path, process::Command};

#[test]
fn default_assertions_report_faults_and_custom_callbacks_can_continue() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("interpreter-assertion-callbacks");
    std::fs::create_dir_all(&dir).unwrap();
    for (name, source, status, diagnostic) in [
        (
            "hidden-frame",
            "#import \"Basic\";\nvisible :: () { assert(false, \"hidden frame marker\"); }\nhidden :: () #no_debug { visible(); }\nmain :: () { hidden(); }\n",
            1,
            "assertion failed: hidden frame marker",
        ),
        (
            "default",
            "#import \"Basic\";\nmain :: () { assert(false, \"original assertion marker\"); }\n",
            1,
            "assertion failed: original assertion marker",
        ),
        (
            "compile-time",
            "#import \"Basic\";\n#run { assert(false, \"compile-time assertion marker\"); }\nmain :: () {}\n",
            1,
            "assertion failed: compile-time assertion marker",
        ),
        (
            "custom",
            "#import \"Basic\";\nhandled :: (loc: Source_Code_Location, message: string) -> bool { return false; }\nmain :: () { context.assertion_failed = handled; assert(false, \"handled\"); print(\"ok\\n\"); }\n",
            0,
            "",
        ),
    ] {
        let file = dir.join(format!("{name}.jai"));
        std::fs::write(&file, source).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(file)
            .args(["-interp", "-no_workspace_output"])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(status), "{name}: {result:?}");
        let stderr = String::from_utf8_lossy(&result.stderr);
        if status == 0 {
            assert_eq!(result.stdout, b"ok\n");
            assert!(stderr.is_empty(), "{stderr}");
        } else {
            assert!(stderr.contains(diagnostic), "{name}: {stderr}");
            assert!(stderr.contains("error:"), "{name}: {stderr}");
            if name == "hidden-frame" {
                assert!(stderr.contains("`visible` at"), "{stderr}");
                assert!(!stderr.contains("`hidden` at"), "{stderr}");
            }
        }
    }
}

/// A failed `assert` is reported at its own line, whatever code is laid out after the check's
/// out-of-line failure path.
#[test]
fn a_failed_assertion_names_its_own_line() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("interpreter-assertion-lines");
    std::fs::create_dir_all(&dir).unwrap();
    let source = "\
#import \"Basic\";
three :: () -> s32 { return 3; }
main :: () {
    mark := get_temporary_storage_mark();
    defer set_temporary_storage_mark(mark);
    code := three();
    assert(code == 3);
    if code > 0 {
        assert(code == 2 && code > 0);
    }
    print(\"not reached\\n\");
}
";
    let file = dir.join("lines.jai");
    std::fs::write(&file, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(&file)
        .args(["-interp", "-no_workspace_output"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("lines.jai:9:9: error:"), "{stderr}");
    assert!(
        stderr.contains("assert(code == 2 && code > 0);"),
        "{stderr}"
    );
}

/// The run ends when `main` does, failed or not, even while another thread waits in C (as an Io
/// resolver helper does on its semaphore): a native program's exit ends such threads too.
#[test]
fn the_run_ends_while_a_thread_waits_in_c() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("interpreter-exit-blocked-thread");
    std::fs::create_dir_all(&dir).unwrap();
    for (name, ending, status) in [
        ("returns", "print(\"done\\n\");", 0),
        ("asserts", "assert(false, \"blocked thread marker\");", 1),
    ] {
        let file = dir.join(format!("{name}.jai"));
        let source = format!(
            "#import \"Basic\";\n#import \"Thread\";\nsem: Semaphore;\n\
             main :: () {{\n    init(*sem);\n    t: Thread;\n    \
             thread_init(*t, (t: *Thread) -> s64 {{ wait_for(*sem); return 0; }});\n    \
             thread_start(*t);\n    sleep_milliseconds(50);\n    {ending}\n}}\n"
        );
        std::fs::write(&file, source).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(&file)
            .args(["-interp", "-no_workspace_output"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        let code = loop {
            if let Some(exit) = child.try_wait().unwrap() {
                break exit.code();
            }
            if std::time::Instant::now() > deadline {
                let _ = child.kill();
                panic!("{name}: the run did not end after main returned");
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        assert_eq!(code, Some(status), "{name}");
    }
}
