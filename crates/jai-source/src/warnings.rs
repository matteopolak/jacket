//! Nonfatal language diagnostics retain their exact immutable source allocation.
use crate::{SourceRecord, SourceSpan};
use std::{path::Path, sync::Arc};

/// Bound retained diagnostic work; exceeding this limit is an explicit error.
pub const MAX_SOURCE_WARNINGS: usize = 16_384;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarningLocation {
    location: SourceSpan,
    path: Arc<Path>,
    text: Arc<str>,
    line: usize,
    column: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningLocationError {
    WrongSource,
    InvalidSpan,
}

impl WarningLocation {
    pub fn new(source: &SourceRecord, location: SourceSpan) -> Result<Self, WarningLocationError> {
        if source.id() != location.source {
            return Err(WarningLocationError::WrongSource);
        }
        let text = source.text();
        let span = location.span;
        if span.start > span.end
            || span.end > text.len()
            || !text.is_char_boundary(span.start)
            || !text.is_char_boundary(span.end)
        {
            return Err(WarningLocationError::InvalidSpan);
        }
        let prefix = &text[..span.start];
        Ok(Self {
            location,
            path: Arc::from(source.path()),
            text: source.shared_text(),
            line: prefix.bytes().filter(|&byte| byte == b'\n').count() + 1,
            column: prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
        })
    }
    pub fn span(&self) -> SourceSpan {
        self.location
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
    pub fn text(&self) -> &str {
        self.location.span.text(&self.text)
    }
    pub fn shared_text(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }
    pub fn matches_source(&self, source: &SourceRecord) -> bool {
        self.location.source == source.id()
            && self.path.as_ref() == source.path()
            && Arc::ptr_eq(&self.text, &source.shared_text())
    }
    pub fn same_source(&self, other: &Self) -> bool {
        self.location.source == other.location.source
            && self.path == other.path
            && Arc::ptr_eq(&self.text, &other.text)
    }
    fn prefix(&self) -> String {
        format!("{}:{}:{}", self.path.display(), self.line, self.column)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceWarningKind {
    DeprecatedProcedureReference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarningNote {
    pub location: WarningLocation,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceWarning {
    kind: SourceWarningKind,
    location: WarningLocation,
    message: String,
    notes: Vec<WarningNote>,
}
impl SourceWarning {
    pub fn new(
        kind: SourceWarningKind,
        location: WarningLocation,
        message: impl Into<String>,
        notes: Vec<WarningNote>,
    ) -> Self {
        Self {
            kind,
            location,
            message: message.into(),
            notes,
        }
    }
    pub fn kind(&self) -> SourceWarningKind {
        self.kind
    }
    pub fn location(&self) -> &WarningLocation {
        &self.location
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn notes(&self) -> &[WarningNote] {
        &self.notes
    }
    pub fn render(&self) -> String {
        let mut rendered = format!("{}: warning: {}", self.location.prefix(), self.message);
        for note in &self.notes {
            rendered.push_str(&format!(
                "\n{}: note: {}",
                note.location.prefix(),
                note.message
            ));
        }
        rendered
    }
}
