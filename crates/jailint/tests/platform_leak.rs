//! `platform_leak`: portable code that imports something an operating system does not build.
use std::{path::Path, process::Command};
use test_support::TempDir;

/// A project with a small library under `lib/` held to every system, and a standard library
/// whose `POSIX` module asserts a Unix target.
fn project(name: &str, settings: &str) -> TempDir {
    let root = TempDir::new(name);
    for dir in [
        "stdlib/prelude",
        "stdlib/Runtime_Support",
        "stdlib/POSIX",
        "lib",
    ] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    let write = |path: &str, text: &str| std::fs::write(root.join(path), text).unwrap();
    write(
        "stdlib/prelude/module.jai",
        "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }",
    );
    write(
        "stdlib/Runtime_Support/module.jai",
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);",
    );
    write(
        "stdlib/POSIX/module.jai",
        "#assert OS == .LINUX || OS == .MACOS, \"POSIX needs a Unix\";\ngetpid :: () -> s32 { return 1; }\n",
    );
    write("jacket.toml", settings);
    root
}

const UNIX_ONLY: &str = "[lint.portability]\nportable = [\"lib/**\"]\n\n[lint.portability.platforms]\n\"stdlib/POSIX/**\" = [\"linux\", \"macos\"]\n";

/// The project's settings with `platform_leak` on.
fn enabled(extra: &str) -> String {
    format!("[lint.rules]\nplatform_leak = \"warn\"\n\n{UNIX_ONLY}{extra}")
}

/// Run `jailint -A all -W platform_leak` on `path`: standard output and the exit code.
fn lint(root: &Path, path: &str) -> (String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .current_dir(root)
        .env("JAIC_STDLIB", root.join("stdlib"))
        .args(["--color", "never", "-A", "all", "-W", "platform_leak", path])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let code = output.status.code().unwrap();
    assert!(code == 0 || code == 1, "{stderr}");
    (String::from_utf8(output.stdout).unwrap(), code)
}

fn write(root: &Path, path: &str, text: &str) {
    std::fs::write(root.join(path), text).unwrap();
}

#[test]
fn an_import_of_a_unix_module_is_flagged_for_windows_only() {
    let root = project("flagged", &enabled(""));
    write(
        &root,
        "lib/a.jai",
        "#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let (out, code) = lint(&root, "lib/a.jai");
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("warning[platform_leak]: POSIX is not available on windows"),
        "{out}"
    );
    assert!(
        !out.contains("available on linux") && !out.contains("available on macos"),
        "{out}"
    );
    assert!(out.contains("--> lib/a.jai:1:1"), "{out}");
    assert!(out.contains("#import \"POSIX\";"), "{out}");
    assert!(out.contains("only builds on linux and macos"), "{out}");
    assert!(
        out.contains("= note: the `#assert` at stdlib/POSIX/module.jai:1:9 fails on windows: POSIX needs a Unix"),
        "{out}"
    );
    assert_eq!(out.matches("platform_leak]").count(), 1, "{out}");
}

#[test]
fn the_rule_is_off_unless_it_is_turned_on() {
    let root = project("off", UNIX_ONLY);
    write(
        &root,
        "lib/a.jai",
        "#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .current_dir(&root)
        .env("JAIC_STDLIB", root.join("stdlib"))
        .args(["--color", "never", "-A", "unused_import", "lib/a.jai"])
        .output()
        .unwrap();
    let out = String::from_utf8(output.stdout).unwrap();
    assert!(!out.contains("platform_leak"), "{out}");
    assert_eq!(output.status.code(), Some(0), "{out}");
    let listing = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .arg("--list")
        .output()
        .unwrap();
    let listing = String::from_utf8(listing.stdout).unwrap();
    let line = listing
        .lines()
        .find(|l| l.starts_with("platform_leak"))
        .unwrap();
    assert!(
        line.contains("restriction") && line.contains("allow"),
        "{line}"
    );
}

#[test]
fn a_platforms_glob_silences_the_import_on_the_systems_it_leaves_out() {
    let root = project("glob", &enabled("\"lib/a.jai\" = [\"linux\", \"macos\"]\n"));
    write(
        &root,
        "lib/a.jai",
        "#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let (out, code) = lint(&root, "lib/a.jai");
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("platform_leak"), "{out}");
}

#[test]
fn a_platforms_comment_in_the_file_silences_it_too() {
    let root = project("comment", &enabled(""));
    write(
        &root,
        "lib/a.jai",
        "// Unix only.\n// jailint: platforms(linux, macos)\n#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib/a.jai");
    assert!(!out.contains("platform_leak"), "{out}");
    // A comment that lists Windows does not.
    write(
        &root,
        "lib/a.jai",
        "// jailint: platforms(linux, windows)\n#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib/a.jai");
    assert!(out.contains("POSIX is not available on windows"), "{out}");
    assert!(!out.contains("available on linux"), "{out}");
    // Once code starts, a later comment is not read.
    write(
        &root,
        "lib/a.jai",
        "#import \"POSIX\";\n// jailint: platforms(linux, macos)\nmain :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib/a.jai");
    assert!(out.contains("POSIX is not available on windows"), "{out}");
}

#[test]
fn an_allow_comment_covers_the_import() {
    let root = project("allow", &enabled(""));
    write(
        &root,
        "lib/a.jai",
        "// jailint: allow(platform_leak)\n#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib/a.jai");
    assert!(!out.contains("platform_leak"), "{out}");
}

#[test]
fn a_chain_is_reported_at_the_first_import_that_leaves_the_portable_code() {
    // lib/outer.jai -> lib/middle.jai -> POSIX: both files are portable, so the leak is the
    // import in the middle.
    let root = project("chain", &enabled(""));
    write(
        &root,
        "lib/outer.jai",
        "#load \"middle.jai\";\nmain :: () { helper(); }\n",
    );
    write(
        &root,
        "lib/middle.jai",
        "#import \"POSIX\";\nhelper :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib/outer.jai");
    assert_eq!(out.matches("platform_leak]").count(), 1, "{out}");
    assert!(out.contains("--> lib/middle.jai:1:1"), "{out}");

    // With the middle file limited to Unix, the portable code that loads it is the leak.
    let root = project(
        "chain-limited",
        &enabled("\"lib/middle.jai\" = [\"linux\", \"macos\"]\n"),
    );
    write(
        &root,
        "lib/outer.jai",
        "#load \"middle.jai\";\nmain :: () { helper(); }\n",
    );
    write(
        &root,
        "lib/middle.jai",
        "#import \"POSIX\";\nhelper :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib/outer.jai");
    assert_eq!(out.matches("platform_leak]").count(), 1, "{out}");
    assert!(
        out.contains("middle.jai is not available on windows"),
        "{out}"
    );
    assert!(out.contains("--> lib/outer.jai:1:7"), "{out}");
    assert!(out.contains("#load \"middle.jai\""), "{out}");
}

#[test]
fn every_leak_is_reported_not_only_the_first() {
    let root = project("every", &enabled(""));
    write(
        &root,
        "lib/a.jai",
        "#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    write(
        &root,
        "lib/b.jai",
        "#import \"POSIX\";\nrun :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "lib");
    assert_eq!(out.matches("platform_leak]").count(), 2, "{out}");
    assert!(
        out.contains("--> lib/a.jai:1:1") && out.contains("--> lib/b.jai:1:1"),
        "{out}"
    );
    // Two imports in one file, each failing.
    std::fs::write(
        root.join("stdlib/POSIX/other.jai"),
        "#assert OS == .LINUX, \"other\";\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("stdlib/Other")).unwrap();
    write(
        &root,
        "stdlib/Other/module.jai",
        "#assert OS == .LINUX, \"Other needs Linux\";\n",
    );
    std::fs::remove_file(root.join("lib/b.jai")).unwrap();
    write(
        &root,
        "lib/a.jai",
        "#import \"POSIX\";\n#import \"Other\";\nmain :: () { getpid(); }\n",
    );
    let settings = enabled("\"stdlib/Other/**\" = [\"linux\"]\n");
    write(&root, "jacket.toml", &settings);
    let (out, _) = lint(&root, "lib/a.jai");
    assert!(out.contains("POSIX is not available on windows"), "{out}");
    assert!(out.contains("Other is not available on windows"), "{out}");
    assert!(out.contains("Other is not available on macos"), "{out}");
    assert_eq!(out.matches("platform_leak]").count(), 3, "{out}");
}

#[test]
fn an_assert_in_a_portable_file_is_reported_on_the_assert() {
    let root = project("own", &enabled(""));
    write(
        &root,
        "lib/a.jai",
        "#assert OS == .LINUX, \"Linux only\";\nmain :: () {}\n",
    );
    let (out, _) = lint(&root, "lib/a.jai");
    assert!(out.contains("a.jai is not available on macos"), "{out}");
    assert!(out.contains("a.jai is not available on windows"), "{out}");
    assert!(out.contains("--> lib/a.jai:1:9"), "{out}");
    assert!(out.contains("fails on windows"), "{out}");
}

#[test]
fn code_outside_the_portable_globs_may_import_anything() {
    let root = project("outside", &enabled(""));
    std::fs::create_dir_all(root.join("app")).unwrap();
    write(
        &root,
        "app/main.jai",
        "#import \"POSIX\";\nmain :: () { getpid(); }\n",
    );
    let (out, _) = lint(&root, "app/main.jai");
    assert!(!out.contains("platform_leak"), "{out}");
}

#[test]
fn unknown_systems_are_a_configuration_error() {
    let root = project(
        "unknown-os",
        "[lint.portability]\ntargets = [\"beos\"]\nportable = [\"lib/**\"]\n",
    );
    write(&root, "lib/a.jai", "main :: () {}\n");
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .current_dir(&root)
        .env("JAIC_STDLIB", root.join("stdlib"))
        .args(["--color", "never", "lib/a.jai"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("unknown operating system \"beos\""),
        "{stderr}"
    );
}
