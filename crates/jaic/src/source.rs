//! Source files, spans and diagnostics.
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FileId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// No location: a diagnostic or note with this span prints without a file and snippet.
    pub const NONE: Span = Span {
        file: FileId(u32::MAX),
        start: 0,
        end: 0,
    };

    pub fn new(file: FileId, start: usize, end: usize) -> Self {
        Self {
            file,
            start: start as u32,
            end: end as u32,
        }
    }

    pub fn to(self, other: Span) -> Span {
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

pub struct SourceFile {
    pub path: String,
    pub text: Rc<str>,
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(path: String, text: Rc<str>) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        Self {
            path,
            text,
            line_starts,
        }
    }

    /// One-based line and column of a byte offset.
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        (line as u32 + 1, offset - self.line_starts[line] + 1)
    }

    /// Byte offset of a one-based line and column (clamped to the file).
    pub fn offset_of(&self, line: u32, col: u32) -> u32 {
        let index = (line.max(1) as usize - 1).min(self.line_starts.len() - 1);
        (self.line_starts[index] + col.max(1) - 1).min(self.text.len() as u32)
    }

    pub fn line_text(&self, line: u32) -> &str {
        let start = self.line_starts[(line - 1) as usize] as usize;
        let end = self
            .line_starts
            .get(line as usize)
            .map_or(self.text.len(), |&e| e as usize);
        self.text[start..end].trim_end_matches(['\n', '\r'])
    }
}

#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, path: String, text: Rc<str>) -> FileId {
        self.files.push(SourceFile::new(path, text));
        FileId(self.files.len() as u32 - 1)
    }

    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// The text of `span`, or "" for a span without a place.
    pub fn snippet_or_empty(&self, span: Span) -> &str {
        if (span.file.0 as usize) < self.files.len() && span != Span::default() {
            let text = &self.get(span.file).text;
            text.get(span.start as usize..span.end as usize)
                .unwrap_or("")
        } else {
            ""
        }
    }

    pub fn snippet(&self, span: Span) -> &str {
        let file = self.get(span.file);
        &file.text[span.start as usize..span.end as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
    /// Text under the primary span's carets (outside the plain layout).
    pub label: Option<String>,
    /// Related locations; a note whose span has no file (`Span::NONE`) prints as text only.
    /// Outside the plain layout, a located note is a label in the snippet.
    pub notes: Vec<(Span, String)>,
    /// `help:` lines printed last: how to fix the problem, when that is clear.
    pub help: Vec<String>,
    /// The fix the first help line describes: replace this span with this text. Shown as the
    /// changed line under the help, outside the plain layout.
    pub fix: Option<Box<(Span, String)>>,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, span, message.into())
    }

    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, span, message.into())
    }

    fn new(severity: Severity, span: Span, message: String) -> Self {
        Self {
            severity,
            span,
            message,
            label: None,
            notes: Vec::new(),
            help: Vec::new(),
            fix: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_note(mut self, span: Span, message: impl Into<String>) -> Self {
        self.notes.push((span, message.into()));
        self
    }

    pub fn with_help(mut self, message: impl Into<String>) -> Self {
        self.help.push(message.into());
        self
    }

    /// A help line with the replacement it suggests.
    pub fn with_fix(
        mut self,
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.help.insert(0, message.into());
        self.fix = Some(Box::new((span, replacement.into())));
        self
    }

    /// The diagnostic as text, in the process-wide style (`crate::render::style`).
    pub fn render(&self, sources: &SourceMap) -> String {
        self.report(sources).render()
    }

    /// The diagnostic in the shared renderer's terms.
    pub fn report<'a>(&'a self, sources: &'a SourceMap) -> crate::render::Report<'a> {
        use crate::render::{Fix, Help, Label, Report, Severity as S};
        let label = |span: Span, message: Option<String>| {
            has_location(sources, span).then(|| {
                let file = sources.get(span.file);
                Label {
                    path: &file.path,
                    text: &file.text,
                    start: span.start as usize,
                    end: span.end as usize,
                    message,
                }
            })
        };
        let mut report = Report::new(
            match self.severity {
                Severity::Error => S::Error,
                Severity::Warning => S::Warning,
                Severity::Note => S::Note,
            },
            self.message.clone(),
        );
        report.primary = label(self.span, self.label.clone());
        for (span, note) in &self.notes {
            match label(*span, Some(note.clone())) {
                Some(l) => report.secondary.push(l),
                None => report.notes.push(note.clone()),
            }
        }
        for (i, help) in self.help.iter().enumerate() {
            let fix = self
                .fix
                .as_deref()
                .filter(|_| i == 0)
                .and_then(|(span, text)| {
                    has_location(sources, *span).then(|| Fix {
                        text: &sources.get(span.file).text,
                        edits: vec![(span.start as usize, span.end as usize, text.clone())],
                    })
                });
            report.help.push(Help {
                message: help.clone(),
                fix,
            });
        }
        report
    }
}

/// Whether `span` names a place in a file. `Span::NONE` does not, and neither does the default
/// span (file 0, empty, at offset 0), which diagnostics without a location have long used.
fn has_location(sources: &SourceMap, span: Span) -> bool {
    (span.file.0 as usize) < sources.len() && span != Span::default()
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
