//! `jacket run` on a project `jacket new` made, with the tools laid out like a release archive
//! (`jacket` and `jaic` side by side). The project's build metaprogram
//! names its own executable, which `jacket` cannot work out from the command line.
use std::path::{Path, PathBuf};
use std::process::Command;

fn target_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_jacket"))
        .parent()
        .expect("jacket lives in a directory")
        .to_path_buf()
}

fn stdlib() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib")
}

/// A scratch directory of its own, so tests do not share files.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jacket-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

/// `jacket` and `jaic` copied into a fresh directory, as an unpacked archive has them. `None`
/// when `jaic` has not been built (`cargo test -p jacket` alone does not build it).
fn archive(root: &Path) -> Option<PathBuf> {
    let suffix = std::env::consts::EXE_SUFFIX;
    let jaic = target_dir().join(format!("jaic{suffix}"));
    if !jaic.is_file() {
        eprintln!("skipped: build `jaic` first (`cargo build -p jaic-cli`)");
        return None;
    }
    let bin = root.join("archive");
    std::fs::create_dir_all(&bin).expect("create the archive directory");
    std::fs::copy(&jaic, bin.join(format!("jaic{suffix}"))).expect("copy jaic");
    let jacket = bin.join(format!("jacket{suffix}"));
    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).expect("copy jacket");
    Some(jacket)
}

fn jacket(exe: &Path, dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(exe)
        .args(args)
        .current_dir(dir)
        .env("JAI_STDLIB", stdlib())
        .env("JAIC_STDLIB", stdlib())
        .output()
        .expect("start jacket")
}

#[test]
fn run_starts_the_program_a_new_project_builds() {
    let root = scratch("new");
    let Some(exe) = archive(&root) else {
        return;
    };
    let made = jacket(&exe, &root, &["new", "demo"]);
    assert!(made.status.success(), "{made:?}");
    let out = jacket(&exe, &root.join("demo"), &["run"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("Hello from demo!"),
        "{out:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_starts_a_program_built_from_one_file() {
    let root = scratch("file");
    let Some(exe) = archive(&root) else {
        return;
    };
    std::fs::write(
        root.join("hi.jai"),
        "#import \"Basic\";\nmain :: () { print(\"plain file\\n\"); }\n",
    )
    .expect("write the source");
    let out = jacket(&exe, &root, &["run", "hi.jai", "-exe", "named"]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("plain file"));
    let _ = std::fs::remove_dir_all(&root);
}

/// `jacket run` names the project in jaic's status lines and adds its own `Running` line.
#[test]
fn run_shows_the_build_status() {
    let root = scratch("status");
    let Some(exe) = archive(&root) else {
        return;
    };
    let made = jacket(&exe, &root, &["new", "demo"]);
    assert!(made.status.success(), "{made:?}");
    let out = Command::new(&exe)
        .arg("run")
        .current_dir(root.join("demo"))
        .env("JAI_STDLIB", stdlib())
        .env("JAIC_STDLIB", stdlib())
        .env("JAIC_PROGRESS", "plain")
        .output()
        .expect("start jacket");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    let lines: Vec<&str> = stderr.lines().collect();
    assert!(lines[0].starts_with("   Compiling demo ("), "{stderr}");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("    Finished debug [unoptimized] in ")),
        "{stderr}"
    );
    assert!(
        lines
            .last()
            .is_some_and(|l| l.starts_with("     Running `")),
        "{stderr}"
    );
    assert!(!stderr.contains('\x1b'), "{stderr}");
    let _ = std::fs::remove_dir_all(&root);
}
