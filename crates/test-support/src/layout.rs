//! Checks that every integration test file is compiled.
//!
//! Each crate builds its `tests/` folder as one binary, `tests/main.rs`, so the compiler is
//! linked once instead of once per file. Cargo then no longer finds test files by itself
//! (`autotests = false`), so a file missing from `main.rs` would silently never run.
//! [`assert_every_test_file_is_built`] fails instead, naming the line to add.

use std::collections::BTreeSet;
use std::path::Path;

/// Panics unless every `tests/*.rs` file and `tests/*/main.rs` folder is built: as a module of
/// `tests/main.rs`, as its own `[[test]]` target in `Cargo.toml`, or as a support module that a
/// test file declares with `mod name;`.
///
/// ```no_run
/// #[test]
/// fn every_test_file_is_built() {
///     test_support::layout::assert_every_test_file_is_built(env!("CARGO_MANIFEST_DIR"));
/// }
/// ```
pub fn assert_every_test_file_is_built(manifest_dir: &str) {
    let missing = unbuilt_test_files(Path::new(manifest_dir));
    assert!(
        missing.is_empty(),
        "these test files are never compiled; add each to tests/main.rs:\n{}",
        missing
            .iter()
            .map(|name| format!("    #[path = \"{name}\"]\n    mod {};", module_name(name)))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The test files under `crate_dir/tests` that nothing builds, as paths relative to `tests/`
/// (`foo.rs` or `foo/main.rs`).
pub fn unbuilt_test_files(crate_dir: &Path) -> Vec<String> {
    let tests = crate_dir.join("tests");
    let read = |path: &Path| std::fs::read_to_string(path).unwrap_or_default();
    let mut built: BTreeSet<String> = path_attributes(&read(&tests.join("main.rs")));
    built.insert("main.rs".into());
    for line in read(&crate_dir.join("Cargo.toml")).lines() {
        if let Some(rest) = line.trim().strip_prefix("path = \"tests/") {
            built.insert(rest.trim_end_matches('"').to_string());
        }
    }
    let mut candidates = Vec::new();
    let mut support = BTreeSet::new();
    let mut entries: Vec<_> = std::fs::read_dir(&tests)
        .map(|dir| dir.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default();
    entries.sort();
    for path in entries {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() {
            if path.join("main.rs").is_file() {
                candidates.push((format!("{name}/main.rs"), name));
            }
        } else if let Some(stem) = name.strip_suffix(".rs") {
            // A top-level file's `mod x;` resolves to `tests/x.rs`, so `x` is support.
            support.extend(declared_modules(&read(&path)));
            support.extend(path_attributes(&read(&path)));
            candidates.push((name.clone(), stem.to_string()));
        }
    }
    candidates
        .into_iter()
        .filter(|(file, stem)| {
            !built.contains(file) && !support.contains(stem) && !support.contains(file)
        })
        .map(|(file, _)| file)
        .collect()
}

/// The module `#[path = "foo/main.rs"]` loads: `foo`.
fn module_name(file: &str) -> &str {
    file.split(['/', '.']).next().unwrap_or(file)
}

/// The files named by `#[path = "..."]` attributes in `text`.
fn path_attributes(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("#[path = \""))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_string)
        .collect()
}

/// The names of `mod name;` declarations in `text` (with or without `pub`) that resolve next to
/// the file. A `mod` under a `#[path]` attribute loads that path instead, so it does not count.
fn declared_modules(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut after_path = false;
    for line in text.lines().map(str::trim) {
        let unqualified = line
            .strip_prefix("pub(crate) ")
            .or_else(|| line.strip_prefix("pub "))
            .unwrap_or(line);
        if let Some(name) = unqualified
            .strip_prefix("mod ")
            .and_then(|r| r.strip_suffix(';'))
            && !after_path
        {
            names.insert(name.to_string());
        }
        after_path = line.starts_with("#[path = ");
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_test_file_missing_from_main_is_reported() {
        let dir = crate::TempDir::new("layout");
        let tests = dir.path().join("tests");
        std::fs::create_dir_all(tests.join("folder")).unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[[test]]\nname = \"own\"\npath = \"tests/own.rs\"\n",
        )
        .unwrap();
        std::fs::write(
            tests.join("main.rs"),
            "#[path = \"listed.rs\"]\nmod listed;\n",
        )
        .unwrap();
        // `listed.rs` loads `helper.rs` and its own `inner`; the top-level `inner.rs` is unbuilt.
        std::fs::write(
            tests.join("listed.rs"),
            "mod helper;\n#[path = \"folder/inner.rs\"]\nmod inner;\n",
        )
        .unwrap();
        std::fs::write(tests.join("inner.rs"), "").unwrap();
        std::fs::write(tests.join("helper.rs"), "").unwrap();
        std::fs::write(tests.join("own.rs"), "").unwrap();
        std::fs::write(tests.join("forgotten.rs"), "").unwrap();
        std::fs::write(tests.join("folder/main.rs"), "").unwrap();
        assert_eq!(
            unbuilt_test_files(dir.path()),
            ["folder/main.rs", "forgotten.rs", "inner.rs"]
        );
    }
}
