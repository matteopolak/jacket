//! Source files, spans and diagnostics.
use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct FileId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// No location: a diagnostic or note with this span prints without a file and snippet.
    /// It is the only span without one; `Span` has no `Default`, so an empty span at the start
    /// of the first file always means that place.
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

    #[must_use]
    pub fn to(self, other: Span) -> Span {
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Where the lines of one text start and end: the single answer to "which line and column is
/// this offset" for the compiler, the renderer, the language server and the linter.
///
/// Lines end at `\n`; a `\r` before it is not part of the line (`line_end` excludes it), and
/// neither is a `\r` ending the text. Offsets are bytes. Columns are counted in characters
/// (`line_col`) or UTF-16 units (`utf16_position`, the language server protocol's unit).
/// Building is one pass; every query after that is a binary search, so callers that ask
/// about many offsets of one text build the index once and keep it (`SourceFile::lines`).
#[derive(Clone, Debug, Default)]
pub struct LineIndex {
    starts: Vec<u32>,
    ends: Vec<u32>,
}

/// Why `LineIndex::offset_utf16` found no offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utf16Error {
    /// The line is past the last one.
    LineOutside,
    /// The column falls between the two halves of a surrogate pair.
    SplitsPair,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut starts = vec![0u32];
        let mut ends = Vec::new();
        for (at, _) in text.match_indices('\n') {
            ends.push((at - usize::from(at > 0 && bytes[at - 1] == b'\r')) as u32);
            starts.push(at as u32 + 1);
        }
        let last = *starts.last().unwrap_or(&0) as usize;
        let len = text.len();
        ends.push((len - usize::from(len > last && bytes[len - 1] == b'\r')) as u32);
        Self {
            starts,
            ends,
        }
    }

    /// The number of lines: one more than the newlines, so the empty text has one, and a text
    /// ending in a newline has an empty last line.
    pub fn line_count(&self) -> usize {
        self.starts.len()
    }

    /// The zero-based line holding `offset`; offsets past the end belong to the last line.
    pub fn line_of(&self, offset: usize) -> usize {
        self.starts
            .partition_point(|&start| start as usize <= offset)
            .saturating_sub(1)
    }

    /// The offset where the line holding `offset` starts.
    pub fn line_start(&self, offset: usize) -> usize {
        self.starts[self.line_of(offset)] as usize
    }

    /// The offset where the line holding `offset` ends, before its `\n` and any `\r` in front
    /// of it.
    pub fn line_end(&self, offset: usize) -> usize {
        self.ends[self.line_of(offset)] as usize
    }

    /// The byte range of zero-based `line`, without its line ending.
    pub fn line_range(&self, line: usize) -> Option<(usize, usize)> {
        Some((*self.starts.get(line)? as usize, self.ends[line] as usize))
    }

    /// The byte ranges of every line, in order.
    pub fn lines(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.starts
            .iter()
            .zip(&self.ends)
            .map(|(&start, &end)| (start as usize, end as usize))
    }

    /// One-based line and column of `offset` in `text` (the text the index was built from),
    /// the column counted in characters. An offset past the end or inside a character is moved
    /// back to the character it falls in.
    pub fn line_col(&self, text: &str, offset: usize) -> (usize, usize) {
        let offset = text.floor_char_boundary(offset.min(text.len()));
        let line = self.line_of(offset);
        let start = self.starts[line] as usize;
        (line + 1, text[start..offset].chars().count() + 1)
    }

    /// One-based line and column of `offset`, the column counted in bytes.
    pub fn line_byte_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_of(offset);
        (line + 1, offset - self.starts[line] as usize + 1)
    }

    /// Zero-based line and UTF-16 column of `offset` in `text`, or `None` for an offset past
    /// the end or inside a character. An offset inside a line ending counts to the line's end.
    pub fn utf16_position(&self, text: &str, offset: usize) -> Option<(u32, u32)> {
        if offset > text.len() || !text.is_char_boundary(offset) {
            return None;
        }
        let line = self.line_of(offset);
        let end = offset.min(self.ends[line] as usize);
        let column = utf16_len(&text[self.starts[line] as usize..end]);
        Some((line as u32, column))
    }

    /// The offset of zero-based `line` and UTF-16 column `column` in `text`. A column past the
    /// line's end names the end.
    pub fn offset_utf16(&self, text: &str, line: u32, column: u32) -> Result<usize, Utf16Error> {
        let (start, end) = self
            .line_range(line as usize)
            .ok_or(Utf16Error::LineOutside)?;
        let mut units = 0u32;
        for (relative, ch) in text[start..end].char_indices() {
            if units == column {
                return Ok(start + relative);
            }
            units += ch.len_utf16() as u32;
            if units > column {
                return Err(Utf16Error::SplitsPair);
            }
        }
        Ok(end)
    }
}

/// The length of `text` in UTF-16 units, the language server protocol's column unit.
pub fn utf16_len(text: &str) -> u32 {
    text.chars().map(|c| c.len_utf16() as u32).sum()
}

/// The start of the line holding `offset` in `text`, for callers that ask once. For many
/// offsets of one text, build a `LineIndex`. An offset past the end counts as the end; one
/// inside a character is fine.
pub fn line_start(text: &str, offset: usize) -> usize {
    let bytes = &text.as_bytes()[..offset.min(text.len())];
    bytes.iter().rposition(|&b| b == b'\n').map_or(0, |n| n + 1)
}

/// The end of the line holding `offset` in `text`, before its line break (a `\r` in front of
/// the `\n` is not part of the line either); the counterpart of `line_start`.
pub fn line_end(text: &str, offset: usize) -> usize {
    let bytes = text.as_bytes();
    let from = offset.min(bytes.len());
    let end = bytes[from..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |n| from + n);
    if end > 0 && bytes[end - 1] == b'\r' {
        end - 1
    } else {
        end
    }
}

/// What a source is named: a file on disk, or a label for text with no file (`#import,string`,
/// `add_build_string`, `#insert` expansions).
///
/// A virtual source prints as its label in angle brackets (`<added string 1>`), in
/// diagnostics, `#file` and `#location` alike; the brackets are not part of the label.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SourcePath {
    File(PathBuf),
    Virtual(String),
}

impl SourcePath {
    /// A source with no file on disk, named by `label` (written without the angle brackets).
    pub fn virtual_source(label: impl Into<String>) -> Self {
        SourcePath::Virtual(label.into())
    }

    /// The path of a source that is a file, `None` for a virtual one.
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            SourcePath::File(path) => Some(path),
            SourcePath::Virtual(_) => None,
        }
    }

    pub fn is_virtual(&self) -> bool {
        matches!(self, SourcePath::Virtual(_))
    }

    /// The name as a path, for splitting it into a directory and a file name. A virtual
    /// source's path is its printed label.
    pub fn to_path(&self) -> Cow<'_, Path> {
        match self {
            SourcePath::File(path) => Cow::Borrowed(path),
            SourcePath::Virtual(_) => Cow::Owned(PathBuf::from(self.to_string())),
        }
    }
}

impl fmt::Display for SourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourcePath::File(path) => path.display().fmt(f),
            SourcePath::Virtual(label) => write!(f, "<{label}>"),
        }
    }
}

/// Text read as a path names a file; a virtual source is always made with
/// [`SourcePath::virtual_source`].
impl From<PathBuf> for SourcePath {
    fn from(path: PathBuf) -> Self {
        SourcePath::File(path)
    }
}

impl From<&Path> for SourcePath {
    fn from(path: &Path) -> Self {
        SourcePath::File(path.to_path_buf())
    }
}

impl From<String> for SourcePath {
    fn from(path: String) -> Self {
        SourcePath::File(path.into())
    }
}

impl From<&str> for SourcePath {
    fn from(path: &str) -> Self {
        SourcePath::File(path.into())
    }
}

#[derive(Clone)]
pub struct SourceFile {
    pub path: SourcePath,
    pub text: Rc<str>,
    lines: LineIndex,
}

impl SourceFile {
    pub fn new(path: SourcePath, text: Rc<str>) -> Self {
        let lines = LineIndex::new(&text);
        Self {
            path,
            text,
            lines,
        }
    }

    /// A source with no file on disk: `#import,string`, `add_build_string` or an expansion.
    pub fn is_virtual(&self) -> bool {
        self.path.is_virtual()
    }

    /// A file whose line index was built elsewhere, from the same `text`.
    pub fn with_lines(path: SourcePath, text: Rc<str>, lines: LineIndex) -> Self {
        Self {
            path,
            text,
            lines,
        }
    }

    /// The file's line index, built when it was added.
    pub fn lines(&self) -> &LineIndex {
        &self.lines
    }

    /// One-based line and column of a byte offset. The column counts bytes, as the compiler's
    /// own locations (`#location`, debug info) do; `LineIndex::line_col` counts characters.
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let (line, col) = self.lines.line_byte_col(offset as usize);
        (line as u32, col as u32)
    }

    /// Byte offset of a one-based line and column (clamped to the file).
    pub fn offset_of(&self, line: u32, col: u32) -> u32 {
        let index = (line.max(1) as usize - 1).min(self.lines.line_count() - 1);
        let start = self.lines.line_range(index).map_or(0, |r| r.0) as u32;
        (start + col.max(1) - 1).min(self.text.len() as u32)
    }

    /// The text of one-based `line`, without its line ending.
    pub fn line_text(&self, line: u32) -> &str {
        let (start, end) = self
            .lines
            .line_range(line as usize - 1)
            .unwrap_or((self.text.len(), self.text.len()));
        &self.text[start..end]
    }
}

#[derive(Clone, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

static TYPECHECKED_NAMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether any source read so far names `Message_Typechecked` outside the files that define
/// and deliver it. When none does, no code can look inside a TYPECHECKED message and the
/// contents are not exported. It only ever turns on, and for the whole process, so a compiler
/// that cannot be sure keeps the full export.
pub fn typechecked_named() -> bool {
    TYPECHECKED_NAMED.load(std::sync::atomic::Ordering::Relaxed)
}

fn note_typechecked_name(path: &SourcePath, text: &str) {
    if typechecked_named() || !text.contains("Message_Typechecked") {
        return;
    }
    // The record type itself and the delivery code of the `Compiler` module.
    let own = path.to_string().replace('\\', "/");
    if [
        "Compiler/contract-records.jai",
        "compiler-api/messages.jai",
        "compiler-api/registry.jai",
    ]
    .iter()
    .any(|file| own.ends_with(file))
    {
        return;
    }
    TYPECHECKED_NAMED.store(true, std::sync::atomic::Ordering::Relaxed);
}

impl SourceMap {
    pub fn add(&mut self, path: impl Into<SourcePath>, text: Rc<str>) -> FileId {
        let path = path.into();
        note_typechecked_name(&path, &text);
        self.files.push(SourceFile::new(path, text));
        FileId(self.files.len() as u32 - 1)
    }

    pub fn add_file(&mut self, file: SourceFile) -> FileId {
        note_typechecked_name(&file.path, &file.text);
        self.files.push(file);
        FileId(self.files.len() as u32 - 1)
    }

    /// The file `id` names. Only for ids of files known to exist; a span's file may be
    /// `Span::NONE`'s (builtins such as `OS` are declared nowhere), so take it with `try_get`.
    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id.0 as usize]
    }

    /// The file `id` names, or `None` for an id with no file (`Span::NONE`).
    pub fn try_get(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// The text of `span`, or "" for a span without a place: no file (`Span::NONE`), or a
    /// range that is reversed, past the end or not on character boundaries.
    pub fn snippet(&self, span: Span) -> &str {
        self.try_get(span.file)
            .and_then(|f| f.text.get(span.start as usize..span.end as usize))
            .unwrap_or("")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// What a diagnostic reports, for code that treats particular errors specially: the message
/// is for people, and nothing matches its text. Tools outside the compiler see it as `code()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DiagnosticKind {
    #[default]
    Other,
    /// A name no scope in reach declares. `scope` is the index of the sema scope the lookup
    /// started from, for the "did you mean" help worked out when the error is rendered.
    UnknownIdentifier { scope: Option<u32> },
    /// A value whose type does not convert to the type expected there.
    TypeMismatch,
    /// A concrete generic argument fails a structural interface restriction.
    InterfaceConstraint,
    /// A failed `#assert`.
    StaticAssert,
    /// A `#foreign` procedure names a library no `#library` declares (often one declared only
    /// under another OS's `#if`).
    UnknownLibrary,
    /// A `#load` names a file that does not exist.
    MissingFile,
    /// The program called a foreign procedure this host cannot provide (the browser's sandbox
    /// has no native libraries).
    Unavailable,
    /// A declaration is being reflected without concrete generic arguments.
    /// Consumers may retain symbolic metadata; ordinary semantic failures must
    /// still propagate rather than being mistaken for an absent default.
    UnboundGenericParameter,
    /// A `#modify` block on a procedure or struct with no polymorph variables, which would
    /// never run. Reported at the declaration, not as a call's mismatch.
    ModifyWithoutPolymorphs,
    /// The compiled program stopped with a trap (a failed check, an assertion) while running
    /// `main`, after it compiled; a trap in compile-time code is a compile error instead.
    Runtime,
    /// A `#complete` switch leaves out members of its enum.
    IncompleteSwitch,
    /// The parser gave up past `MAX_NESTING` levels. Error recovery stops at it rather than
    /// rebuilding the enclosing blocks; it has no code of its own.
    NestingLimit,
    /// Compile-time code was not run because the workspace is untrusted (editor trust mode):
    /// the value is unknown, not wrong, so editors do not report it.
    Untrusted,
}

impl DiagnosticKind {
    /// A stable name for tools (the playground's JSON, the language server), or `None` for
    /// diagnostics without a kind of their own.
    pub fn code(self) -> Option<&'static str> {
        match self {
            DiagnosticKind::Other | DiagnosticKind::NestingLimit => None,
            DiagnosticKind::UnknownIdentifier {
                ..
            } => Some("unknown-identifier"),
            DiagnosticKind::TypeMismatch => Some("type-mismatch"),
            DiagnosticKind::IncompleteSwitch => Some("incomplete-switch"),
            DiagnosticKind::InterfaceConstraint => Some("interface-constraint"),
            DiagnosticKind::StaticAssert => Some("static-assert"),
            DiagnosticKind::UnknownLibrary => Some("unknown-library"),
            DiagnosticKind::MissingFile => Some("missing-file"),
            DiagnosticKind::Unavailable => Some("unavailable"),
            DiagnosticKind::UnboundGenericParameter => Some("unbound-generic-parameter"),
            DiagnosticKind::ModifyWithoutPolymorphs => Some("modify-without-polymorphs"),
            DiagnosticKind::Runtime => Some("runtime-error"),
            DiagnosticKind::Untrusted => Some("untrusted"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
    pub kind: DiagnosticKind,
    /// Text under the primary span's carets (outside the plain layout). Boxed, as `fixes`
    /// is, to keep `Result<_, Diagnostic>` small.
    pub label: Option<Box<str>>,
    /// Related locations; a note whose span has no file (`Span::NONE`) prints as text only.
    /// Outside the plain layout, a located note is a label in the snippet.
    pub notes: Vec<(Span, String)>,
    /// `help:` lines printed last: how to fix the problem, when that is clear.
    pub help: Vec<String>,
    /// What the help lines suggest, in a form tools can apply.
    pub fixes: Option<Box<Fixes>>,
}

/// The changes a diagnostic's help suggests. Boxed in the diagnostic, as its label is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fixes {
    /// The fix the first help line describes: replace this span with this text. Shown as the
    /// changed line under the help, outside the plain layout.
    pub replace: Option<(Span, String)>,
    /// The `#import`s that would declare an unknown name, best first (worked out with the "did
    /// you mean" help when the error is rendered): the help names the first, an editor offers
    /// each.
    pub imports: Vec<ImportSuggestion>,
}

/// An `#import` that would bring an unknown name into scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSuggestion {
    /// The module to import, as written in `#import "module"`.
    pub module: String,
    /// The name to bind the module to (`name :: #import "module";`) when the unknown name is
    /// the module itself, used as a namespace (`Math.sqrt`); `None` imports its names.
    pub name: Option<String>,
}

impl ImportSuggestion {
    /// The import statement, as it would be written: `#import "Basic";`.
    pub fn statement(&self) -> String {
        match &self.name {
            Some(name) => format!("{name} :: #import \"{}\";", self.module),
            None => format!("#import \"{}\";", self.module),
        }
    }
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
            kind: DiagnosticKind::Other,
            label: None,
            notes: Vec::new(),
            help: Vec::new(),
            fixes: None,
        }
    }

    #[must_use]
    pub fn with_kind(mut self, kind: DiagnosticKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into().into_boxed_str());
        self
    }

    #[must_use]
    pub fn with_note(mut self, span: Span, message: impl Into<String>) -> Self {
        self.notes.push((span, message.into()));
        self
    }

    #[must_use]
    pub fn with_help(mut self, message: impl Into<String>) -> Self {
        self.help.push(message.into());
        self
    }

    /// A help line with the replacement it suggests.
    #[must_use]
    pub fn with_fix(
        mut self,
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.help.insert(0, message.into());
        self.fixes.get_or_insert_default().replace = Some((span, replacement.into()));
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
                    lines: Some(file.lines()),
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
        report.primary = label(self.span, self.label.as_deref().map(String::from));
        for (span, note) in &self.notes {
            match label(*span, Some(note.clone())) {
                Some(l) => report.secondary.push(l),
                None => report.notes.push(note.clone()),
            }
        }
        for (i, help) in self.help.iter().enumerate() {
            let fix = self
                .fixes
                .as_deref()
                .and_then(|f| f.replace.as_ref())
                .filter(|_| i == 0)
                .and_then(|(span, text)| {
                    has_location(sources, *span).then(|| Fix {
                        text: &sources.get(span.file).text,
                        edits: vec![(span.start as usize, span.end as usize, text.clone())],
                        lines: Some(sources.get(span.file).lines()),
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

/// Whether `span` names a place in a file (`Span::NONE` does not).
fn has_location(sources: &SourceMap, span: Span) -> bool {
    sources.try_get(span.file).is_some()
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests;
