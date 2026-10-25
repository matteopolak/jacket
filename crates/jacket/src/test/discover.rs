//! Where tests are. A test root is a folder with a `tests/` directory:
//!
//! - `tests/*.jai` with a `.stdout`, `.stderr` or `.exit` file beside it is a golden-output test,
//!   a program built and run alone;
//! - `tests/compile_fail/*.jai` must fail to compile, with the diagnostics in its `.stderr`;
//! - every other `tests/*.jai` goes into one build, whose `@test` procedures are the unit tests;
//! - folders below `tests/` hold what those files `#load` or `#import`.
//!
//! The roots are `[test] paths` in jacket.toml (only those the current directory is in, when it
//! is in some), else the project's folder.
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
    /// The name a program imports the library by: its path in the standard library
    /// (`Extensions/Testing`), else its folder's name, found through `-import_dir`.
    pub module: String,
    /// A library's doc tests.
    pub docs: Vec<super::doctest::Block>,
}

pub const EXPECTED: [&str; 3] = ["stdout", "stderr", "exit"];

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

/// Whether `source` has an expected-output file beside it.
pub fn is_golden(source: &Path) -> bool {
    EXPECTED.iter().any(|e| source.with_extension(e).is_file())
}

impl Root {
    /// `seen` holds the files earlier roots read doc tests from; a file is read for one root.
    pub fn read(dir: PathBuf, stdlib: &Path, seen: &mut BTreeSet<PathBuf>) -> Root {
        let tests = dir.join("tests");
        let (golden, unit) = jai_files(&tests).into_iter().partition(|f| is_golden(f));
        let label = dir
            .file_name()
            .map_or_else(|| "tests".into(), |n| n.to_string_lossy().into_owned());
        let library = dir.join("module.jai").is_file();
        let module = module_name(&dir, stdlib).unwrap_or_else(|| label.clone());
        Root {
            docs: if library {
                super::doctest::blocks(&dir, &module, stdlib, seen)
            } else {
                Vec::new()
            },
            module,
            library,
            compile_fail: jai_files(&tests.join("compile_fail")),
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
    }
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
        configured.to_vec()
    } else {
        inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn files_are_sorted_into_kinds() {
        let dir = std::env::temp_dir().join(format!("jacket-discover-{}", std::process::id()));
        let tests = dir.join("tests");
        std::fs::create_dir_all(tests.join("compile_fail")).unwrap();
        std::fs::create_dir_all(tests.join("helpers")).unwrap();
        for file in [
            "unit.jai",
            "hello.jai",
            "hello.stdout",
            "compile_fail/bad.jai",
            "compile_fail/bad.stderr",
            "helpers/h.jai",
        ] {
            std::fs::write(tests.join(file), "").unwrap();
        }
        std::fs::write(dir.join("module.jai"), "").unwrap();
        let root = Root::read(dir.clone(), Path::new("/nowhere"), &mut BTreeSet::new());
        assert_eq!(root.module, root.label);
        assert!(root.library);
        assert_eq!(root.unit, [tests.join("unit.jai")]);
        assert_eq!(root.golden, [tests.join("hello.jai")]);
        assert_eq!(root.compile_fail, [tests.join("compile_fail/bad.jai")]);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
