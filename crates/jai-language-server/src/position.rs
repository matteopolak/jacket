use crate::Error;
use jaic::source::Utf16Error;
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

    pub(crate) fn byte(&self, text: &str, position: Position) -> Result<usize, Error> {
        self.0
            .offset_utf16(text, position.line, position.character)
            .map_err(|error| match error {
                Utf16Error::LineOutside => Error::Position("line is outside the document"),
                Utf16Error::SplitsPair => {
                    Error::Position("UTF-16 position splits a surrogate pair")
                }
            })
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
