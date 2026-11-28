//! A missing prerequisite ends a command before it does any work, with an `error:` that names the
//! need and the command that needs it, and a `note:` with the next step. Each test runs a copy of
//! `jacket` alone in a folder, with an empty `PATH`, so no tool is found.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A folder holding only a copy of `jacket`, an empty folder for `PATH`, and a cache folder.
struct Machine {
    dir: test_support::TempDir,
}

impl Machine {
    fn new(name: &str) -> Machine {
        let dir = test_support::TempDir::new(&format!("jacket-prerequisites-{name}"));

        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::create_dir_all(dir.join("path")).unwrap();
        std::fs::create_dir_all(dir.join("cache")).unwrap();
        let exe = dir
            .join("bin")
            .join(format!("jacket{}", std::env::consts::EXE_SUFFIX));

        std::fs::copy(env!("CARGO_BIN_EXE_jacket"), exe).unwrap();
        Machine {
            dir,
        }
    }

    fn path(&self) -> PathBuf {
        self.dir.join("path")
    }

    fn jacket(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(
            self.dir
                .join("bin")
                .join(format!("jacket{}", std::env::consts::EXE_SUFFIX)),
        )
        .args(args)
        .current_dir(cwd)
        .env("PATH", self.path())
        .env("XDG_CACHE_HOME", self.dir.join("cache"))
        .env("LOCALAPPDATA", self.dir.join("cache"))
        .env_remove("JAIC_STDLIB")
        .output()
        .expect("run jacket")
    }
}

fn stderr(output: &Output) -> String {
    assert!(!output.status.success(), "expected a failure: {output:?}");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The error names `program` and the `command` that needs it, and a note follows.
fn assert_missing(text: &str, program: &str, command: &str) {
    assert!(text.contains("error: "), "{text}");
    assert!(text.contains(&format!("`{program}`")), "{text}");
    assert!(text.contains(&format!("`{command}` needs it")), "{text}");
    assert!(text.contains("\nnote: "), "{text}");
    assert!(!text.contains("os error"), "{text}");
}

#[test]
fn fmt_without_jaifmt_says_what_to_install() {
    let machine = Machine::new("jaifmt");

    std::fs::write(machine.dir.join("a.jai"), "main :: () {}\n").unwrap();
    let text = stderr(&machine.jacket(&machine.dir, &["fmt", "a.jai"]));

    assert_missing(&text, "jaifmt", "jacket fmt");
}

#[test]
fn lint_without_jailint_says_what_to_install() {
    let machine = Machine::new("jailint");

    std::fs::write(machine.dir.join("a.jai"), "main :: () {}\n").unwrap();
    let text = stderr(&machine.jacket(&machine.dir, &["lint", "a.jai"]));

    assert_missing(&text, "jailint", "jacket lint");
}

#[test]
fn toolchain_add_without_curl_stops_before_touching_the_cache() {
    let machine = Machine::new("curl");
    let text = stderr(&machine.jacket(
        &machine.dir,
        &["toolchain", "add", "aarch64-unknown-linux-gnu"],
    ));

    assert_missing(&text, "curl", "jacket toolchain add");
    assert_eq!(
        std::fs::read_dir(machine.dir.join("cache"))
            .unwrap()
            .count(),
        0
    );
}

#[cfg(unix)]
#[test]
fn toolchain_add_with_an_unwritable_cache_names_the_folder_and_the_variable() {
    let machine = Machine::new("cache");

    // A file where the cache folder should be.
    use std::os::unix::fs::PermissionsExt;

    for tool in ["curl", "tar"] {
        let stub = machine.path().join(tool);

        std::fs::write(&stub, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let blocker = machine.dir.join("blocker");

    std::fs::write(&blocker, "").unwrap();
    let output = Command::new(machine.dir.join("bin/jacket"))
        .args(["toolchain", "add", "aarch64-unknown-linux-gnu"])
        .current_dir(&machine.dir)
        .env("PATH", machine.path())
        .env("XDG_CACHE_HOME", &blocker)
        .output()
        .unwrap();
    let text = stderr(&output);

    assert!(
        text.contains("error: cannot write to the cache folder"),
        "{text}"
    );
    assert!(text.contains("XDG_CACHE_HOME"), "{text}");
    assert!(text.contains("\nnote: "), "{text}");
}

#[test]
fn migrate_apply_in_a_repository_without_git_says_what_to_install() {
    let machine = Machine::new("git");
    let project = machine.dir.join("project");

    std::fs::create_dir_all(project.join(".git")).unwrap();
    std::fs::write(
        project.join("main.jai"),
        "main :: () {\n    c: Context;\n}\n",
    )
    .unwrap();
    let text = stderr(&machine.jacket(&project, &["migrate", "--apply", "main.jai"]));

    assert_missing(&text, "git", "jacket migrate");
}

#[test]
fn upgrade_without_curl_says_what_to_install() {
    let machine = Machine::new("upgrade");

    // An installed layout: `jaic` and `stdlib` beside `jacket`.
    std::fs::write(machine.dir.join("bin/jaic"), "").unwrap();
    std::fs::create_dir_all(machine.dir.join("bin/stdlib")).unwrap();
    let text = stderr(&machine.jacket(&machine.dir, &["upgrade", "--check"]));

    assert_missing(&text, "curl", "jacket upgrade");
}

#[test]
fn test_with_a_runner_that_is_not_installed_stops_before_building() {
    let machine = Machine::new("runner");
    let text = stderr(&machine.jacket(
        &machine.dir,
        &["test", "--runner", "wasmtime", "--target", "wasm32-wasi"],
    ));

    assert_missing(&text, "wasmtime", "jacket test");
}

#[cfg(unix)]
fn stub(machine: &Machine, tool: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = machine.path().join(tool);

    std::fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn upgrade_without_tar_stops_before_downloading_or_asking() {
    let machine = Machine::new("upgrade-tar");

    std::fs::write(machine.dir.join("bin/jaic"), "").unwrap();
    std::fs::create_dir_all(machine.dir.join("bin/stdlib")).unwrap();
    stub(&machine, "curl");
    let text = stderr(&machine.jacket(&machine.dir, &["upgrade", "--yes"]));

    assert_missing(&text, "tar", "jacket upgrade");
    assert!(!text.contains("Replace"), "{text}");
}

#[test]
fn doc_with_a_runtime_and_no_node_stops_before_building_the_site() {
    let machine = Machine::new("doc-node");
    let text = stderr(&machine.jacket(&machine.dir, &["doc", "--runtime", "runtime-dir"]));

    assert_missing(&text, "node", "jacket doc --runtime");
    assert!(text.contains("drop --runtime"), "{text}");
}
