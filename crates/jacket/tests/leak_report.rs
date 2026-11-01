//! A leak found by the test framework's allocator tracking is reported at the line in the test that asked for the memory,
//! not at the allocator or the library code between the two.
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::TempDir;

const PROGRAM: &str = r#"#import "Basic";
#import "String";
#import "Extensions/Testing";

joins_and_forgets :: () {
    parts := string.["one", "two", "three"];
    text := join(..parts, ", ");
} @test

allocates_and_forgets :: () {
    alloc(32);
} @test

grows_an_array_and_forgets :: () {
    items: [..] int;
    array_add(*items, 1);
} @test
"#;

/// What `jacket test -q` must show for those three tests. The line after each "made here" is the source line.
const GOLDEN: &str = include_str!("golden/leak_report.txt");

/// The directory holding `jaic` beside `jacket`; `None` when `jaic` has not been built.
fn jaic_dir() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_BIN_EXE_jacket"))
        .parent()?
        .to_path_buf();
    dir.join("jaic").is_file().then_some(dir)
}

#[test]
fn golden_a_leak_is_reported_where_the_test_allocated() {
    let Some(_) = jaic_dir() else {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return;
    };
    let project = TempDir::new("jacket-leak-golden");
    let tests = project.join("tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(project.join("jacket.toml"), "[project]\n").unwrap();
    std::fs::write(tests.join("leaks.jai"), PROGRAM).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jacket"))
        .args(["test", "-q"])
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
    assert!(!text.contains("tracking.jai"), "{text}");
    for block in GOLDEN.split("\n\n").filter(|block| !block.is_empty()) {
        assert!(text.contains(block), "missing:\n{block}\n\nin:\n{text}");
    }
}
