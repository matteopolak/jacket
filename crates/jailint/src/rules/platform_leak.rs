//! `platform_leak`: code meant to build on every operating system imports code that does not.
//!
//! ```jai
//! #import "POSIX";   // asserts `OS == .LINUX || OS == .MACOS ...`: fails to compile on Windows
//! ```
//!
//! A library that is supposed to work everywhere sometimes pulls in an OS-only module by
//! accident, and nobody finds out until a build for that OS. The rule compiles the portable
//! code once per target and reports the import that brings in a file the target is not
//! supposed to build, when an `#assert` in what it brings in fails there.
//!
//! Which code is portable, which systems it is built for and which files are limited to a few
//! systems are settings (`[lint.portability]`, see `Portability` in `config.rs`) or a
//! `// jailint: platforms(linux, macos)` comment at the top of a file. Off by default.
//!
//! Unlike the other rules this one is not run on one compiled program: the driver compiles each
//! portable root for each target with failing asserts recorded
//! (`driver::leak_pass`), and [`analyze`] decides where to report from the imports that were met
//! and the asserts that failed. An import is reported when
//!
//! - it is written in portable code that builds on the target,
//! - it brings in a file the settings say does not build on the target, and
//! - an `#assert` fails in that file or in something it brings in.
//!
//! An `#assert` that fails in a portable file with no such import above it is reported on the
//! assert itself: the file needs to be limited to the systems it is written for.
use crate::config::{Config, file_platforms};
use crate::{Finding, syntax::Cx};
use jaic::fxhash::{HashMap, HashSet};
use jaic::sema::{FailedAssert, ImportEdge};
use jaic::target::TargetOs;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "platform_leak",
    category: crate::config::Category::Restriction,
    summary: "portable code imports something that asserts an operating system it is not built for",
    check,
};

/// Registered so the rule has a name, a level and a summary; the driver does the work.
pub(crate) fn check(_cx: &Cx, _out: &mut Vec<Finding>) {
}

/// What a file's settings and `platforms(...)` comment say about where it builds.
pub(crate) struct Probe<'a> {
    pub config: &'a Config,
    comments: RefCell<HashMap<PathBuf, Option<Vec<TargetOs>>>>,
}

impl<'a> Probe<'a> {
    pub fn new(config: &'a Config) -> Self {
        Probe {
            config,
            comments: RefCell::default(),
        }
    }

    /// Whether the file at `path` builds on `os`.
    pub fn allowed(&self, path: &Path, os: TargetOs) -> bool {
        if !self.config.builds_on(path, os) {
            return false;
        }
        let mut comments = self.comments.borrow_mut();
        let listed = comments.entry(path.to_path_buf()).or_insert_with(|| {
            let text = std::fs::read_to_string(path).ok()?;
            file_platforms(&text)
        });
        listed.as_ref().is_none_or(|systems| systems.contains(&os))
    }

    /// Whether the file at `path` is held to every target and builds on `os`. A file the
    /// compiler made up (a module's stand-in root) is not a file of the project.
    pub fn portable(&self, path: &Path, os: TargetOs) -> bool {
        self.config.is_portable(path) && path.is_file() && self.allowed(path, os)
    }

    /// The targets the file at `path` builds on.
    pub fn systems(&self, path: &Path) -> Vec<TargetOs> {
        self.config
            .portability
            .targets()
            .iter()
            .copied()
            .filter(|os| self.allowed(path, *os))
            .collect()
    }
}

/// Where portable code goes wrong on one target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Leak {
    /// The `#import` or `#load` at `start..end` of `from` brings in `to`, which does not build
    /// on the target, and `assert` fails below it.
    Import {
        from: PathBuf,
        start: usize,
        end: usize,
        to: PathBuf,
        assert: FailedAssert,
    },
    /// `assert` fails in a file that is held to every target.
    Own { assert: FailedAssert },
}

/// The leaks of one compile for `os`: `edges` are the imports it met, `failed` the asserts that
/// failed. At most one per import site and one per failing assert.
pub(crate) fn analyze(
    probe: &Probe,
    os: TargetOs,
    edges: &[ImportEdge],
    failed: &[FailedAssert],
) -> Vec<Leak> {
    let mut into: HashMap<&Path, Vec<&ImportEdge>> = HashMap::default();
    let mut out_of: HashMap<&Path, Vec<&ImportEdge>> = HashMap::default();
    for edge in edges {
        into.entry(&edge.to).or_default().push(edge);
        out_of.entry(&edge.from).or_default().push(edge);
    }
    let bad: HashSet<&Path> = failed.iter().map(|f| f.path.as_path()).collect();
    // The files from which a failing assert can be reached.
    let mut reaches: HashSet<&Path> = bad.clone();
    let mut queue: Vec<&Path> = bad.iter().copied().collect();
    while let Some(file) = queue.pop() {
        for edge in into.get(file).into_iter().flatten() {
            if reaches.insert(&edge.from) {
                queue.push(&edge.from);
            }
        }
    }
    let mut leaks = Vec::new();
    let mut seen_sites = HashSet::default();
    let mut explained: HashSet<&Path> = HashSet::default();
    for edge in edges {
        if !reaches.contains(edge.to.as_path())
            || !probe.portable(&edge.from, os)
            || probe.allowed(&edge.to, os)
            || !seen_sites.insert((&edge.from, edge.start, edge.end))
        {
            continue;
        }
        // The nearest failing file below the import, and everything below it.
        let mut below: HashSet<&Path> = HashSet::default();
        let mut order: Vec<&Path> = vec![&edge.to];
        below.insert(&edge.to);
        let mut next = 0;
        while next < order.len() {
            let file = order[next];
            next += 1;
            for e in out_of.get(file).into_iter().flatten() {
                if below.insert(&e.to) {
                    order.push(&e.to);
                }
            }
        }
        explained.extend(below.iter().copied());
        let Some(witness) = order
            .iter()
            .find_map(|file| failed.iter().find(|f| f.path == *file))
        else {
            continue;
        };
        leaks.push(Leak::Import {
            from: edge.from.clone(),
            start: edge.start,
            end: edge.end,
            to: edge.to.clone(),
            assert: witness.clone(),
        });
    }
    let mut own_sites = HashSet::default();
    for assert in failed {
        if !explained.contains(assert.path.as_path())
            && probe.portable(&assert.path, os)
            && own_sites.insert((&assert.path, assert.start))
        {
            leaks.push(Leak::Own {
                assert: assert.clone(),
            });
        }
    }
    leaks
}

/// What a file is called in a message: a module by its folder, any other file by its name.
pub(crate) fn display_name(path: &Path) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
    match name.as_deref() {
        Some("module.jai") => path
            .parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        Some(name) => name.to_string(),
        None => path.to_string_lossy().into_owned(),
    }
}

/// `3:9` for the byte `at` of `text`.
pub(crate) fn line_col(text: &str, at: usize) -> (usize, usize) {
    let at = at.min(text.len());
    let before = &text[..at];
    let line = before.matches('\n').count() + 1;
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    (line, text[start..at].chars().count() + 1)
}

/// The words of a list of systems: `linux and macos`.
pub(crate) fn system_list(systems: &[TargetOs]) -> String {
    let names: Vec<&str> = systems
        .iter()
        .map(|os| jaic::target::os_name(*os))
        .collect();
    match names.as_slice() {
        [] => "no target".to_string(),
        [one] => (*one).to_string(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_positions() {
        assert_eq!(display_name(Path::new("/s/POSIX/module.jai")), "POSIX");
        assert_eq!(
            display_name(Path::new("/s/net/fields-windows.jai")),
            "fields-windows.jai"
        );
        assert_eq!(line_col("ab\ncd\n", 4), (2, 2));
        assert_eq!(line_col("abc", 0), (1, 1));
        assert_eq!(
            system_list(&[TargetOs::Linux, TargetOs::MacOS, TargetOs::Windows]),
            "linux, macos and windows"
        );
        assert_eq!(
            system_list(&[TargetOs::Linux, TargetOs::MacOS]),
            "linux and macos"
        );
        assert_eq!(system_list(&[]), "no target");
    }
}
