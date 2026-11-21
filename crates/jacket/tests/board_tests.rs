//! `jacket test --target <board>` builds the tests for a board and runs each through the board's
//! emulator, with the result coming back over semihosting.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use test_support::{TempDir, toolchain};

const TESTS: &str = r#"#import "Basic";
#import "Extensions/Testing";

adds :: (t: *Test) {
    expect(1 + 1 == 2);
} @test

heap :: (t: *Test) {
    numbers: [..] int;
    defer array_free(numbers);
    for 1..100 array_add(*numbers, it);
    expect(numbers.count == 100);
    expect(numbers[99] == 100);
} @test

fails_on_purpose :: (t: *Test) {
    expect(2 + 2 == 5);
} @test
"#;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(jacket: &Path, dir: &TempDir, args: &[&str]) -> Output {
    Command::new(jacket)
        .arg("test")
        .args(args)
        .env("JAIC_STDLIB", repo().join("stdlib"))
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn tests_run_on_the_board_and_report_through_semihosting() {
    let jacket = PathBuf::from(env!("CARGO_BIN_EXE_jacket"));
    if toolchain::jaic_beside(&jacket).is_none()
        || toolchain::optional("qemu-system-aarch64").is_none()
    {
        return;
    }
    let dir = TempDir::new("board-tests");
    dir.write("tests/basic.jai", TESTS);
    let out = run(&jacket, &dir, &["--target", "Qemu_Virt_Aarch64"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // The report goes to standard error when it is not a terminal.
    let text = format!("{}{stderr}", String::from_utf8_lossy(&out.stdout));
    if stderr.contains("ld.lld not found") && !toolchain::strict("ld.lld") {
        toolchain::skip(&stderr);
        return;
    }
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("PASS") && text.contains("adds"), "{text}");
    assert!(text.contains("heap"), "{text}");
    assert!(
        text.contains("FAIL") && text.contains("fails_on_purpose"),
        "{text}"
    );
    assert!(text.contains("expected true, got false"), "{text}");
    assert!(text.contains("2 passed, 1 failed"), "{text}");

    // One test by name: its own process, and the run succeeds.
    let out = run(
        &jacket,
        &dir,
        &["--target", "Qemu_Virt_Aarch64", "--exact", "heap"],
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{text}");
    assert!(text.contains("1 passed"), "{text}");
}
