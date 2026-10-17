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
