//! Where tests are. A test root is a folder with a `tests/` directory or a `module.jai`:
//!
//! - a `.jai` file anywhere below `tests/` with a `.stdout`, `.stderr` or `.exit` file beside it is a
//!   golden-output test, a program built and run alone; with `// test: compile_fail` on top it
//!   must instead fail to compile, with the diagnostics in its `.stderr`;
//! - `tests/compile_fail/*.jai` must fail to compile, with the diagnostics in its `.stderr`;
//! - every other `tests/*.jai` goes into one build, whose `@test` procedures are the unit tests;
//! - the rest of the files below `tests/` are what those files `#load` or `#import`.
//!
//! The roots are `[test] paths` in jacket.toml, expanded by [`expand`] (only those the current
//! directory is in, when it is in some), else the project's folder.
use jacket_config::{PathPattern, relative_text};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Root {
    pub dir: PathBuf,
    /// The name tests of this root are prefixed with when several roots run (`Bit_Array`).
    pub label: String,
    /// A library (`module.jai`): its parent folder is an import folder, so the tests can
    /// `#import` it by name.
    pub library: bool,
    pub unit: Vec<PathBuf>,
    pub golden: Vec<PathBuf>,
    pub compile_fail: Vec<PathBuf>,
    /// What `jacket bench` builds: `benches/*.jai`, and the `tests/*.jai` that hold a `@bench`.
    pub bench: Vec<PathBuf>,
    /// The name a program imports the library by: its path in the standard library
    /// (`Extensions/Testing`), else its folder's name, found through `-import_dir`.
    pub module: String,
    /// A library's doc tests.
    pub docs: Vec<super::doctest::Block>,
}

pub const EXPECTED: [&str; 3] = ["stdout", "stderr", "exit"];

/// The folders among jaic `flags` that `-import_dir` and `-I` name (`-I=dir` too), as they are
/// on disk.
pub fn import_folders(flags: &[String]) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    let mut flags = flags.iter();
    while let Some(flag) = flags.next() {
        let value = match flag.split_once('=') {
            Some(("-import_dir" | "-I", value)) => Some(value),
            _ if flag == "-import_dir" || flag == "-I" => flags.next().map(String::as_str),
            _ => None,
        };
        if let Some(folder) = value.map(|v| super::watch::canonical(Path::new(v)))
            && folder.is_dir()
        {
            folders.push(folder);
        }
    }
    folders
}

/// The `.jai` files below `dir` at any depth (`tests/` helper folders included), sorted.
fn jai_files_below(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "jai") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Whether the file's top comment block holds `// test: compile_fail` (or `compile_fail(text)`).
fn marked_compile_fail(source: &Path) -> bool {
    let text = std::fs::read_to_string(source).unwrap_or_default();
    super::options::header_notes(&text)
        .iter()
        .any(|n| n == "compile_fail" || n.starts_with("compile_fail("))
}

fn jai_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "jai"))
        .collect();
    files.sort();
    files
}

/// `dir`'s path below `stdlib`, with `/` separators, when it is there.
fn module_name(dir: &Path, stdlib: &Path) -> Option<String> {
    let dir = dir.canonicalize().ok()?;
    let below = dir.strip_prefix(stdlib.canonicalize().ok()?).ok()?;
    let parts: Vec<String> = below
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// Whether the module is one of the standard library's private ones (`internal/hash`): a program
/// cannot import it, so its examples run through the public module that loads it.
fn is_private(module: &str) -> bool {
    module.split('/').next() == Some("internal")
}

/// Whether the file has a procedure with a `@bench` note.
fn holds_bench(source: &Path) -> bool {
    std::fs::read_to_string(source).is_ok_and(|text| text.contains("@bench"))
}

/// Whether `source` has an expected-output file beside it, or says in its header what the
/// diagnostics contain (`compile_fail("text")`), which needs none.
pub fn is_golden(source: &Path) -> bool {
    EXPECTED.iter().any(|e| source.with_extension(e).is_file())
        || super::options::header_notes(&std::fs::read_to_string(source).unwrap_or_default())
            .iter()
            .any(|n| n.starts_with("compile_fail("))
}

impl Root {
    /// `seen` holds the files earlier roots read doc tests from; a file is read for one root.
    pub fn read(
        dir: PathBuf,
        stdlib: &Path,
        jaic: &Path,
        seen: &mut BTreeSet<PathBuf>,
        doc_tests: bool,
    ) -> Root {
        let tests = dir.join("tests");
        let compile_dir = tests.join("compile_fail");
        let (mut unit, mut golden) = (Vec::new(), Vec::new());
        let mut compile_fail = jai_files(&compile_dir);
        for file in jai_files_below(&tests) {
            if file.starts_with(&compile_dir) {
                continue;
            }
            if !is_golden(&file) {
                if file.parent() == Some(tests.as_path()) {
                    unit.push(file);
                }
            } else if marked_compile_fail(&file) {
                compile_fail.push(file);
            } else {
                golden.push(file);
            }
        }
        let label = dir
            .file_name()
            .map_or_else(|| "tests".into(), |n| n.to_string_lossy().into_owned());
        let library = dir.join("module.jai").is_file();
        let module = module_name(&dir, stdlib).unwrap_or_else(|| label.clone());
        let mut bench = jai_files(&dir.join("benches"));
        bench.extend(unit.iter().filter(|file| holds_bench(file)).cloned());
        Root {
            bench,
            docs: if library && doc_tests && !is_private(&module) {
                super::doctest::blocks(&dir, &module, (jaic, stdlib), seen)
            } else {
                Vec::new()
            },
            module,
            library,
            compile_fail,
            dir,
            label,
            unit,
            golden,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.unit.is_empty()
            && self.golden.is_empty()
            && self.compile_fail.is_empty()
            && self.docs.is_empty()
            && self.bench.is_empty()
    }
}

/// Whether `dir` holds tests: a `tests/` folder, or a library's `module.jai`.
fn is_test_root(dir: &Path) -> bool {
    dir.join("tests").is_dir() || dir.join("module.jai").is_file()
}

/// Whether the folder `relative` is ignored: the patterns match a file in it (`fuzz/**` does not
/// match `fuzz` itself).
fn ignored(ignore: &[PathPattern], relative: &str) -> bool {
    let inside = format!("{relative}/x");
    ignore.iter().any(|i| i.is_match(&inside))
}

/// The folders `[test] paths` names, as absolute paths under `project`, sorted.
///
/// A plain path is that folder. A pattern is every test root it matches exactly, found below
/// its plain leading folders. The walk skips hidden entries, `tests/` folders, and folders
/// `ignore` matches (paths relative to `project`).
pub fn expand(
    patterns: &[String],
    project: &Path,
    ignore: &[PathPattern],
) -> Result<Vec<PathBuf>, String> {
    let mut found = BTreeSet::new();
    for text in patterns {
        let pattern = PathPattern::new(text).map_err(|e| e.to_string())?;
        if pattern.is_literal() {
            found.insert(project.join(text));
            continue;
        }
        let base = project.join(pattern.literal_prefix());
        let mut pending = vec![base];
        while let Some(dir) = pending.pop() {
            let relative = relative_text(&dir, project);
            if is_test_root(&dir) && pattern.is_exact_match(&relative) {
                found.insert(dir.clone());
            }
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                let path = entry.path();
                if name.starts_with('.')
                    || name == "tests"
                    || !entry.file_type().is_ok_and(|t| t.is_dir())
                    || ignored(ignore, &relative_text(&path, project))
                {
                    continue;
                }
                pending.push(path);
            }
        }
    }
    Ok(found.into_iter().collect())
}

/// The roots to test: `explicit` folders if any, else `[test] paths` (narrowed to those `cwd` is
/// in, when it is in some), else `project`.
pub fn roots(
    explicit: &[PathBuf],
    configured: &[PathBuf],
    project: &Path,
    cwd: &Path,
) -> Vec<PathBuf> {
    if !explicit.is_empty() {
        return explicit.iter().map(|p| cwd.join(p)).collect();
    }
    if configured.is_empty() {
        return vec![project.to_path_buf()];
    }
    let inside: Vec<PathBuf> = configured
        .iter()
        .filter(|root| cwd.starts_with(root))
        .cloned()
        .collect();
    if inside.is_empty() {
        return configured.to_vec();
    }
    // The innermost: inside `A/B` of roots `A` and `A/B`, only `A/B` runs.
    inside
        .iter()
        .filter(|root| {
            !inside
                .iter()
                .any(|other| other != *root && other.starts_with(root))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_folders_are_the_existing_ones_flags_name() {
        let here = std::env::temp_dir();
        let flags: Vec<String> = [
            "-O1".to_string(),
            "-import_dir".into(),
            here.to_string_lossy().into_owned(),
            "-I".into(),
            "/no/such/folder".into(),
            format!("-I={}", here.display()),
            "-import_dir".into(),
        ]
        .into();
        let folders = import_folders(&flags);
        assert_eq!(folders.len(), 2, "{folders:?}");
        assert!(folders.iter().all(|f| f.is_dir()));
    }

    #[test]
    fn the_current_directory_narrows_the_configured_roots() {
        let configured = [PathBuf::from("/r/stdlib/A"), PathBuf::from("/r/stdlib/B")];
        let project = Path::new("/r");
        assert_eq!(
            roots(&[], &configured, project, Path::new("/r")),
            configured
        );
        assert_eq!(
            roots(&[], &configured, project, Path::new("/r/stdlib/B/tests")),
            [PathBuf::from("/r/stdlib/B")]
        );
        // Of nested roots, the innermost one the directory is in.
        let nested = [PathBuf::from("/r/A"), PathBuf::from("/r/A/B")];
        assert_eq!(
            roots(&[], &nested, project, Path::new("/r/A/B")),
            [PathBuf::from("/r/A/B")]
        );
        assert_eq!(
            roots(&[], &nested, project, Path::new("/r/A/tests")),
            [PathBuf::from("/r/A")]
        );
        assert_eq!(
            roots(&[], &[], project, Path::new("/r/x")),
            [PathBuf::from("/r")]
        );
        assert_eq!(
            roots(
                &[PathBuf::from("lib")],
                &configured,
                project,
                Path::new("/w")
            ),
            [PathBuf::from("/w/lib")]
        );
    }

    #[test]
    fn patterns_expand_to_the_test_roots_they_match() {
        let dir = std::env::temp_dir().join(format!("jacket-expand-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for folder in [
            "lib/A/tests/inner",
            "lib/A/B",
            "lib/C/sub/tests",
            "lib/D",
            "lib/E/tests",
            "lib/.hidden/tests",
            "lib/skip/tests",
            "other/F/tests",
        ] {
            std::fs::create_dir_all(dir.join(folder)).unwrap();
        }
        std::fs::write(dir.join("lib/A/B/module.jai"), "").unwrap();
        std::fs::write(dir.join("lib/D/notes.txt"), "").unwrap();
        // `lib/A/tests/inner` is under a tests folder: never a root.
        std::fs::create_dir_all(dir.join("lib/A/tests/inner/tests")).unwrap();
        let ignore = [PathPattern::new("lib/skip/**").unwrap()];
        let found = |patterns: &[&str]| -> Vec<String> {
            let patterns: Vec<String> = patterns.iter().map(ToString::to_string).collect();
            expand(&patterns, &dir, &ignore)
                .unwrap()
                .iter()
                .map(|p| relative_text(p, &dir))
                .collect()
        };
        assert_eq!(
            found(&["lib/**"]),
            ["lib/A", "lib/A/B", "lib/C/sub", "lib/E"]
        );
        assert_eq!(found(&["lib/*"]), ["lib/A", "lib/E"]);
        assert_eq!(found(&["lib/{A,E}"]), ["lib/A", "lib/E"]);
        assert_eq!(found(&["lib/A"]), ["lib/A"]);
        // A plain path is taken as written, root or not.
        assert_eq!(found(&["lib/D"]), ["lib/D"]);
        assert_eq!(found(&["lib/**", "lib/A"]), found(&["lib/**"]));
        assert!(expand(&["a[".into()], &dir, &[]).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn files_are_sorted_into_kinds() {
        let dir = std::env::temp_dir().join(format!("jacket-discover-{}", std::process::id()));
        let tests = dir.join("tests");
        std::fs::create_dir_all(tests.join("compile_fail")).unwrap();
        std::fs::create_dir_all(tests.join("helpers")).unwrap();
        std::fs::create_dir_all(tests.join("faults")).unwrap();
        std::fs::create_dir_all(tests.join("negative")).unwrap();
        for file in [
            "unit.jai",
            "hello.jai",
            "hello.stdout",
            "compile_fail/bad.jai",
            "compile_fail/bad.stderr",
            "helpers/h.jai",
            "faults/f.jai",
            "faults/f.exit",
            "negative/n.jai",
            "negative/n.stderr",
        ] {
            std::fs::write(tests.join(file), "").unwrap();
        }
        std::fs::write(tests.join("negative/n.jai"), "// test: compile_fail\n").unwrap();
        std::fs::write(dir.join("module.jai"), "").unwrap();
        let root = Root::read(
            dir.clone(),
            Path::new("/nowhere"),
            Path::new("jaic"),
            &mut BTreeSet::new(),
            true,
        );
        assert_eq!(root.module, root.label);
        assert!(root.library);
        assert_eq!(root.unit, [tests.join("unit.jai")]);
        assert_eq!(
            root.golden,
            [tests.join("faults/f.jai"), tests.join("hello.jai")]
        );
        assert_eq!(
            root.compile_fail,
            [
                tests.join("compile_fail/bad.jai"),
                tests.join("negative/n.jai")
            ]
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
