//! A dependency's own `import-path` reaches the compiler as `-package_import_dir`, with a
//! compiler that only records its arguments.
#![cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use test_support::TempDir;

const COMPILER: &str =
    "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > \"$ARGS_LOG\"\n";

fn run(files: &[(&str, &str)], args: &[&str]) -> (TempDir, std::process::Output) {
    let root = TempDir::new("package-roots");

    for (path, text) in files {
        let full = root.join(path);

        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }

    let compiler = root.join("jaic");

    std::fs::write(&compiler, COMPILER).unwrap();
    std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o755)).unwrap();
    let jacket = root.join("jacket");

    std::fs::copy(env!("CARGO_BIN_EXE_jacket"), &jacket).unwrap();
    let log = root.join("args.txt");
    let output = Command::new(&jacket)
        .current_dir(&root)
        .args(args)
        .env("ARGS_LOG", &log)
        .output()
        .unwrap();

    (root, output)
}

fn seen(files: &[(&str, &str)], args: &[&str]) -> (TempDir, Vec<String>) {
    let (root, output) = run(files, args);

    assert!(output.status.success(), "{output:?}");
    let log = root.join("args.txt");
    let lines = std::fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();

    (root, lines)
}

const TOML: &str = "[dependencies]\ndep = { git = \"x/dep\", tag = \"v1\" }\nmoved = { git = \"x/moved\", tag = \"v1\", path = \"vendor/moved\" }\nplain = { git = \"x/plain\", tag = \"v1\" }\n";

#[test]
fn each_dependency_with_import_folders_is_passed_as_its_own_flags() {
    let (root, args) = seen(
        &[
            ("jacket.toml", TOML),
            ("modules/dep/.git", "gitdir: x"),
            ("vendor/moved/.git", "gitdir: x"),
            ("modules/plain/.git", "gitdir: x"),
            (
                "modules/dep/jacket.toml",
                "[package]\nimport-path = [\"lib\", \"more\"]\n",
            ),
            (
                "vendor/moved/jacket.toml",
                "[package]\nimport-path = \"v\"\n",
            ),
            ("modules/plain/jacket.toml", "[package]\nname = \"plain\"\n"),
        ],
        &["build", "main.jai"],
    );
    let at = |p: &str| root.join(p).display().to_string();
    let expected: Vec<String> = [
        ("modules/dep", "modules/dep/lib"),
        ("modules/dep", "modules/dep/more"),
        ("vendor/moved", "vendor/moved/v"),
    ]
    .iter()
    .flat_map(|(package, dir)| {
        [
            "-package_import_dir".to_string(),
            format!("{}={}", at(package), at(dir)),
        ]
    })
    .collect();

    assert_eq!(args[..args.len() - 1], expected[..], "{args:?}");
    assert_eq!(args.last().map(String::as_str), Some("main.jai"));
}

#[test]
fn a_module_below_the_repository_root_is_found_through_its_parent_folder() {
    let (root, args) = seen(
        &[
            (
                "jacket.toml",
                "[dependencies]\nwrap = { git = \"x/wrap\", tag = \"v1\" }\nrecorded = { git = \"x/rec\", tag = \"v1\", module = \"src/Deep\" }\n",
            ),
            ("modules/wrap/.git", "gitdir: x"),
            (
                "modules/wrap/jacket.toml",
                "[package]\nmodule = \"modules/Inner\"\n",
            ),
            ("modules/recorded/.git", "gitdir: x"),
        ],
        &["build", "main.jai"],
    );
    let at = |p: &str| root.join(p).display().to_string();

    assert_eq!(
        args,
        [
            "-import_dir",
            &at("modules/recorded/src"),
            "-import_dir",
            &at("modules/wrap/modules"),
            "main.jai"
        ]
    );
}

#[test]
fn a_dependency_manifest_that_cannot_be_read_stops_the_build_and_names_the_file() {
    let (root, output) = run(
        &[
            ("jacket.toml", TOML),
            ("modules/dep/.git", "gitdir: x"),
            ("vendor/moved/.git", "gitdir: x"),
            ("modules/plain/.git", "gitdir: x"),
            ("modules/dep/jacket.toml", "[package\n"),
        ],
        &["build", "main.jai"],
    );

    assert!(!output.status.success(), "{output:?}");
    let err = String::from_utf8_lossy(&output.stderr);

    assert!(
        err.contains(&root.join("modules/dep/jacket.toml").display().to_string()),
        "{err}"
    );
}
