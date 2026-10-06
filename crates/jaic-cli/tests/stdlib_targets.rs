//! Every stdlib module type-checks for every target OS, including the code nothing calls.
//!
//! `jaic check -no_dce -os <os> -cpu <cpu>` on a program that only imports the module checks every
//! procedure body and declaration in it, so code for other platforms and procedures no test
//! calls cannot hide type errors. Modules that refuse a target on purpose (`#assert(OS == ...)`)
//! are listed in `tests/stdlib-targets.txt` with the first error they stop at; the test fails when
//! a module fails that is not listed, fails differently, or passes although it is listed.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// `os-cpu` names; the CPU is always given, so the result does not depend on the host's.
const TARGETS: [&str; 7] = [
    "linux-x64",
    "linux-arm64",
    "macos-x64",
    "macos-arm64",
    "windows-x64",
    "windows-arm64",
    "wasm",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `stdlib/X.jai` and `stdlib/X/module.jai`, by import name.
fn modules(stdlib: &Path, skip: &[String]) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(stdlib)
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            // A folder's whole name (`freetype-2.12.1`), a file's without `.jai`.
            let name = if path.is_dir() {
                path.file_name()
            } else {
                path.file_stem()
            }?
            .to_str()?
            .to_string();
            let module = if path.is_dir() {
                path.join("module.jai").is_file()
            } else {
                path.extension().is_some_and(|x| x == "jai")
            };
            module.then_some(name)
        })
        .filter(|name| !skip.contains(name))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// The expectations file: `skip Module` lines and `Module os: file: message` lines.
fn expectations(text: &str) -> (Vec<String>, BTreeMap<String, String>) {
    let mut skip = Vec::new();
    let mut failures = BTreeMap::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix("skip ") {
            skip.push(name.trim().to_string());
        } else {
            let (key, error) = line.split_once(": ").expect("`Module os: error` line");
            failures.insert(key.to_string(), error.to_string());
        }
    }
    (skip, failures)
}

/// The first error of a failed check, as `file: message` with the stdlib path and the line and
/// column removed so that unrelated edits to the file do not change it.
fn first_error(stderr: &str, stdlib: &str) -> String {
    let line = stderr
        .lines()
        .find(|l| l.contains("error: "))
        .unwrap_or("no error message");
    let line = line.replace(stdlib, "");
    let (location, message) = line.split_once(": error: ").unwrap_or(("", &line));
    let file = location.split(':').next().unwrap_or("");
    format!("{file}: {message}")
}

#[test]
fn every_stdlib_module_checks_for_every_target() {
    let root = root().canonicalize().unwrap();
    let stdlib = root.join("stdlib");
    let stdlib_prefix = format!("{}/", stdlib.display());
    let listed = root.join("tests/stdlib-targets.txt");
    let (skip, expected) = expectations(&std::fs::read_to_string(&listed).unwrap());
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stdlib-targets");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let jobs: Vec<(String, &str)> = modules(&stdlib, &skip)
        .into_iter()
        .flat_map(|m| TARGETS.map(|os| (m.clone(), os)))
        .collect();
    let next = Mutex::new(jobs.into_iter());
    let actual = Mutex::new(BTreeMap::new());
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    let Some((module, os)) = next.lock().unwrap().next() else {
                        return;
                    };
                    let source = dir.join(format!("{module}.{os}.jai"));
                    std::fs::write(&source, format!("#import \"{module}\";\nmain :: () {{}}\n"))
                        .unwrap();
                    let (target_os, cpu) = os.split_once('-').unwrap_or((os, ""));
                    let mut command = Command::new(JAIC);
                    command
                        .arg("check")
                        .arg(&source)
                        .args(["-no_dce", "-os", target_os]);
                    if !cpu.is_empty() {
                        command.args(["-cpu", cpu]);
                    }
                    let output = command.env("JAIC_STDLIB", &stdlib).output().unwrap();
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        actual.lock().unwrap().insert(
                            format!("{module} {os}"),
                            first_error(&stderr, &stdlib_prefix),
                        );
                    }
                }
            });
        }
    });

    let actual = actual.into_inner().unwrap();
    let mut problems = Vec::new();
    for (key, error) in &actual {
        match expected.get(key) {
            None => problems.push(format!("new failure: {key}: {error}")),
            Some(want) if want != error => {
                problems.push(format!("{key} now fails with `{error}` (listed: `{want}`)"));
            }
            Some(_) => {}
        }
    }
    for key in expected.keys() {
        if !actual.contains_key(key) {
            problems.push(format!(
                "{key} passes now: remove it from tests/stdlib-targets.txt"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "{} of the stdlib/target checks differ from tests/stdlib-targets.txt:\n{}",
        problems.len(),
        problems.join("\n")
    );
}
