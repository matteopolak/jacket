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

#[derive(Clone, Debug)]
pub(crate) struct LineIndex {
    starts: Vec<usize>,
    ends: Vec<usize>,
}

impl LineIndex {
    pub(crate) fn new(text: &str) -> Self {
        let mut starts = vec![0];
        let mut ends = vec![];
        for (at, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                ends.push(if at > 0 && text.as_bytes()[at - 1] == b'\r' {
                    at - 1
                } else {
                    at
                });
                starts.push(at + 1);
            }
        }
        ends.push(text.len());
        Self {
            starts,
            ends,
        }
    }

    pub(crate) fn byte(&self, text: &str, position: Position) -> Result<usize, Error> {
        let line = position.line as usize;
        let start = *self
            .starts
            .get(line)
            .ok_or(Error::Position("line is outside the document"))?;
        let end = self.ends[line];
        let mut units = 0u32;
        for (relative, ch) in text[start..end].char_indices() {
            if units == position.character {
                return Ok(start + relative);
            }
            units += ch.len_utf16() as u32;
            if units > position.character {
                return Err(Error::Position("UTF-16 position splits a surrogate pair"));
            }
        }
        // LSP positions beyond a line's character count denote the line end.
        Ok(end)
    }

    pub(crate) fn position(&self, text: &str, byte: usize) -> Result<Position, Error> {
        if byte > text.len() || !text.is_char_boundary(byte) {
            return Err(Error::Position("byte position is outside a UTF-8 boundary"));
        }
        let line = self
            .starts
            .partition_point(|start| *start <= byte)
            .saturating_sub(1);
        let end = byte.min(self.ends[line]);
        let character = text[self.starts[line]..end].encode_utf16().count() as u32;
        Ok(Position {
            line: line as u32,
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
        self.starts.iter().copied().zip(self.ends.iter().copied())
    }
}
