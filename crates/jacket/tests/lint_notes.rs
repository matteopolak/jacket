//! `jacket lint` leaves the linter's notes to `jailint`, which prints them itself, and reports a
//! configuration error once, itself, before it starts `jailint`.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

use test_support::TempDir;

/// Run `jacket lint` in a project with `config` as its `jacket.toml`, beside a stand-in `jailint`
/// that prints one note.
fn lint(name: &str, config: &str) -> Output {
    let root = TempDir::new(name);
    let bin = root.join("bin");

    std::fs::create_dir_all(&bin).expect("create the archive directory");
    let jacket = bin.join("jacket");

    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).expect("copy jacket");
    let stand_in = bin.join("jailint");

    std::fs::write(
        &stand_in,
        "#!/bin/sh\necho 'jailint: warning: note from the linter' >&2\n",
    )
    .expect("write the stand-in");
    std::fs::set_permissions(&stand_in, std::fs::Permissions::from_mode(0o755))
        .expect("make it run");
    let project: PathBuf = root.join("project");

    std::fs::create_dir_all(&project).expect("create the project");
    std::fs::write(project.join("jacket.toml"), config).expect("write config");
    Command::new(&jacket)
        .arg("lint")
        .current_dir(&project)
        .output()
        .expect("start jacket")
}

#[test]
fn lint_prints_each_note_once() {
    let out = lint("jacket-test-lint-notes", "[lint]\nmax-nesting = 3\n");
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert_eq!(
        stderr.matches("note from the linter").count(),
        1,
        "{stderr}"
    );
    assert!(stderr.starts_with("jailint:"), "{stderr}");
}

#[test]
fn lint_reports_an_unknown_setting_once() {
    let out = lint("jacket-test-lint-unknown", "[lint]\nmax-nestng = 3\n");
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert_eq!(stderr.matches("max-nestng").count(), 1, "{stderr}");
    assert!(stderr.starts_with("jacket: error:"), "{stderr}");
    assert!(
        stderr.contains("did you mean `lint.max-nesting`?"),
        "{stderr}"
    );
    assert!(!stderr.contains("note from the linter"), "{stderr}");
    assert!(!out.status.success());
}
