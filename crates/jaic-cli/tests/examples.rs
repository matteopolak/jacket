//! Every program in `examples/` passes `jaic -check` for each OS it supports. A file that another
//! example `#load`s is part of that program and is checked through it (a workspace's sections, the
//! notes API of the HTTP examples). A new example must be added to `PROGRAMS` and have a case in
//! `tests/corpus/examples.json`, which `tools/differential/jaic_sweep.py examples` runs.
use std::path::{Path, PathBuf};
use std::process::Command;

const ALL: &[&str] = &["linux", "windows", "macos", "wasm"];
const NATIVE: &[&str] = &["linux", "windows", "macos"];
const NO_WINDOW: &str = "draws into a Window_Creation window; the browser has no Window_Type";
const NO_SOCKETS: &str = "Extensions/Http needs sockets, which the browser has not";
const NO_FILES: &str = "reads and writes the file system, which the browser sandbox has not";

/// Each program, the OSes it is checked for, and why the others are left out.
const PROGRAMS: &[(&str, &[&str], &str)] = &[
    ("arrays-and-strings/main.jai", ALL, ""),
    ("basics/main.jai", ALL, ""),
    ("compile-time-record/main.jai", ALL, ""),
    ("files/main.jai", ALL, ""),
    ("foreign-libraries/main.jai", ALL, ""),
    ("machine/main.jai", ALL, ""),
    ("memory/main.jai", ALL, ""),
    ("metaprogramming/main.jai", ALL, ""),
    ("polymorphism/main.jai", ALL, ""),
    ("raymarch/main.jai", ALL, ""),
    ("raymarch-gpu/main.jai", ALL, ""),
    ("runtime-safety/main.jai", ALL, ""),
    ("sum/main.jai", ALL, ""),
    ("textured-cube/main.jai", ALL, ""),
    ("threads/main.jai", ALL, ""),
    ("type-variants/main.jai", ALL, ""),
    ("using-records/main.jai", ALL, ""),
    ("webgpu-compute/main.jai", ALL, ""),
    ("webgpu-raymarch/main.jai", NATIVE, NO_WINDOW),
    ("webgpu-triangle/main.jai", NATIVE, NO_WINDOW),
    ("http/client.jai", NATIVE, NO_SOCKETS),
    ("http/crud.jai", NATIVE, NO_SOCKETS),
    ("http/crud_test.jai", NATIVE, NO_SOCKETS),
    ("http/middleware.jai", NATIVE, NO_SOCKETS),
    ("editor/main.jai", NATIVE, NO_WINDOW),
    ("editor/smoke.jai", NATIVE, NO_FILES),
    ("ui-gallery/main.jai", NATIVE, NO_WINDOW),
    ("game2d/game/main.jai", NATIVE, NO_WINDOW),
    ("game3d/game/main.jai", NATIVE, NO_WINDOW),
    ("game2d/server/main.jai", NATIVE, NO_SOCKETS),
    ("game2d/loadtest/main.jai", NATIVE, NO_SOCKETS),
    ("game2d/tools/assetgen/main.jai", NATIVE, NO_FILES),
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
    jacket_config::slashed(path.strip_prefix(&root).unwrap())
}

#[test]
fn every_example_is_a_listed_program_or_loaded_by_one() {
    let mut files = Vec::new();
    jai_files(&examples_dir().canonicalize().unwrap(), &mut files);
    let loaded: Vec<PathBuf> = files.iter().flat_map(|f| loads(f)).collect();
    for file in &files {
        let name = relative(file);
        let listed = PROGRAMS.iter().any(|(program, ..)| *program == name);
        // A local module is `#import`-ed by the programs next to it, and checked through them.
        let module = file.file_name().is_some_and(|n| n == "module.jai");
        assert!(
            listed || module || loaded.contains(file),
            "examples/{name} is neither in PROGRAMS, a module, nor #load-ed by another example"
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

/// Every folder of `examples/` is one workspace with a case of the same name in the corpus, which
/// the sweep runs and (with `bundle`) the browser bundle lists.
#[test]
fn every_example_folder_has_a_corpus_case() {
    let corpus: serde_json::Value = serde_json::from_slice(
        &std::fs::read(examples_dir().join("../tests/corpus/examples.json")).unwrap(),
    )
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    let mut folders = Vec::new();
    for entry in std::fs::read_dir(examples_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            folders.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    assert!(!folders.is_empty());
    for folder in &folders {
        let case = cases
            .iter()
            .find(|case| case["directory"] == format!("examples/{folder}"))
            .unwrap_or_else(|| {
                panic!("examples/{folder} has no case in tests/corpus/examples.json")
            });
        assert_eq!(
            case["id"],
            folder.as_str(),
            "the case of examples/{folder} is named for it"
        );
        let main = case["main"].as_str().unwrap();
        assert!(
            PROGRAMS
                .iter()
                .any(|(program, ..)| *program == format!("{folder}/{main}")),
            "examples/{folder}/{main} is not in PROGRAMS"
        );
    }
    for case in cases {
        let directory = case["directory"].as_str().unwrap();
        assert!(
            folders
                .iter()
                .any(|folder| directory == format!("examples/{folder}")),
            "{directory} is gone"
        );
    }
}
