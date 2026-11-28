//! Definitions, sources and hovers.
use std::path::Path;

use super::{Session, layout_line};
use crate::analysis::{Span, TokenKind};
use crate::hover::{Block, HoverText, Inline};
use crate::semantic::{self};
use crate::{
    DocumentUri, Error, Hover, Location, MarkupContent, MarkupKind, Position, position::LineIndex,
};

impl Session {
    /// Where the name at `position` is declared: usually one place, several for a name that
    /// stands for more than one declaration, none when nothing is found. Fails when the document
    /// is not open.
    pub fn definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position);

        if let Some(location) = self.link_definition(uri, byte) {
            return Ok(vec![location]);
        }

        if let Some(locations) = self.doc_link_definition(uri, byte) {
            return Ok(locations);
        }

        {
            let (mut found, shift) = self
                .with_semantic_at(uri, &doc.text, byte, |a, path, at| {
                    let definitions = a.definition(path, at);
                    let builtin = a
                        .file(path)
                        .is_some_and(|file| a.compiler.ide_builtin_reference(file, at as u32));

                    (!definitions.is_empty() || builtin).then_some(definitions)
                })
                .unwrap_or_default();

            // The compile left library bodies unchecked, and with them the generic instances
            // that read a note from there: check them once for this question.
            if found.is_empty() && self.may_compile() && in_note(&doc.text, byte) {
                found = self.note_readers_unskipped(uri, &doc.text, byte);
            }

            let locations: Vec<Location> = found
                .into_iter()
                .filter_map(|(path, text, start, end)| {
                    if let Some(label) = generated_label(&path) {
                        let uri = self.generated_uri(label, &text);
                        let range = LineIndex::new(&text)
                            .range(
                                &text,
                                Span {
                                    start,
                                    end,
                                },
                            )
                            .ok()?;

                        return Some(Location {
                            uri,
                            range,
                        });
                    }

                    let target = self.client_uri(Path::new(&path))?;
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

    /// The compile-time code that read the note at `byte`, from a compile that checks every
    /// library body. The compile is not kept.
    fn note_readers_unskipped(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Vec<crate::semantic::Decl> {
        let Some(environment) = self.environment.as_ref() else {
            return Vec::new();
        };
        let Some((files, root, dirs)) = self.compile_inputs(uri, text) else {
            return Vec::new();
        };
        let Some(mut analysis) =
            self.semantic
                .borrow_mut()
                .compile_detached_unskipped(environment, &root, &dirs, files)
        else {
            return Vec::new();
        };
        let path = Path::new(uri.path());

        analysis.focus(path);
        analysis.definition(path, byte)
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

    /// The `jai-generated:` URI of a source with no file (`label` as it prints, without the
    /// angle brackets), which this session then serves: named by the text, so the same code
    /// has the same document whichever program generated it.
    pub(crate) fn generated_uri(&self, label: &str, text: &std::sync::Arc<str>) -> String {
        let mut hasher = jaic::fxhash::FxHasher::default();

        std::hash::Hasher::write(&mut hasher, text.as_bytes());
        let name: String = label
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let uri = format!(
            "jai-generated:///{:016x}/{name}.jai",
            std::hash::Hasher::finish(&hasher)
        );

        self.generated
            .borrow_mut()
            .insert(uri.clone(), text.clone());
        uri
    }

    /// The text of a `jai-generated:` document a location was given for.
    pub fn generated_source(&self, uri: &str) -> Option<String> {
        self.generated.borrow().get(uri).map(ToString::to_string)
    }

    /// Text of a file the editor may not have open (a definition in a module or the stdlib):
    /// the open document's text, else the environment's file system, decoded like the compiler.
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
        Some(String::from_utf8_lossy(&bytes).into_owned())
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
        if self.config_document(uri).is_some() {
            return self.manifest_hover(uri, position, kind);
        }

        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position);
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
                Self::source_detail(&self.document(rows[0].0)?.text, rows[0].1).into(),
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
        let (mut facts, shift) =
            self.with_semantic_at(uri, text, byte, semantic::Analysis::hover)?;

        // Past an edit the compile did not see, the word hovered is where it is now.
        facts.start = shift.to_new(facts.start)?;
        facts.end = shift.to_new(facts.end)?;
        (facts.end <= text.len()
            && text.is_char_boundary(facts.start)
            && text.is_char_boundary(facts.end))
        .then_some(facts)
    }
}

/// Whether `byte` is in a note: the name after an `@`, or what stands in the parentheses that
/// follow one (`@json(default)`).
fn in_note(text: &str, byte: usize) -> bool {
    let line_start = text[..byte].rfind('\n').map_or(0, |i| i + 1);
    let before = &text[line_start..byte.min(text.len())];
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let name_start = before.trim_end_matches(word).len();

    if before[..name_start].ends_with('@') {
        return true;
    }

    // Inside `@name( ... )`: the nearest unclosed `(` is preceded by `@name`.
    let mut depth = 0usize;

    for (at, c) in before.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' if depth > 0 => depth -= 1,
            '(' => {
                let head = before[..at].trim_end_matches(word);

                return head.ends_with('@');
            }
            ';' => return false,
            _ => {}
        }
    }

    false
}

/// The label of a source with no file, which prints as `<label>`.
fn generated_label(path: &str) -> Option<&str> {
    path.strip_prefix('<')?.strip_suffix('>')
}
