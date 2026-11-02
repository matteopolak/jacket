//! `jacket test --list` names the `@compile_time` tests without running them: an editor lists
//! on every refresh, and the compiler would otherwise run each body while it builds.
#[path = "support/installed.rs"]
mod installed;
use installed::{archive, jacket, scratch};

const TESTS: &str = r#"#import "Basic";
#import "Extensions/Testing";

// Running this in the compiler ends the build.
ends_the_compile :: () {
    print("compile_time_marker\n");
    exit(7);
} @test @compile_time

fails_in_the_compiler :: () {
    expect(false, "compile_time_failure");
} @test @compile_time

plain :: () {
    expect(true);
} @test
"#;

#[test]
fn list_does_not_run_compile_time_tests() {
    let root = scratch("ct");
    let Some(exe) = archive(&root) else {
        return;
    };
    let project = root.join("project");
    std::fs::create_dir_all(project.join("tests")).expect("create tests");
    std::fs::write(project.join("tests/ct.jai"), TESTS).expect("write the tests");

    let args = ["test", "--list"];
    let out = jacket(&exe, &project, &args);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}{stderr}");
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            "ends_the_compile: test",
            "fails_in_the_compiler: test",
            "plain: test"
        ],
        "{stderr}"
    );
}

#[test]
fn list_deps_prints_the_files_the_test_program_reads() {
    let root = scratch("deps");
    let Some(exe) = archive(&root) else {
        return;
    };
    let project = root.join("project");
    std::fs::create_dir_all(project.join("tests")).expect("create tests");
    let tests = "#import \"Basic\";\n#import \"Extensions/Testing\";\n\nplain :: () {\n    expect(true);\n} @test\n";
    std::fs::write(project.join("tests/ct.jai"), tests).expect("write the tests");

    let out = jacket(&exe, &project, &["test", "--list", "--deps"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stdout}{stderr}");
    let files: Vec<&str> = stdout.lines().collect();
    let has = |suffix: &str| files.iter().any(|f| f.ends_with(suffix));
    assert!(has("tests/ct.jai"), "{files:?}");
    assert!(has("Extensions/Testing/module.jai"), "{files:?}");
    assert!(has("prelude/module.jai"), "{files:?}");
    assert!(!stdout.contains("plain"), "files, not test names: {stdout}");

    let refused = jacket(&exe, &project, &["test", "--deps"]);
    assert!(!refused.status.success());
}
