//! `jacket fmt` starts `jaifmt` with the arguments as written and its exit status, and reads no
//! configuration itself.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use test_support::TempDir;

#[test]
fn fmt_forwards_every_argument_and_the_exit_status() {
    let root = TempDir::new("jacket-test-fmt-passthrough");
    let bin = root.join("bin");

    std::fs::create_dir_all(&bin).expect("create the archive directory");
    let jacket = bin.join("jacket");

    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).expect("copy jacket");
    let stand_in = bin.join("jaifmt");

    // Prints its arguments one per line and the working directory, then exits with 1.
    std::fs::write(
        &stand_in,
        "#!/bin/sh\nfor a in \"$@\"; do echo \"arg:$a\"; done\npwd\nexit 1\n",
    )
    .expect("write the stand-in");
    std::fs::set_permissions(&stand_in, std::fs::Permissions::from_mode(0o755))
        .expect("make it run");
    let project = root.join("project");

    std::fs::create_dir_all(project.join("src")).expect("create the project");
    // A setting jacket would refuse: the passthrough leaves it to `jaifmt`.
    std::fs::write(project.join("jacket.toml"), "[format]\nindent-width = 99\n")
        .expect("write config");
    let out = Command::new(&jacket)
        .args([
            "fmt",
            "--check",
            "--set",
            "max-width=80",
            "src",
            "--config",
            "x.toml",
        ])
        .current_dir(project.join("src"))
        .output()
        .expect("start jacket");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.starts_with(
            "arg:--check\narg:--set\narg:max-width=80\narg:src\narg:--config\narg:x.toml\n"
        ),
        "{stdout}"
    );
    assert!(stdout.trim_end().ends_with("/project/src"), "{stdout}");
}
