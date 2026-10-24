//! Where each `#import` and `#load` of a compile came from, and which `#assert`s failed.
//!
//! A tool that checks code against several targets needs more than the first error: which file
//! asked for the file whose `#assert` failed, and (with [`Compiler::keep_going_past_asserts`])
//! every failing `#assert` of the compile rather than the first. Import sites are noted only in
//! that mode, so an ordinary compile pays nothing; failed asserts are always noted.
use super::Compiler;
use crate::source::{FileId, Span};
use std::cell::RefCell;
use std::path::PathBuf;

/// What a compile noted while loading files.
#[derive(Default)]
pub struct ImportTrace {
    /// `(site, target)`: the `#import` or `#load` at `site` asked for the file at `target`.
    /// A site is noted once per request, so a file loaded twice has two entries.
    edges: RefCell<Vec<(Span, PathBuf)>>,
    /// `#assert`s that failed, by the span the diagnostic points at, with its message.
    failed: RefCell<Vec<(Span, String, bool)>>,
    /// A failing `#assert` is noted and compilation goes on, and import sites are noted.
    keep_going: bool,
}

/// One `#import` or `#load` in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportEdge {
    /// The file holding the statement.
    pub from: PathBuf,
    /// Byte range of the statement in `from`.
    pub start: usize,
    pub end: usize,
    /// The file it brought in (a module's entry file for an `#import`).
    pub to: PathBuf,
}

/// An `#assert` that failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailedAssert {
    pub path: PathBuf,
    /// Byte range of the assert's condition in `path`.
    pub start: usize,
    pub end: usize,
    /// The diagnostic's text (`#assert failed: ...`).
    pub message: String,
    /// The condition is about the operating system (it names `OS`) or is `false`, rather than
    /// about a size, a CPU or a build setting.
    pub about_os: bool,
}

impl ImportTrace {
    pub(super) fn edge(&self, site: Span, target: PathBuf) {
        if self.keep_going && site != Span::NONE {
            self.edges.borrow_mut().push((site, target));
        }
    }

    pub(super) fn failure(&self, span: Span, message: &str, about_os: bool) {
        self.failed
            .borrow_mut()
            .push((span, message.to_string(), about_os));
    }

    pub(super) fn keeps_going(&self) -> bool {
        self.keep_going
    }
}

impl Compiler {
    /// Note a failing `#assert` and go on compiling instead of stopping at it, so one compile
    /// finds them all. What the program does afterwards is no longer meaningful.
    pub fn keep_going_past_asserts(&mut self) {
        self.src.import_trace.keep_going = true;
    }

    /// The path of the file `file` is part of, when it is a real one.
    fn trace_path(&self, file: FileId) -> Option<PathBuf> {
        if file == FileId(u32::MAX) {
            return None;
        }
        self.src.sources.get(file).path.as_file().map(PathBuf::from)
    }

    /// Every `#import` and `#load` met so far, in the order they were met.
    pub fn import_edges(&self) -> Vec<ImportEdge> {
        self.src
            .import_trace
            .edges
            .borrow()
            .iter()
            .filter_map(|(site, to)| {
                Some(ImportEdge {
                    from: self.trace_path(site.file)?,
                    start: site.start as usize,
                    end: site.end as usize,
                    to: to.clone(),
                })
            })
            .collect()
    }

    /// Every `#assert` that failed so far, in the order they failed.
    pub fn failed_asserts(&self) -> Vec<FailedAssert> {
        self.src
            .import_trace
            .failed
            .borrow()
            .iter()
            .filter_map(|(span, message, about_os)| {
                Some(FailedAssert {
                    path: self.trace_path(span.file)?,
                    start: span.start as usize,
                    end: span.end as usize,
                    message: message.clone(),
                    about_os: *about_os,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sema::{Options, ProgramSource, VirtualFs};
    use std::rc::Rc;

    fn compile(keep_going: bool) -> (Compiler, bool) {
        let mut fs = VirtualFs::default();
        fs.insert(
            "/p/main.jai",
            b"#load \"a.jai\";\nmain :: () {}\n".as_slice(),
        );
        fs.insert(
            "/p/a.jai",
            b"#assert false, \"first\";\n#assert false, \"second\";\n".as_slice(),
        );
        let mut options = Options::host();
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(fs));
        if keep_going {
            compiler.keep_going_past_asserts();
        }
        let result = compiler.compile_sources(&[ProgramSource::File("/p/main.jai".into())]);
        (compiler, result.is_ok())
    }

    #[test]
    fn an_ordinary_compile_stops_at_the_first_failed_assert_and_notes_no_imports() {
        let (compiler, ok) = compile(false);
        assert!(!ok);
        let failed = compiler.failed_asserts();
        assert_eq!(failed.len(), 1, "{failed:?}");
        assert_eq!(failed[0].path, PathBuf::from("/p/a.jai"));
        assert_eq!(failed[0].message, "#assert failed: first");
        assert_eq!(compiler.import_edges(), []);
    }

    #[test]
    fn a_failed_assert_is_noted_with_the_load_that_brought_in_its_file() {
        let (compiler, _) = compile(true);
        let edges = compiler.import_edges();
        assert_eq!(edges.len(), 1, "{edges:?}");
        assert_eq!(edges[0].from, PathBuf::from("/p/main.jai"));
        assert_eq!(edges[0].to, PathBuf::from("/p/a.jai"));
        assert_eq!((edges[0].start, edges[0].end), (6, 13));
    }

    #[test]
    fn keeping_going_notes_every_failed_assert() {
        let (compiler, ok) = compile(true);
        assert!(ok);
        let messages: Vec<_> = compiler
            .failed_asserts()
            .into_iter()
            .map(|f| f.message)
            .collect();
        assert_eq!(
            messages,
            ["#assert failed: first", "#assert failed: second"]
        );
    }
}
