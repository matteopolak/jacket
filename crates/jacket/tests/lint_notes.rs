//! `jacket lint` leaves the configuration notes to `jailint`, which prints them itself.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn lint_prints_each_note_once() {
    let root = std::env::temp_dir().join(format!("jacket-test-lint-notes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("create the archive directory");
    let jacket = bin.join("jacket");
    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).expect("copy jacket");
    // A stand-in `jailint` that reports the note as the real one does.
    let lint = bin.join("jailint");
    std::fs::write(
        &lint,
        "#!/bin/sh\necho 'jailint: warning: unknown setting `lint.max_nestng` (ignored)' >&2\n",
    )
    .expect("write the stand-in");
    std::fs::set_permissions(&lint, std::fs::Permissions::from_mode(0o755)).expect("make it run");
    let project: PathBuf = root.join("project");
    std::fs::create_dir_all(&project).expect("create the project");
    std::fs::write(project.join("jacket.toml"), "[lint]\nmax_nestng = 3\n").expect("write config");

    let out = Command::new(&jacket)
        .arg("lint")
        .current_dir(&project)
        .output()
        .expect("start jacket");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr.matches("max_nestng").count(), 1, "{stderr}");
    assert!(stderr.starts_with("jailint:"), "{stderr}");
    let _ = std::fs::remove_dir_all(&root);
}
