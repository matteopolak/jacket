//! Failed `#assert`s and the imports that led to them, as the linter records them.
use jaic::fxhash::HashSet;
use jaic::sema::{Compiler, ImportEdge};
use jaic::target::TargetOs;
use std::path::{Path, PathBuf};

/// An `#assert` that failed while compiling `root` for `os`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssertFailure {
    pub root: PathBuf,
    pub os: TargetOs,
    pub path: PathBuf,
    /// Byte range of the assert's condition in `path`.
    pub start: usize,
    pub end: usize,
    /// The compiler's text (`#assert failed: ...`).
    pub message: String,
    /// The condition names `OS` or is `false`: the failure is about the operating system.
    pub about_os: bool,
    /// The `#import` or `#load` statements that led from the root's entry to `path`, outermost
    /// first: each is in the file named, at the byte range given.
    pub chain: Vec<ImportSite>,
}

/// One `#import` or `#load` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSite {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
}

/// The sites that loaded `file`, outermost first: each file's first request, followed up to a
/// file nothing asked for.
fn import_chain(edges: &[ImportEdge], file: &Path) -> Vec<ImportSite> {
    let mut chain = Vec::new();
    let mut seen: HashSet<&Path> = HashSet::default();
    seen.insert(file);
    let mut at = file;
    while let Some(edge) = edges
        .iter()
        .find(|e| e.to == at && !seen.contains(e.from.as_path()))
    {
        chain.push(ImportSite {
            path: edge.from.clone(),
            start: edge.start,
            end: edge.end,
        });
        seen.insert(&edge.from);
        at = &edge.from;
    }
    chain.reverse();
    chain
}

/// The failed asserts of `compiler`, once each, with their import chains.
pub(super) fn failures_of(compiler: &Compiler, root: &Path) -> Vec<AssertFailure> {
    let edges = compiler.import_edges();
    let mut seen = HashSet::default();
    compiler
        .failed_asserts()
        .into_iter()
        .filter(|f| seen.insert((f.path.clone(), f.start)))
        .map(|f| AssertFailure {
            root: root.to_path_buf(),
            os: compiler.options.os,
            chain: import_chain(&edges, &f.path),
            path: f.path,
            start: f.start,
            end: f.end,
            message: f.message,
            about_os: f.about_os,
        })
        .collect()
}
