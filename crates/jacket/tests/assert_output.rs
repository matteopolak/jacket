//! A failed `assert` under `jacket test`, in a test or a doc comment's example: the report is
//! the failure's place and line, with no stack trace or crash note, and the tests after it run.
use std::path::Path;
use std::process::Command;
use test_support::{TempDir, toolchain};

const MODULE: &str = r#"#import "Basic";

// Two.
//
// ```jai
// assert(two() == 3);
// ```
two :: () -> int { return 2; }
"#;

const TESTS: &str = r#"#import "Basic";
#import "Extensions/Testing";
#import "Lib";

bad :: (t: *Test) { assert(two() == 3); } @test
after :: (t: *Test) { expect(two() == 2); } @test
"#;

#[test]
fn a_failed_assert_is_reported_without_a_crash() {
    if toolchain::jaic_beside(Path::new(env!("CARGO_BIN_EXE_jacket"))).is_none() {
        return;
    }
    // The project's folder name is the module's.
    let root = TempDir::new("jacket-assert-output");
    let project = root.join("Lib");
    std::fs::create_dir_all(project.join("tests")).unwrap();
    std::fs::write(
        project.join("jacket.toml"),
        "[project]\nentry = [\"module.jai\"]\n\n[test]\njobs = 1\n",
    )
    .unwrap();
    std::fs::write(project.join("module.jai"), MODULE).unwrap();
    std::fs::write(project.join("tests/lib.jai"), TESTS).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jacket"))
        .arg("test")
        .current_dir(&project)
        .env(
            "JAIC_STDLIB",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        )
        .output()
        .expect("start jacket");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("tests/lib.jai:5:\n    assertion failed"),
        "{text}"
    );
    assert!(
        text.contains("module.jai:6:\n    assertion failed"),
        "{text}"
    );
    assert!(text.contains("PASS") && text.contains("after"), "{text}");
    for unwanted in ["signal", "stack trace", "native stack", "stderr:"] {
        assert!(!text.contains(unwanted), "{unwanted}: {text}");
    }
}
