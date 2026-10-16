//! Safe independent anonymous-import controls supplied by foundations.
use std::{path::PathBuf, process::Command};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/producer-requests/import-scope")
        .join(name)
}

#[test]
fn anonymous_loaded_import_runs_in_interpreter_and_compile_time() {
    for (name, args, expected) in [
        ("anonymous-load.jai", vec!["-interp"], "ok\n"),
        (
            "anonymous-compile-time.jai",
            vec!["-check", "-", "test"],
            "selected\n",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture(name))
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn named_aliases_constants_and_absent_imports_stay_private_or_unknown() {
    for (name, identifier) in [
        ("named-private-negative.jai", "File"),
        ("other-named-private-negative.jai", "Hidden_File"),
        ("constant-private-negative.jai", "private_number"),
        ("missing-import-negative.jai", "get_absolute_path"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
            .arg(fixture(name))
            .arg("-check")
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name} unexpectedly succeeded");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("unknown identifier `{identifier}`")),
            "{name}: {error}"
        );
    }
}

#[cfg(feature = "llvm")]
#[test]
fn anonymous_loaded_import_executes_native_o0_and_o2() {
    let compiler = PathBuf::from(env!("CARGO_BIN_EXE_jaic"));
    let scratch = compiler
        .parent()
        .unwrap()
        .join(format!("loaded-import-scope-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    for optimization in ["-O0", "-O2"] {
        let executable = scratch.join(format!("anonymous{optimization}"));
        let output = Command::new(&compiler)
            .arg(fixture("anonymous-load.jai"))
            .args([optimization, "-exe"])
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        let output = Command::new(&executable).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
        assert!(output.stderr.is_empty());
    }
    std::fs::remove_dir_all(scratch).unwrap();
}
