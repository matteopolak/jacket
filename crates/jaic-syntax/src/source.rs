//! Source files, spans and diagnostics.
use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod suggestion;
pub use suggestion::{Applicability, Edit, ImportSuggestion, Suggestion, Suggestions};

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
/// Lines end at `\n`, `\r\n` or a lone `\r`, the three endings the language server protocol
/// names; the line ending is not part of the line (`line_end` excludes it). Offsets are bytes. Columns are counted in characters
/// (`line_col`) or UTF-16 units (`utf16_position`, the language server protocol's unit).
/// Building is one pass; every query after that is a binary search, so callers that ask
/// about many offsets of one text build the index once and keep it (`SourceFile::lines`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineIndex {
    starts: Vec<u32>,
    ends: Vec<u32>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut index = Self {
            starts: vec![0],
            ends: Vec::new(),
        };
        index.scan(text);
        index
    }

    /// Bring the index up to date after an edit: `text` is the new text and `from` the first
    /// byte that differs from the text the index was built for. Lines before the one holding
    /// `from` keep their entries; that line and every one after it are scanned again.
    pub fn update(&mut self, text: &str, from: usize) {
        let mut from = from.min(text.len());
        // A `\n` typed after a `\r` turns that line ending into `\r\n`.
        if from > 0 && text.as_bytes()[from - 1] == b'\r' {
            from -= 1;
        }
        let line = self.line_of(from);
        self.starts.truncate(line + 1);
        self.ends.truncate(line);
        self.scan(text);
    }

    /// Record the lines of `text` from the start of the last known line to the end.
    fn scan(&mut self, text: &str) {
        let bytes = text.as_bytes();
        let mut at = self.starts.last().map_or(0, |&start| start as usize);
        while let Some(found) = bytes[at..].iter().position(|&b| b == b'\n' || b == b'\r') {
            let end = at + found;
            let next = if bytes[end] == b'\r' && bytes.get(end + 1) == Some(&b'\n') {
                end + 2
            } else {
                end + 1
            };
            self.ends.push(end as u32);
            self.starts.push(next as u32);
            at = next;
        }
        self.ends.push(text.len() as u32);
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

    /// The offset of zero-based `line` and UTF-16 column `column` in `text`, clamped the way
    /// the language server protocol asks: a line past the last one names the end of the text,
    /// a column past the line's end names the line's end, and a column inside a surrogate pair
    /// names the start of that character.
    pub fn offset_utf16(&self, text: &str, line: u32, column: u32) -> usize {
        let Some((start, end)) = self.line_range(line as usize) else {
            return text.len();
        };
        let mut units = 0u32;
        for (relative, ch) in text[start..end].char_indices() {
            units += ch.len_utf16() as u32;
            if units > column {
                return start + relative;
            }
        }
        end
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
    let bytes = text.as_bytes();
    let mut end = offset.min(bytes.len());
    // The `\n` of a `\r\n` still belongs to the line the `\r` ends.
    if end > 0 && bytes.get(end) == Some(&b'\n') && bytes[end - 1] == b'\r' {
        end -= 1;
    }
    bytes[..end]
        .iter()
        .rposition(|&b| b == b'\n' || b == b'\r')
        .map_or(0, |n| n + 1)
}

/// The end of the line holding `offset` in `text`, before its line break (`\n`, `\r\n` or a
/// lone `\r`); the counterpart of `line_start`.
pub fn line_end(text: &str, offset: usize) -> usize {
    let bytes = text.as_bytes();
    let mut from = offset.min(bytes.len());
    if from > 0 && bytes.get(from) == Some(&b'\n') && bytes[from - 1] == b'\r' {
        from -= 1;
    }
    from + bytes[from..]
        .iter()
        .position(|&b| b == b'\n' || b == b'\r')
        .unwrap_or(bytes.len() - from)
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

/// The files of a map, in id order. Append-only and shared by every map made with
/// `SourceMap::share`: a cross build's two compilers number their files from one table, so a
/// span means the same file to both.
#[derive(Default)]
struct FileTable {
    /// Only `push` writes, and it never moves or drops a file once added (a `SourceFile` is
    /// reached through its `Rc` allocation, not through this vector), so references handed out
    /// by `get` stay valid while the table grows.
    files: std::cell::UnsafeCell<Vec<Rc<SourceFile>>>,
    /// How far `SourceMap::reads_procedure_bodies` has looked, and what it found.
    scanned: std::cell::Cell<usize>,
}

impl FileTable {
    fn push(&self, file: SourceFile) -> FileId {
        // SAFETY: the compiler is single-threaded, no reference into the vector outlives this
        // call (`get` copies a pointer to the file, not to the vector's storage), and `push`
        // only appends.
        #[allow(unsafe_code)]
        let files = unsafe { &mut *self.files.get() };
        files.push(Rc::new(file));
        FileId(files.len() as u32 - 1)
    }

    fn get(&self, id: FileId) -> Option<&SourceFile> {
        // SAFETY: see `push`. The returned reference points into the file's `Rc` allocation,
        // which the table keeps alive for its own lifetime.
        #[allow(unsafe_code)]
        let files = unsafe { &*self.files.get() };
        let file = files.get(id.0 as usize)?;
        // SAFETY: files are never removed, so the `Rc` outlives `&self`.
        #[allow(unsafe_code)]
        Some(unsafe { &*Rc::as_ptr(file) })
    }

    fn len(&self) -> usize {
        // SAFETY: see `push`.
        #[allow(unsafe_code)]
        unsafe {
            (*self.files.get()).len()
        }
    }
}

#[derive(Default)]
pub struct SourceMap {
    table: Rc<FileTable>,
    typechecked_use: TypecheckedUse,
}

impl Clone for SourceMap {
    /// A snapshot: later additions to either map are not seen by the other (`share` is the map
    /// that follows).
    fn clone(&self) -> Self {
        let copy = FileTable::default();
        // SAFETY: see `FileTable::push`.
        #[allow(unsafe_code)]
        let files = unsafe { &*self.table.files.get() };
        // SAFETY: `copy` is not shared yet.
        #[allow(unsafe_code)]
        unsafe {
            (*copy.files.get()).extend(files.iter().cloned());
        }
        Self {
            table: Rc::new(copy),
            typechecked_use: self.typechecked_use.clone(),
        }
    }
}

/// What the sources of one build say about reading `Message_Typechecked`, shared by every
/// compiler of the build (see `Compiler::attach_workspaces`). Bits only ever turn on, so a
/// build that cannot be sure keeps the full export.
pub type TypecheckedUse = Rc<TypecheckedState>;

/// The bits of a `TypecheckedUse`, and the file tables of the maps that report to it (so the
/// files of every compiler of the build can be searched for the readers of procedure bodies).
#[derive(Default)]
pub struct TypecheckedState {
    bits: std::cell::Cell<u8>,
    tables: std::cell::RefCell<Vec<Rc<FileTable>>>,
}

impl TypecheckedState {
    pub fn get(&self) -> u8 {
        self.bits.get()
    }

    fn set(&self, bits: u8) {
        self.bits.set(bits);
    }

    fn register(&self, table: &Rc<FileTable>) {
        let mut tables = self.tables.borrow_mut();
        if !tables.iter().any(|known| Rc::ptr_eq(known, table)) {
            tables.push(table.clone());
        }
    }
}

/// Some source names `Message_Typechecked` outside the files that define and deliver it:
/// no code can look inside a TYPECHECKED message when none does, and the contents are not
/// exported.
pub const TYPECHECKED_NAMED: u8 = 1;

/// Some source can read a procedure body in a TYPECHECKED message (found by
/// `SourceMap::reads_procedure_bodies`).
pub const TYPECHECKED_BODIES: u8 = 2;

/// Where the `#string` at byte `at` ends: after its delimiter, or at the end of the text.
fn here_string_end(text: &str, at: usize) -> usize {
    let rest = &text[at + "#string".len()..];
    let line_end = rest.find('\n').unwrap_or(rest.len());
    let header = rest[..line_end].trim_start_matches(|c: char| c != ' ' && c != '\t');
    let delimiter = header.trim();
    let body = at + "#string".len() + line_end;
    if delimiter.is_empty() {
        return text.len();
    }
    text[body..]
        .find(delimiter)
        .map_or(text.len(), |found| body + found + delimiter.len())
}

/// `text` with its comments blanked, for a search that must not count a word in a comment.
/// Words in strings stay: they may be code that is inserted. A `#string` runs to its
/// delimiter, and comment markers in it are text.
fn without_comments(text: &str) -> std::borrow::Cow<'_, str> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let (mut i, mut depth, mut in_string) = (0, 0usize, false);
    while i < bytes.len() {
        let b = bytes[i];
        let next = bytes.get(i + 1).copied();
        if depth > 0 {
            if b == b'/' && next == Some(b'*') {
                depth += 1;
                i += 2;
            } else if b == b'*' && next == Some(b'/') {
                depth -= 1;
                i += 2;
            } else {
                i += 1;
            }
            out.push(b' ');
            continue;
        }
        if in_string {
            if b == b'\\' && next.is_some() {
                out.extend_from_slice(&bytes[i..i + 2]);
                i += 2;
                continue;
            }
            in_string = b != b'"';
        } else if b == b'"' {
            in_string = true;
        } else if bytes[i..].starts_with(b"#string") {
            let end = here_string_end(text, i);
            out.extend_from_slice(&bytes[i..end]);
            i = end;
            continue;
        } else if b == b'/' && next == Some(b'/') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            out.push(b' ');
            continue;
        } else if b == b'/' && next == Some(b'*') {
            depth = 1;
            i += 2;
            out.push(b' ');
            continue;
        }
        out.push(b);
        i += 1;
    }
    // Only whole comments were replaced by a space, so what is left is still valid UTF-8.
    String::from_utf8(out).map_or_else(|_| text.into(), Into::into)
}

/// Whether `text` can read the body of a procedure that a TYPECHECKED message reports: it
/// names the members that lead to one (`procedure_bodies`, `body_or_null`, `subexpressions`),
/// the node type or kind of a body, `.all` (the list that includes the bodies) or, in a file
/// that names the message, a bare `all`. Comments do not count.
fn can_read_procedure_bodies(text: &str) -> bool {
    const NAMES: [&str; 5] = [
        "procedure_bodies",
        "body_or_null",
        "subexpressions",
        "Code_Procedure_Body",
        "PROCEDURE_BODY",
    ];
    let names_message = text.contains("Message_Typechecked");
    let candidate = (names_message && text.contains("all"))
        || text.contains(".all")
        || NAMES.iter().any(|name| text.contains(name));
    if !candidate {
        return false;
    }
    let code = without_comments(text);
    let word = |at: usize, len: usize| {
        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let bytes = code.as_bytes();
        let after = bytes.get(at + len).copied();
        !after.is_some_and(ident) && (at == 0 || !ident(bytes[at - 1]))
    };
    NAMES.iter().any(|name| code.contains(name))
        || code.match_indices(".all").any(|(at, _)| word(at + 1, 3))
        || (names_message && code.match_indices("all").any(|(at, _)| word(at, 3)))
}

fn note_typechecked_name(usage: &TypecheckedUse, path: &SourcePath, text: &str) {
    if usage.get() & TYPECHECKED_NAMED != 0 || !text.contains("Message_Typechecked") {
        return;
    }
    // The record type itself and the delivery code of the `Compiler` module.
    let own = crate::paths::to_slash(&path.to_string());
    if ["Compiler/contract-records.jai"]
        .iter()
        .any(|file| own.ends_with(file))
    {
        return;
    }
    usage.set(usage.get() | TYPECHECKED_NAMED);
}

impl SourceMap {
    /// Whether any source of the build can read the body of a procedure reported by a
    /// TYPECHECKED message (see `can_read_procedure_bodies`). Files are looked at once, as they
    /// appear, so ask only when a metaprogram names the message. The record definition and the
    /// `Compiler` module's declarations do not count.
    pub fn reads_procedure_bodies(&self) -> bool {
        let usage = &self.typechecked_use;
        usage.register(&self.table);
        if usage.get() & TYPECHECKED_BODIES != 0 {
            return true;
        }
        for table in usage.tables.borrow().iter() {
            let upto = table.len();
            for id in table.scanned.get()..upto {
                let Some(file) = table.get(FileId(id as u32)) else {
                    continue;
                };
                let own = crate::paths::to_slash(&file.path.to_string());
                if ["Compiler/contract-records.jai", "Compiler/workspace.jai"]
                    .iter()
                    .any(|name| own.ends_with(name))
                {
                    continue;
                }
                if can_read_procedure_bodies(&file.text) {
                    usage.set(usage.get() | TYPECHECKED_BODIES);
                    return true;
                }
            }
            table.scanned.set(upto);
        }
        false
    }

    /// Report this map's sources to `usage` (the build's) from now on.
    pub fn share_typechecked_use(&mut self, usage: TypecheckedUse) {
        usage.set(usage.get() | self.typechecked_use.get());
        usage.register(&self.table);
        self.typechecked_use = usage;
    }

    pub fn add(&mut self, path: impl Into<SourcePath>, text: Rc<str>) -> FileId {
        let path = path.into();
        note_typechecked_name(&self.typechecked_use, &path, &text);
        self.table.push(SourceFile::new(path, text))
    }

    pub fn add_file(&mut self, file: SourceFile) -> FileId {
        note_typechecked_name(&self.typechecked_use, &file.path, &file.text);
        self.table.push(file)
    }

    /// A map over the same files: what either adds, both see, under the same ids.
    #[must_use]
    pub fn share(&self) -> Self {
        Self {
            table: self.table.clone(),
            typechecked_use: self.typechecked_use.clone(),
        }
    }

    /// The file `id` names. Only for ids of files known to exist; a span's file may be
    /// `Span::NONE`'s (builtins such as `OS` are declared nowhere), so take it with `try_get`.
    pub fn get(&self, id: FileId) -> &SourceFile {
        self.table
            .get(id)
            .expect("invariant: a span names a file of its map")
    }

    /// The file `id` names, or `None` for an id with no file (`Span::NONE`).
    pub fn try_get(&self, id: FileId) -> Option<&SourceFile> {
        self.table.get(id)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// The files these sources were read from (virtual sources have none).
    pub fn files(&self) -> impl Iterator<Item = &Path> + '_ {
        (0..self.table.len() as u32)
            .filter_map(|id| self.try_get(FileId(id)))
            .filter_map(|file| file.path.as_file())
    }

    pub fn is_empty(&self) -> bool {
        self.table.len() == 0
    }

    /// The text of `span`, or "" for a span without a place: no file (`Span::NONE`), or a
    /// range that is reversed, past the end or not on character boundaries.
    pub fn snippet(&self, span: Span) -> &str {
        self.try_get(span.file)
            .and_then(|f| f.text.get(span.start as usize..span.end as usize))
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests;
