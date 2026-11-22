//! `--target NAME` and `jacket targets`, with a compiler that only records its arguments.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output};
use test_support::TempDir;

const COMPILER: &str =
    "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > \"$TARGET_LOG\"\n";

const PROJECT: &str = "[target.pico]\ntriple = \"thumbv6m-none-eabi\"\nrunner = \"flash {exe}\"\n\
                       [target.dev]\nboard = \"Qemu_Virt_Aarch64\"\n";

fn project() -> (TempDir, std::path::PathBuf) {
    let root = TempDir::new("named-targets");
    std::fs::write(root.join("jacket.toml"), PROJECT).unwrap();
    let compiler = root.join("jaic");
    std::fs::write(&compiler, COMPILER).unwrap();
    std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o755)).unwrap();
    let jacket = root.join("jacket");
    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).unwrap();
    (root, jacket)
}

fn jacket(root: &TempDir, bin: &std::path::Path, args: &[&str]) -> (Output, Vec<String>) {
    let log = root.join("args.txt");
    let _ = std::fs::remove_file(&log);
    let output = Command::new(bin)
        .current_dir(root)
        .args(args)
        .env("TARGET_LOG", &log)
        .output()
        .unwrap();
    let seen = std::fs::read_to_string(&log)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect();
    (output, seen)
}

#[test]
fn a_named_target_reaches_the_compiler_as_its_triple_and_board_plugin() {
    let (root, bin) = project();
    let (output, seen) = jacket(&root, &bin, &["build", "m.jai", "--target", "pico", "-O2"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(seen, ["m.jai", "-target", "thumbv6m-none-eabi", "-O2"]);

    let (_, seen) = jacket(&root, &bin, &["check", "m.jai", "--target=dev"]);
    assert_eq!(
        seen,
        [
            "-check",
            "m.jai",
            "+Extensions/Embedded/Boards/Qemu_Virt_Aarch64"
        ]
    );
}

#[test]
fn a_triple_still_reaches_the_compiler_untouched() {
    let (root, bin) = project();
    let (_, seen) = jacket(
        &root,
        &bin,
        &["build", "m.jai", "-target", "x86_64-pc-windows-gnu"],
    );
    assert_eq!(seen, ["m.jai", "-target", "x86_64-pc-windows-gnu"]);
}

#[test]
fn an_unknown_target_names_the_nearest_one() {
    let (root, bin) = project();
    let (output, seen) = jacket(&root, &bin, &["build", "m.jai", "--target", "pic"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("unknown target `pic`"), "{stderr}");
    assert!(stderr.contains("did you mean `pico`?"), "{stderr}");
    assert!(seen.is_empty(), "the compiler must not start");
}

#[test]
fn the_targets_command_lists_tables_and_boards() {
    let (root, bin) = project();
    let (output, _) = jacket(&root, &bin, &["targets"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{text}");
    assert!(text.starts_with("TARGET"), "{text}");
    let row = |name: &str| text.lines().find(|l| l.starts_with(name)).unwrap_or("");
    assert!(
        row("pico").contains("thumbv6m-none-eabi") && row("pico").contains("flash (not installed)"),
        "{text}"
    );
    assert!(
        row("dev").contains("Qemu_Virt_Aarch64") && row("dev").contains("project"),
        "{text}"
    );
    assert!(row("Qemu_Virt_Aarch64").contains("built in"), "{text}");
}
