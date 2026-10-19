//! References, renames, highlights and type definitions.
use super::RenameEdits;
use crate::analysis::TokenKind;
use crate::session::Session;
use crate::{DocumentUri, Error, Location, Position, Range, TextEdit};

impl Session {
    // Navigation

    /// Every use of what the name at `position` names, in the files the check recorded.
    pub fn references(
        &self,
        uri: &DocumentUri,
        position: Position,
        declaration: bool,
    ) -> Result<Vec<Location>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let found = self
            .checked(uri, |a, f| {
                a.compiler
                    .ide_references(f, byte as u32)
                    .into_iter()
                    .filter(|(_, decl)| declaration || !decl)
                    .filter_map(|(span, _)| a.location(span))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(found
            .into_iter()
            .filter_map(|(path, text, start, end)| self.location_in(&path, &text, start, end))
            .collect())
    }

    /// The name at `position`, when it can be renamed: the check recorded what it names (a
    /// struct field or enum member only when its declaration is in the project's files).
    pub fn prepare_rename(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<Range>, Error> {
        let doc = self.document(uri)?;
        let Some((_, token)) = self.word(uri, position)? else {
            return Ok(None);
        };
        if token.kind != TokenKind::Ident {
            return Ok(None);
        }
        let range = doc.index.range(&doc.text, token.span)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let outside = self
            .checked(uri, |a, f| a.compiler.ide_member_declared(f, byte as u32))
            .flatten()
            == Some(false);
        if outside {
            return Ok(None);
        }
        // A name nothing in the checked files declares (a built-in type) has no declaration to
        // rewrite, so renaming its uses would break them. A declaration spelled otherwise is
        // another name for the same thing (`T` bound to `int`), not this one's.
        let word = {
            let start = doc.index.byte(&doc.text, range.start)?;
            let end = doc.index.byte(&doc.text, range.end)?;
            doc.text[start..end].to_string()
        };
        let declared = self
            .checked(uri, |a, f| {
                a.compiler
                    .ide_references(f, byte as u32)
                    .into_iter()
                    .filter(|&(_, decl)| decl)
                    .filter_map(|(span, _)| a.location(span))
                    .any(|(_, text, start, end)| text.get(start..end) == Some(word.as_str()))
            })
            .unwrap_or(false);
        if !declared {
            return Ok(None);
        }
        let found = self.references(uri, position, true)?;
        Ok(found
            .iter()
            .any(|l| l.uri == uri.as_str() && l.range == range)
            .then_some(range))
    }

    /// Edits renaming what the name at `position` names, in every file the check recorded,
    /// grouped by document; `None` when it cannot be renamed.
    pub fn rename(
        &self,
        uri: &DocumentUri,
        position: Position,
        new_name: &str,
    ) -> Result<Option<RenameEdits>, Error> {
        let valid = new_name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
            && new_name.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !valid || jaic::lexicon::is_reserved(new_name) {
            return Err(Error::InvalidEdit("the new name is not an identifier"));
        }
        let Some(range) = self.prepare_rename(uri, position)? else {
            return Ok(None);
        };
        let doc = self.document(uri)?;
        let old = {
            let start = doc.index.byte(&doc.text, range.start)?;
            let end = doc.index.byte(&doc.text, range.end)?;
            doc.text[start..end].to_string()
        };
        let mut edits: Vec<(String, Vec<TextEdit>)> = Vec::new();
        for location in self.references(uri, position, true)? {
            // In an open document, edit only spans that spell the old name.
            if let Ok(target) = DocumentUri::parse(&location.uri)
                && let Some(open) = self.documents.get(&target)
            {
                let start = open.index.byte(&open.text, location.range.start)?;
                let end = open.index.byte(&open.text, location.range.end)?;
                if open.text.get(start..end) != Some(old.as_str()) {
                    continue;
                }
            }
            let edit = TextEdit {
                range: location.range,
                new_text: new_name.into(),
            };
            match edits.iter_mut().find(|(u, _)| *u == location.uri) {
                Some((_, list)) => list.push(edit),
                None => edits.push((location.uri, vec![edit])),
            }
        }
        Ok(Some(edits))
    }

    /// References within the document itself.
    pub fn document_highlights(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Range>, Error> {
        Ok(self
            .references(uri, position, true)?
            .into_iter()
            .filter(|l| l.uri == uri.as_str())
            .map(|l| l.range)
            .collect())
    }

    /// Where the type of the expression at `position` is declared.
    pub fn type_definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let found = self
            .checked(uri, |a, f| {
                a.compiler
                    .ide_type_definition(f, byte as u32)
                    .into_iter()
                    .filter_map(|span| a.location(span))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(found
            .into_iter()
            .filter_map(|(path, text, start, end)| {
                // Narrow `Name :: struct {...}` to the name when the span starts there.
                let line_start = jaic::source::line_start(&text, start);
                let before = &text[line_start..start];
                let name = before.split("::").next().unwrap_or("").trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let at = line_start
                        + before
                            .find(name)
                            .expect("invariant: `name` is a piece of `before`");
                    return self.location_in(&path, &text, at, at + name.len());
                }
                self.location_in(&path, &text, start, end.min(start + 1).max(start))
            })
            .collect())
    }
}
