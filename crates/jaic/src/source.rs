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
    pub notes: Vec<(Span, String)>,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            span,
            message: message.into(),
            notes: Vec::new(),
        }
    }
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            span,
            message: message.into(),
            notes: Vec::new(),
        }
    }
    pub fn with_note(mut self, span: Span, message: impl Into<String>) -> Self {
        self.notes.push((span, message.into()));
        self
    }
    pub fn render(&self, sources: &SourceMap) -> String {
        let mut out = String::new();
        let kind = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
        render_one(&mut out, sources, self.span, kind, &self.message);
        for (span, note) in &self.notes {
            render_one(&mut out, sources, *span, "note", note);
        }
        out
    }
}

fn render_one(out: &mut String, sources: &SourceMap, span: Span, kind: &str, message: &str) {
    use std::fmt::Write;
    if (span.file.0 as usize) < sources.len() {
        let file = sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let _ = writeln!(out, "{}:{}:{}: {}: {}", file.path, line, col, kind, message);
        let text = file.line_text(line);
        let _ = writeln!(out, "    {text}");
        let width = (span.end.saturating_sub(span.start))
            .clamp(1, (text.len() as u32 + 1).saturating_sub(col).max(1));
        let pad: String = text
            .chars()
            .take(col as usize - 1)
            .map(|c| {
                if c == '\t' {
                    '\t'
                } else {
                    ' '
                }
            })
            .collect();
        let _ = writeln!(out, "    {}{}", pad, "^".repeat(width as usize));
    } else {
        let _ = writeln!(out, "{kind}: {message}");
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
