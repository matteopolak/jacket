//! Definitions, sources and hovers.
use super::{Session, layout_line};
use crate::analysis::{Span, TokenKind};
use crate::hover::{Block, HoverText, Inline};
use crate::semantic::{self};
use crate::{
    DocumentUri, Error, Hover, Location, MarkupContent, MarkupKind, Position, position::LineIndex,
};
use std::path::Path;

impl Session {
    pub fn definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some(location) = self.link_definition(uri, byte) {
            return Ok(vec![location]);
        }
        if let Some(locations) = self.doc_link_definition(uri, byte) {
            return Ok(locations);
        }
        {
            let found = self
                .with_semantic_at(uri, &doc.text, byte, |a, path| {
                    Some(a.definition(path, byte)).filter(|d| !d.is_empty())
                })
                .unwrap_or_default();
            let locations: Vec<Location> = found
                .into_iter()
                .filter_map(|(path, text, start, end)| {
                    let target = DocumentUri::from_path(Path::new(&path)).ok()?;
                    // An open document's own text and index, else the compiled file's.
                    let range = match self.documents.get(&target) {
                        Some(open) if open.same_positions(&text) => open
                            .index
                            .range(
                                &open.text,
                                Span {
                                    start,
                                    end,
                                },
                            )
                            .ok()?,
                        _ => LineIndex::new(&text)
                            .range(
                                &text,
                                Span {
                                    start,
                                    end,
                                },
                            )
                            .ok()?,
                    };
                    Some(Location {
                        uri: target.as_str().into(),
                        range,
                    })
                })
                .collect();
            if !locations.is_empty() {
                return Ok(locations);
            }
        }
        self.bindings(uri, position)?
            .iter()
            .map(|(target, row)| {
                Ok(Location {
                    uri: target.as_str().into(),
                    range: self
                        .document(target)?
                        .index
                        .range(&self.document(target)?.text, row.selection)?,
                })
            })
            .collect()
    }

    /// Text of a file the editor may not have open (a definition in a module or the stdlib):
    /// the open document's text, else the environment's file system.
    pub fn source(&self, uri: &DocumentUri) -> Option<String> {
        if let Some(key) = self.document_uri(uri)
            && let Some(doc) = self
                .documents
                .get(&key)
                .or_else(|| self.lint_configs.get(&key))
        {
            return Some(doc.text.clone());
        }
        let bytes = self.environment.as_ref()?.fs.read(Path::new(uri.path()))?;
        String::from_utf8(bytes).ok()
    }

    /// Hover as plain text.
    pub fn hover(&self, uri: &DocumentUri, position: Position) -> Result<Option<Hover>, Error> {
        self.hover_as(uri, position, MarkupKind::PlainText)
    }

    /// Hover written as `kind`: Markdown puts code in fenced `jai` blocks and starts each
    /// section (what a macro expands to, what `#run` printed) with a thematic break.
    pub fn hover_as(
        &self,
        uri: &DocumentUri,
        position: Position,
        kind: MarkupKind,
    ) -> Result<Option<Hover>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let found = |start: usize, end: usize, text: HoverText| -> Result<Option<Hover>, Error> {
            Ok(Some(Hover {
                contents: MarkupContent {
                    kind,
                    value: text.render(kind),
                },
                range: doc.index.range(
                    &doc.text,
                    Span {
                        start,
                        end,
                    },
                )?,
            }))
        };
        if let Some((start, end, text)) = self.link_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.format_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.directive_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.cast_flag_hover(&doc.text, byte) {
            return found(start, end, text);
        }
        if doc.has_asm()
            && let Some((start, end, text)) = self.asm_hover(&doc.text, byte)
        {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.doc_link_hover(uri, byte, kind) {
            return found(start, end, text);
        }
        let Some((_, token)) = self.word(uri, position)? else {
            return match self.expansion_hover(uri, byte) {
                Some((start, end, text)) => found(start, end, text),
                None => Ok(None),
            };
        };
        if let Some(facts) = self.semantic_hover(uri, &doc.text, byte) {
            let (start, end) = (facts.start, facts.end);
            let text = self.hover_with_docs(
                uri,
                facts.text,
                facts.layout.map(layout_line),
                &facts.decls,
                kind,
            );
            let text = self.with_macro_expansion(uri, start, text);
            return found(start, end, text);
        }
        let rows = self.bindings(uri, position)?;
        let text = if rows.len() == 1 {
            let mut blocks = vec![Block::Code(
                Self::source_detail(self.document(rows[0].0)?, rows[0].1).into(),
            )];
            let at = rows[0].1.selection.start;
            if let Some(raw) = crate::docs::declaration(&self.documents[rows[0].0].text, at)
                && let Some(doc) = self
                    .render_docs(uri, rows[0].0.path(), at, &[raw], kind)
                    .pop()
            {
                blocks.push(Block::Doc(doc));
            }
            blocks.push(Block::Note(vec![Inline::Text(
                "Source syntax declaration. \
                 Type evaluation and compile-time execution are disabled during editing."
                    .into(),
            )]));
            HoverText::new(blocks)
        } else if token.kind == TokenKind::Keyword {
            HoverText::new(vec![Block::Para(vec![
                Inline::Text("keyword ".into()),
                Inline::Code(token.spelling(&doc.text).into()),
            ])])
        } else {
            return Ok(None);
        };
        found(token.span.start, token.span.end, text)
    }

    /// Hover from the type checker, on the text as typed.
    pub(crate) fn semantic_hover(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<semantic::HoverFacts> {
        let facts = self.with_semantic_at(uri, text, byte, |a, path| a.hover(path, byte))?;
        (facts.end <= text.len()
            && text.is_char_boundary(facts.start)
            && text.is_char_boundary(facts.end))
        .then_some(facts)
    }
}
