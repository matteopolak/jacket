//! `jacket run` on a project `jacket new` made, with the tools laid out like a release archive
//! (`jacket` and `jaic` side by side). The project's build metaprogram
//! names its own executable, which `jacket` cannot work out from the command line.
#[path = "support/installed.rs"]
mod installed;
use installed::{archive, jacket, scratch, stdlib};
use std::process::Command;

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
}
