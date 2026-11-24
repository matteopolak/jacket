//! `jaic -deps`: the source files a compile read.
use std::path::Path;
use std::process::{Command, Output};
use test_support::TempDir;

fn deps(file: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(file)
        .arg("-deps")
        .args(extra)
        .output()
        .unwrap()
}

fn lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn lists_loaded_imported_and_prelude_files() {
    let dir = TempDir::new("deps-list");
    std::fs::write(
        dir.join("main.jai"),
        "#import \"Basic\";\n#load \"part.jai\";\nmain :: () { print(\"%\\n\", part_value()); }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("part.jai"),
        "part_value :: () -> s32 { return 1; }\n",
    )
    .unwrap();
    let output = deps(&dir.join("main.jai"), &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files = lines(&output);
    let has = |suffix: &str| files.iter().any(|f| f.ends_with(suffix));
    assert!(has("main.jai"), "{files:?}");
    assert!(has("part.jai"), "{files:?}");
    assert!(has("Basic/module.jai"), "{files:?}");
    assert!(has("prelude/module.jai"), "{files:?}");
    assert!(
        files.iter().all(|f| Path::new(f).is_absolute()),
        "{files:?}"
    );
    assert!(
        files.windows(2).all(|pair| pair[0] < pair[1]),
        "sorted and unique"
    );
    // Nothing else is written.
    assert!(!dir.join("main").exists());
}

#[test]
fn a_program_that_does_not_compile_lists_nothing() {
    let dir = TempDir::new("deps-error");
    std::fs::write(dir.join("main.jai"), "main :: () { missing(); }\n").unwrap();
    let output = deps(&dir.join("main.jai"), &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "nothing is listed for a failed compile"
    );
}

#[test]
fn files_of_a_plugins_program_are_listed() {
    let dir = TempDir::new("deps-plugin");
    std::fs::write(
        dir.join("main.jai"),
        "#load \"part.jai\";\nmain :: () { part(); }\n",
    )
    .unwrap();
    std::fs::write(dir.join("part.jai"), "part :: () {}\n").unwrap();
    let output = deps(&dir.join("main.jai"), &["+Check"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files = lines(&output);
    let has = |suffix: &str| files.iter().any(|f| f.ends_with(suffix));
    assert!(has("part.jai"), "{files:?}");
    assert!(has("Check/module.jai"), "{files:?}");
}

#[test]
fn deps_cannot_be_combined_with_interp() {
    let dir = TempDir::new("deps-interp");
    std::fs::write(dir.join("main.jai"), "main :: () {}\n").unwrap();
    let output = deps(&dir.join("main.jai"), &["-interp"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("`-deps` and `-interp`"));
}

fn program(dir: &TempDir) -> std::path::PathBuf {
    std::fs::write(
        dir.join("main.jai"),
        "#import \"Basic\";\n#load \"part.jai\";\nmain :: () { print(\"%\\n\", part_value()); }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("part.jai"),
        "part_value :: () -> s32 { return 1; }\n",
    )
    .unwrap();
    dir.join("main.jai")
}

fn jaic(file: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg(file)
        .args(extra)
        .current_dir(file.parent().unwrap())
        .output()
        .unwrap()
}

#[test]
fn deps_file_matches_deps_in_check_mode() {
    let dir = TempDir::new("deps-file-check");
    let main = program(&dir);
    let printed = lines(&deps(&main, &[]));
    let list = dir.join("main.deps");
    let output = jaic(&main, &["-check", "-deps_file", list.to_str().unwrap()]);
    assert!(output.status.success());
    let written = std::fs::read_to_string(&list).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), printed);
    assert!(output.stdout.is_empty(), "the list goes to the file only");
}

#[test]
fn deps_file_is_written_by_a_build() {
    let dir = TempDir::new("deps-file-build");
    let main = program(&dir);
    let printed = lines(&deps(&main, &[]));
    let list = dir.join("main.deps");
    let output = jaic(
        &main,
        &[
            "-quiet",
            "-exe",
            "app",
            "-deps_file",
            list.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written = std::fs::read_to_string(&list).unwrap();
    assert_eq!(written.lines().collect::<Vec<_>>(), printed);
}

#[test]
fn deps_file_is_written_by_interp() {
    let dir = TempDir::new("deps-file-interp");
    let main = program(&dir);
    let list = dir.join("main.deps");
    let output = jaic(&main, &["-interp", "-deps_file", list.to_str().unwrap()]);
    assert!(output.status.success());
    let written = std::fs::read_to_string(&list).unwrap();
    assert!(
        written.lines().any(|f| f.ends_with("part.jai")),
        "{written}"
    );
}

#[test]
fn deps_file_is_not_written_for_a_failed_compile() {
    let dir = TempDir::new("deps-file-error");
    std::fs::write(dir.join("main.jai"), "main :: () { missing(); }\n").unwrap();
    let list = dir.join("main.deps");
    let output = jaic(
        &dir.join("main.jai"),
        &["-check", "-deps_file", list.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!list.exists());
}
