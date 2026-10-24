use crate::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// The language server's view of `jaic::source::LineIndex`: positions in the protocol's
/// terms (`Position`, UTF-16 columns) with the protocol's errors.
#[derive(Clone, Debug)]
pub(crate) struct LineIndex(jaic::source::LineIndex);

impl LineIndex {
    pub(crate) fn new(text: &str) -> Self {
        Self(jaic::source::LineIndex::new(text))
    }

    /// Bring the index up to date after an edit that changed `text` from byte `from` on.
    pub(crate) fn update(&mut self, text: &str, from: usize) {
        self.0.update(text, from);
    }

    /// The byte offset of `position`. One policy serves requests and edits alike, per LSP
    /// 3.17: a line past the last names the end of the document, a character past the end of
    /// its line names the line's end, and one inside a surrogate pair names the start of the
    /// character. No position is an error.
    pub(crate) fn byte(&self, text: &str, position: Position) -> usize {
        self.0.offset_utf16(text, position.line, position.character)
    }

    pub(crate) fn position(&self, text: &str, byte: usize) -> Result<Position, Error> {
        let (line, character) = self
            .0
            .utf16_position(text, byte)
            .ok_or(Error::Position("byte position is outside a UTF-8 boundary"))?;
        Ok(Position {
            line,
            character,
        })
    }

    pub(crate) fn range(&self, text: &str, span: crate::analysis::Span) -> Result<Range, Error> {
        Ok(Range {
            start: self.position(text, span.start)?,
            end: self.position(text, span.end)?,
        })
    }

    pub(crate) fn lines(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.0.lines()
    }
}
