//! Definitions, sources and hovers.
use super::{Session, layout_line};
use crate::analysis::{Span, TokenKind};
use crate::hover::{Block, HoverText, Inline};
use crate::semantic::{self};
use crate::{
    DocumentUri, Error, Hover, Location, MarkupContent, MarkupKind, Position, position::LineIndex,
};
use std::path::Path;

enum MacroHover {
    Captured(Option<jaic::ide::meta::IdeExpansion>),
    LookupAfterDocs,
}

struct HoverSelection {
    facts: semantic::HoverFacts,
    expansion: MacroHover,
}

impl Session {
    pub fn definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let _query = self.discovery_query();
        let fs = self.config_filesystem();
        let (uri, doc) = if let Some(fs) = &fs {
            self.projected_document_entry(fs, uri)?
        } else {
            self.document_entry(uri)?
        };
        let byte = doc.index.byte(&doc.text, position);
        if let Some(location) = self.link_definition(uri, byte) {
            return Ok(location.into_iter().collect());
        }
        if let Some(locations) = self.doc_link_definition(uri, byte) {
            return Ok(locations);
        }
        {
            let locations = self
                .with_best_effort_semantic_at_projected(
                    uri,
                    &doc.text,
                    byte,
                    |a, path, at, shift| {
                        let fs = self.config_filesystem()?;
                        let found = a.definition(path, at);
                        let locations: Vec<Location> = found
                            .into_iter()
                            .filter_map(|(path, text, start, end, file_path)| {
                                let target = if file_path {
                                    self.projected_compiler_file_uri(&fs, Path::new(&path))
                                } else {
                                    self.projected_uri(&fs, Path::new(&path))
                                }?;
                                // A target in this file is where the edits since the compile left it.
                                let moved = target == *uri && !shift.is_none();
                                let (start, end) = if moved {
                                    (shift.to_new(start)?, shift.to_new(end)?)
                                } else {
                                    (start, end)
                                };
                                let text: std::sync::Arc<str> = if moved {
                                    doc.text.as_str().into()
                                } else {
                                    text
                                };
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
                        (!locations.is_empty()).then_some(locations)
                    },
                )
                .map(|(locations, _)| locations)
                .unwrap_or_default();
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

    /// The declaration that introduced the name at `position`, as written: an alias
    /// (`Name :: Other.Name`, a named import) is where `Name` is declared, while `definition`
    /// goes on to what it stands for. A name no open document declares (a module's) has its
    /// definition.
    pub fn declaration(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let rows = self.bindings(uri, position)?;
        if rows.is_empty() {
            return self.definition(uri, position);
        }
        self.row_locations(rows)
    }

    /// The procedures that run for the name at `position`: every procedure of that name with a
    /// body in the open documents the file reaches (overloads share a name), so a call and each
    /// overload lead to all of them. Procedure types, `#foreign` declarations and everything
    /// that is no procedure have none of their own; a name nothing open declares has its
    /// definition.
    pub fn implementation(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let rows = self.bindings(uri, position)?;
        if rows.is_empty() {
            return self.definition(uri, position);
        }
        let mut found: Vec<(&DocumentUri, &crate::analysis::SymbolRow)> = Vec::new();
        for (target, row) in &rows {
            if row.kind != crate::SymbolKind::Function || row.local {
                continue;
            }
            for candidate in self.reachable(target) {
                for other in &self.analyses[candidate].rows {
                    let visible = candidate == *target || !other.file_private;
                    let new = !found
                        .iter()
                        .any(|(u, r)| *u == candidate && r.selection == other.selection);
                    if other.parent.is_none()
                        && !other.local
                        && other.name == row.name
                        && other.kind == crate::SymbolKind::Function
                        && visible
                        && new
                        && self.has_body(candidate, other)
                    {
                        found.push((candidate, other));
                    }
                }
            }
        }
        self.row_locations(found)
    }

    /// Whether the procedure declared by `row` of `uri` is written with a body.
    fn has_body(&self, uri: &DocumentUri, row: &crate::analysis::SymbolRow) -> bool {
        let Some(doc) = self.documents.get(uri) else {
            return false;
        };
        let text = doc
            .text
            .get(row.location.start..row.location.end)
            .unwrap_or("");
        text.contains('{') && !text.contains("#foreign") && !text.contains("#intrinsic")
    }

    fn row_locations(
        &self,
        rows: Vec<(&DocumentUri, &crate::analysis::SymbolRow)>,
    ) -> Result<Vec<Location>, Error> {
        rows.iter()
            .map(|(target, row)| {
                let doc = self.document(target)?;
                Ok(Location {
                    uri: target.as_str().into(),
                    range: doc.index.range(&doc.text, row.selection)?,
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
        let _query = self.discovery_query();
        let mapping = std::cell::RefCell::new(crate::semantic::Observations::default());
        let base = self.config_filesystem();
        let fs = base.as_ref().map(|base| crate::semantic::ObservedFs {
            base,
            proof: Some(&mapping),
            secondary: None,
        });
        let (uri, doc) = match &fs {
            Some(fs) => self.projected_document_entry(fs, uri)?,
            None => self.document_entry(uri)?,
        };
        let byte = doc.index.byte(&doc.text, position);
        let found = |start: usize, end: usize, text: HoverText| -> Result<Option<Hover>, Error> {
            // Syntax-only hovers need the same request-alias guarantee as semantic hovers.
            if self
                .environment
                .as_ref()
                .is_some_and(|env| !mapping.borrow().matches(&*env.fs))
            {
                return Ok(None);
            }
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
        if let Some(link) = self.link_hover(uri, byte) {
            return match link {
                Some((start, end, text)) => found(start, end, text),
                None => Ok(None),
            };
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
        if let Some((start, end, text)) = self.using_filter_hover(uri, byte) {
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
        if let Some(HoverSelection {
            facts,
            expansion,
        }) = self.semantic_hover(uri, &doc.text, byte, kind)
        {
            let (start, end) = (facts.start, facts.end);
            let text = self.hover_with_docs(
                uri,
                facts.text,
                facts.layout.map(layout_line),
                &facts.decls,
                kind,
            );
            let text = match expansion {
                MacroHover::Captured(expansion) => {
                    self.append_macro_expansion(uri, expansion.as_ref(), text)
                }
                MacroHover::LookupAfterDocs => self.with_macro_expansion(uri, start, text),
            };
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

    /// Hover and eligible macro facts from one selected type-checked context.
    fn semantic_hover(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        kind: MarkupKind,
    ) -> Option<HoverSelection> {
        let ((mut facts, expansion), shift) = self.with_best_effort_semantic_at_projected(
            uri,
            text,
            byte,
            |analysis, path, at, shift| {
                let facts = analysis.hover(path, at)?;
                // Markdown links can enrich the compiler before macro lookup. Preserve that
                // ordering, as well as exact lookup for older/alternate/advisory hovers.
                let docs_are_pure = kind == MarkupKind::PlainText
                    || facts.decls.iter().all(|(_, source, start, _)| {
                        crate::docs::declaration(source, *start)
                            .is_none_or(|raw| crate::docs::link_names(&raw).is_empty())
                    });
                let exact = analysis.reference_records_allowed()
                    && shift.is_none()
                    && analysis.compiled_text(path).is_some_and(|compiled| {
                        compiled == self.documents[uri].ide_text().as_ref()
                    });
                let expansion = if exact && docs_are_pure {
                    let file = analysis.file(path)?;
                    MacroHover::Captured(analysis.compiler.ide_expansions(file).into_iter().find(
                        |expansion| {
                            expansion.kind == jaic::ide::meta::IdeExpansionKind::Macro
                                && expansion.span.start as usize == facts.start
                        },
                    ))
                } else {
                    MacroHover::LookupAfterDocs
                };
                Some((facts, expansion))
            },
        )?;
        facts.start = shift.to_new(facts.start)?;
        facts.end = shift.to_new(facts.end)?;
        (facts.end <= text.len()
            && text.is_char_boundary(facts.start)
            && text.is_char_boundary(facts.end))
        .then_some(HoverSelection {
            facts,
            expansion,
        })
    }
}
