//! Every program in `examples/` passes `jaic -check` for each OS it supports. A file that another
//! example `#load`s is part of that program and is checked through it (the tour's sections, the
//! notes API of the HTTP examples). A new example must be added to `PROGRAMS`.
use std::path::{Path, PathBuf};
use std::process::Command;

const ALL: &[&str] = &["linux", "windows", "macos", "wasm"];
const NATIVE: &[&str] = &["linux", "windows", "macos"];

/// Each program, the OSes it is checked for, and why the others are left out.
const PROGRAMS: &[(&str, &[&str], &str)] = &[
    ("compile-time-record.jai", ALL, ""),
    ("foreign-libraries.jai", ALL, ""),
    ("sum.jai", ALL, ""),
    ("type-variants.jai", ALL, ""),
    ("using-records.jai", ALL, ""),
    ("tour/main.jai", ALL, ""),
    ("webgpu/compute.jai", ALL, ""),
    (
        "webgpu/triangle.jai",
        NATIVE,
        "draws into a Window_Creation window; the browser has no Window_Type",
    ),
    (
        "webgpu/raymarch.jai",
        NATIVE,
        "draws into a Window_Creation window; the browser has no Window_Type",
    ),
    (
        "http/client.jai",
        NATIVE,
        "Extensions/Http needs sockets, which the browser has not",
    ),
    (
        "http/crud.jai",
        NATIVE,
        "Extensions/Http needs sockets, which the browser has not",
    ),
    (
        "http/crud_test.jai",
        NATIVE,
        "Extensions/Http needs sockets, which the browser has not",
    ),
    (
        "http/middleware.jai",
        NATIVE,
        "Extensions/Http needs sockets, which the browser has not",
    ),
];

fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn jai_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            jai_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "jai") {
            out.push(path);
        }
    }
}

/// The files `file` names in a `#load "..."`, resolved next to it.
fn loads(file: &Path) -> Vec<PathBuf> {
    let text = std::fs::read_to_string(file).unwrap();
    let dir = file.parent().unwrap();
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("#load \"")?.split('"').next())
        .map(|name| dir.join(name).canonicalize().unwrap())
        .collect()
}

fn relative(path: &Path) -> String {
    let root = examples_dir().canonicalize().unwrap();
    path.strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn every_example_is_a_listed_program_or_loaded_by_one() {
    let mut files = Vec::new();
    jai_files(&examples_dir().canonicalize().unwrap(), &mut files);
    let loaded: Vec<PathBuf> = files.iter().flat_map(|f| loads(f)).collect();
    for file in &files {
        let name = relative(file);
        let listed = PROGRAMS.iter().any(|(program, ..)| *program == name);
        assert!(
            listed || loaded.contains(file),
            "examples/{name} is neither in PROGRAMS nor #load-ed by another example"
        );
        assert!(
            !(listed && loaded.contains(file)),
            "examples/{name} is #load-ed by another example, so it is not a program of its own"
        );
    }
    for (program, oses, reason) in PROGRAMS {
        assert!(
            examples_dir().join(program).is_file(),
            "examples/{program} is gone"
        );
        assert_eq!(
            reason.is_empty(),
            oses.len() == ALL.len(),
            "examples/{program}: give the reason exactly when an OS is left out"
        );
    }
}

#[test]
fn every_example_program_checks_on_its_oses() {
    let cases: Vec<(&str, &str)> = PROGRAMS
        .iter()
        .flat_map(|(program, oses, _)| oses.iter().map(move |os| (*program, *os)))
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failures = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..3 {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some((program, os)) = cases.get(index) else {
                        break;
                    };
                    let output = Command::new(env!("CARGO_BIN_EXE_jaic"))
                        .arg(examples_dir().join(program))
                        .args(["-check", "-quiet", "-os", os])
                        .output()
                        .unwrap();
                    if !output.status.success() {
                        failures.lock().unwrap().push(format!(
                            "examples/{program} -os {os}:\n{}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
