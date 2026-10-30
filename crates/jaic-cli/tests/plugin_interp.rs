//! `jaic -interp +Plugin` runs the program the plugins compile: their code added before it
//! runs, their message loop, the program's arguments and its exit status.
use test_support::TempDir;

use std::{path::PathBuf, process::Command};

fn program(name: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new(name);
    let source = test_support::plugin::write_program(&dir);
    std::fs::write(
        &source,
        r#"#import "Basic";
main :: () {
    args := get_command_line_arguments();
    print("added %, args %\n", from_plugin(), args.count);
    for 1..args.count - 1  print("arg %\n", args[it]);
    exit(7);
}
"#,
    )
    .unwrap();
    (dir, source)
}

// rules: plugin.10
#[test]
fn the_program_runs_with_what_the_plugins_added_and_reports_its_status() {
    let (_dir, source) = program("cli-plugin-interp");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-interp")
        .arg(&source)
        .args(["+Echo_Plugin", "-shout", "--", "one", "two"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    // The plugin's `finish` ran when the compile ended, before the program started.
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "LOUD finished, typechecked: true\nadded 42, args 3\narg one\narg two\n"
    );
}

#[test]
fn a_plugin_that_fails_stops_the_run() {
    let (_dir, source) = program("cli-plugin-interp-refused");
    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
        .arg("-interp")
        .arg(&source)
        .args(["+Echo_Plugin", "-refuse"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("added"));
}
