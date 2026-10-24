//! Inlay hints that name what a closing brace closes (`}` of a long procedure, `// main`), as
//! rust-analyzer's `closingBraceHints` do.
use crate::session::Session;
use crate::{DocumentSymbol, DocumentUri, Error, InlayHint, Range, SymbolKind};

/// Procedures, structs and enums at least this many lines long get a hint
/// (`inlayHints.closingBraceMinLines`); 0 turns them off.
pub(crate) const DEFAULT_MIN_LINES: usize = 25;

impl Session {
    /// Choose how long a declaration must be to get a hint at its closing brace; 0 for never.
    pub fn set_closing_brace_min_lines(&mut self, lines: usize) {
        self.closing_brace_min_lines = lines;
    }

    /// The hints of the type checker (inferred types, argument names, `#run` values) and the
    /// closing-brace names, in document order.
    pub fn inlay_hints(&self, uri: &DocumentUri, range: Range) -> Result<Vec<InlayHint>, Error> {
        let mut hints = self.typed_hints(uri, range)?;
        hints.extend(self.closing_brace_hints(uri, range)?);
        hints.sort_by_key(|h| (h.position.line, h.position.character));
        Ok(hints)
    }

    fn closing_brace_hints(
        &self,
        uri: &DocumentUri,
        range: Range,
    ) -> Result<Vec<InlayHint>, Error> {
        if self.closing_brace_min_lines == 0 {
            return Ok(Vec::new());
        }
        let doc = self.document(uri)?;
        let mut hints = Vec::new();
        let mut pending = self.document_symbols(uri)?;
        while let Some(symbol) = pending.pop() {
            pending.extend(symbol.children.iter().cloned());
            let DocumentSymbol {
                name,
                kind,
                range: whole,
                ..
            } = symbol;
            let lines = (whole.end.line - whole.start.line) as usize + 1;
            if !matches!(
                kind,
                SymbolKind::Function | SymbolKind::Struct | SymbolKind::Enum
            ) || lines < self.closing_brace_min_lines
            {
                continue;
            }
            let end = doc.index.byte(&doc.text, whole.end);
            // The declaration may end in `;` or trail whitespace after its brace.
            let trimmed = doc.text[..end].trim_end().trim_end_matches(';').trim_end();
            if !trimmed.ends_with('}') {
                continue;
            }
            let Ok(position) = doc.index.position(&doc.text, trimmed.len()) else {
                continue;
            };
            if position.line < range.start.line || position.line > range.end.line {
                continue;
            }
            hints.push(InlayHint {
                position,
                label: format!("// {name}"),
                kind: None,
                padding_left: true,
                padding_right: false,
                tooltip: None,
            });
        }
        Ok(hints)
    }
}
