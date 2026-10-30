//! Snapshot testing: [`insta`] snapshots whose text does not depend on where the repository
//! is checked out or which OS produced it.
//!
//! ```ignore
//! test_support::assert_snapshot!("facts", dump);                  // tests/snapshots/<test>__facts.snap
//! test_support::assert_snapshot_in!(case_dir, "bad", rendered);   // <case_dir>/bad.snap
//! ```
//!
//! Both macros run the text through [`normalize_repo_paths`] first. To change what a test
//! prints, rerun it with `INSTA_UPDATE=always` (or review with `cargo insta review`) and read
//! the diff; CI never writes snapshots.

pub use insta;

use std::path::{Path, PathBuf};

/// Asserts that `$text` matches the snapshot `$name` stored beside the calling test file, in
/// `snapshots/<test file>__<name>.snap`. Repository paths in the text are normalised first.
#[macro_export]
macro_rules! assert_snapshot {
    ($name:expr, $text:expr) => {{
        let text = $crate::snapshot::normalize_repo_paths(&$text);
        $crate::snapshot::insta::with_settings!({ omit_expression => true }, {
            $crate::snapshot::insta::assert_snapshot!($name, text);
        });
    }};
}

/// Asserts that `$text` matches the snapshot `$dir/$name.snap`, with no module prefix in the
/// file name: for fixtures that keep their expected output next to their input.
#[macro_export]
macro_rules! assert_snapshot_in {
    ($dir:expr, $name:expr, $text:expr) => {{
        let text = $crate::snapshot::normalize_repo_paths(&$text);
        $crate::snapshot::insta::with_settings!({
            omit_expression => true,
            prepend_module_to_snapshot => false,
            snapshot_path => $dir,
        }, {
            $crate::snapshot::insta::assert_snapshot!($name, text);
        });
    }};
}

/// The repository root, canonical.
fn repo_root() -> PathBuf {
    crate::repo::root()
}

/// `text` with the repository root, and the path from the current directory up to it,
/// replaced by `[repo]`. See [`normalize_paths`] for the forms that are recognised.
#[must_use]
pub fn normalize_repo_paths(text: &str) -> String {
    let root = repo_root();
    let mut redactor = Redactor::new().root("[repo]", &root);
    if let Ok(current) = std::env::current_dir()
        && let Some(up) = relative_up(&current, &root)
    {
        redactor = redactor.relative("[repo]", &up);
    }
    redactor.apply(text)
}

/// `text` with each root of `roots` (a label and the directory it names) replaced by its label
/// and the rest of any path under it written with `/` separators.
#[must_use]
pub fn normalize_paths(text: &str, roots: &[(&str, &Path)]) -> String {
    let mut redactor = Redactor::new();
    for (label, path) in roots {
        redactor = redactor.root(label, path);
    }
    redactor.apply(text)
}

/// The relative path (`../..`) that leads from `from` to its ancestor `to`.
fn relative_up(from: &Path, to: &Path) -> Option<String> {
    let from = from.canonicalize().ok()?;
    let depth = from.strip_prefix(to).ok()?.components().count();
    (depth > 0).then(|| vec![".."; depth].join("/"))
}

/// Replaces directory prefixes in text by labels. Each root is matched in every spelling a
/// path takes in compiler output:
///
/// - canonical, with `/` separators (`/work/repo`, `C:/work/repo`);
/// - with `\` separators (`C:\work\repo`);
/// - with a Windows verbatim prefix (`\\?\C:\work\repo`, `//?/C:/work/repo`);
/// - relative to the current directory (`../..`), added with [`Redactor::relative`].
///
/// A matched prefix becomes its label; the rest of that path (up to whitespace, a quote or
/// a `:`) has its `\` separators turned into `/`.
#[derive(Debug, Default)]
pub struct Redactor {
    /// (spelling, label, whether the spelling is relative), longest spelling first.
    patterns: Vec<(String, String, bool)>,
}

impl Redactor {
    /// A redactor with no roots.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the absolute directory `path` under `label`.
    #[must_use]
    pub fn root(mut self, label: &str, path: &Path) -> Self {
        let text = path.to_string_lossy();
        let slash = text
            .strip_prefix(r"\\?\")
            .or_else(|| text.strip_prefix("//?/"))
            .unwrap_or(&text)
            .replace('\\', "/");
        let slash = slash.trim_end_matches('/').to_string();
        let back = slash.replace('/', "\\");
        for spelling in [
            format!(r"\\?\{back}"),
            format!("//?/{slash}"),
            format!(r"\\?\{slash}"),
            back,
            slash,
        ] {
            self.add(spelling, label, false);
        }
        self
    }

    /// Adds a relative spelling of a root (`../..`), as it appears in paths printed relative to
    /// the current directory. Matches with either separator.
    #[must_use]
    pub fn relative(mut self, label: &str, up: &str) -> Self {
        let slash = up.replace('\\', "/");
        let slash = slash.trim_end_matches('/').to_string();
        self.add(slash.replace('/', "\\"), label, true);
        self.add(slash, label, true);
        self
    }

    fn add(&mut self, spelling: String, label: &str, relative: bool) {
        if spelling.is_empty() || self.patterns.iter().any(|(s, _, _)| *s == spelling) {
            return;
        }
        self.patterns.push((spelling, label.to_string(), relative));
        self.patterns
            .sort_by_key(|(s, _, _)| std::cmp::Reverse(s.len()));
    }

    /// `text` with every recognised root replaced.
    #[must_use]
    pub fn apply(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut at = 0;
        while at < text.len() {
            let rest = &text[at..];
            let previous = text[..at].chars().next_back();
            if let Some((spelling, label)) = self.match_at(rest, previous) {
                out.push_str(label);
                let tail_end = rest[spelling.len()..]
                    .find(is_tail_end)
                    .map_or(rest.len(), |n| spelling.len() + n);
                out.push_str(&rest[spelling.len()..tail_end].replace('\\', "/"));
                at = text.len() - rest.len() + tail_end;
            } else {
                let ch = rest.chars().next().expect("rest is not empty");
                out.push(ch);
                at += ch.len_utf8();
            }
        }
        out
    }

    fn match_at<'a>(&'a self, rest: &str, previous: Option<char>) -> Option<(&'a str, &'a str)> {
        self.patterns
            .iter()
            .find_map(|(spelling, label, relative)| {
                let after = rest.strip_prefix(spelling.as_str())?;
                // `../..` followed by another `/..` names a directory further up, not the root.
                let goes_further_up =
                    *relative && (after.starts_with("/..") || after.starts_with("\\.."));
                let ends_here =
                    !goes_further_up && (after.is_empty() || after.starts_with(['/', '\\']));
                let starts_here = previous.is_none_or(|c| {
                    let in_name = c.is_alphanumeric() || matches!(c, '.' | '_' | '-');
                    !(in_name || (*relative && matches!(c, '/' | '\\' | ':')))
                });
                (ends_here && starts_here).then_some((spelling.as_str(), label.as_str()))
            })
    }
}

/// Whether `c` ends the tail of a path.
fn is_tail_end(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '"' | '\'' | '`' | ':' | ')' | ',' | ';' | '<' | '>' | '|'
        )
}

/// `path` written with `/` separators, as a snapshot should show a relative path on every OS.
#[must_use]
pub fn slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Whether a generated file may be rewritten: `INSTA_UPDATE=always`, the same switch that
/// rewrites snapshots.
#[must_use]
pub fn updating() -> bool {
    jacket_config::env::INSTA_UPDATE
        .get()
        .is_some_and(|v| v == "always")
}

/// Checks a generated file that another tool reads (so it cannot be a `.snap`) against what the
/// code produces now. With `INSTA_UPDATE=always` the file is rewritten instead. `follow_up` is
/// appended to the failure message (what to run after the rewrite).
pub fn assert_generated_file(path: &Path, generated: &str, follow_up: &str) {
    if updating() {
        std::fs::write(path, generated)
            .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        return;
    }
    let current = std::fs::read_to_string(path).unwrap_or_default();
    assert!(
        current == generated,
        "{} is stale: rerun the test with INSTA_UPDATE=always{follow_up}",
        path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redact(text: &str, root: &str) -> String {
        Redactor::new().root("[repo]", Path::new(root)).apply(text)
    }

    #[test]
    fn canonical_paths() {
        assert_eq!(
            redact("error --> /work/repo/stdlib/Basic.jai:3:1", "/work/repo"),
            "error --> [repo]/stdlib/Basic.jai:3:1"
        );
        assert_eq!(redact("at /work/repo", "/work/repo"), "at [repo]");
    }

    #[test]
    fn a_longer_directory_name_is_not_the_root() {
        assert_eq!(
            redact("/work/repository/a.jai", "/work/repo"),
            "/work/repository/a.jai"
        );
    }

    #[test]
    fn relative_paths() {
        let redactor = Redactor::new()
            .root("[repo]", Path::new("/work/repo"))
            .relative("[repo]", "../..");
        assert_eq!(
            redactor.apply("--> ../../stdlib/a.jai:1:1"),
            "--> [repo]/stdlib/a.jai:1:1"
        );
        assert_eq!(
            redactor.apply(r"--> ..\..\stdlib\a.jai:1:1"),
            "--> [repo]/stdlib/a.jai:1:1"
        );
        assert_eq!(redactor.apply("../../.."), "../../..");
        assert_eq!(redactor.apply("x../../a"), "x../../a");
    }

    #[test]
    fn windows_verbatim_prefixes() {
        assert_eq!(
            redact(r"\\?\C:\work\repo\stdlib\a.jai", r"C:\work\repo"),
            "[repo]/stdlib/a.jai"
        );
        assert_eq!(
            redact("//?/C:/work/repo/stdlib/a.jai", r"\\?\C:\work\repo"),
            "[repo]/stdlib/a.jai"
        );
    }

    #[test]
    fn backslash_separators() {
        assert_eq!(
            redact(r"--> C:\work\repo\tests\lint\bad.jai:5:11", r"C:\work\repo"),
            "--> [repo]/tests/lint/bad.jai:5:11"
        );
        assert_eq!(
            redact(
                r"C:\work\repo\a\b.jai and C:\work\repo\c.jai",
                "C:/work/repo"
            ),
            "[repo]/a/b.jai and [repo]/c.jai"
        );
    }

    #[test]
    fn separators_outside_a_matched_path_are_left_alone() {
        assert_eq!(
            redact(r#"print("a\n") /work/repo/a.jai"#, "/work/repo"),
            r#"print("a\n") [repo]/a.jai"#
        );
    }

    #[test]
    fn several_roots() {
        let text = normalize_paths(
            "/tmp/x/a.jai /work/repo/b.jai",
            &[
                ("[tmp]", Path::new("/tmp/x")),
                ("[repo]", Path::new("/work/repo")),
            ],
        );
        assert_eq!(text, "[tmp]/a.jai [repo]/b.jai");
    }

    #[test]
    fn relative_paths_are_slashed() {
        assert_eq!(
            slashed(Path::new("tests/lint/a/bad.jai")),
            "tests/lint/a/bad.jai"
        );
        assert_eq!(
            slashed(Path::new(r"tests\lint\a\bad.jai")),
            "tests/lint/a/bad.jai"
        );
    }

    #[test]
    fn up_paths() {
        let dir = crate::tmp::TempDir::new("up");
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        assert_eq!(
            relative_up(&dir.join("a/b"), &dir).as_deref(),
            Some("../..")
        );
        assert_eq!(relative_up(&dir, &dir), None);
    }
}
