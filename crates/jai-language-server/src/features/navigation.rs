//! References, renames, highlights and type definitions.
use super::RenameEdits;
use crate::analysis::TokenKind;
use crate::session::Session;
use crate::{DocumentUri, Error, Location, Position, Range, TextEdit};

/// A single accepted rename selection; locations and spelling own no compiler identities.
struct RenamePlan {
    range: Range,
    old_name: String,
    locations: Vec<Location>,
}

impl Session {
    // Navigation

    /// Every use of what the name at `position` names, in the files the check recorded.
    pub fn references(
        &self,
        uri: &DocumentUri,
        position: Position,
        declaration: bool,
    ) -> Result<Vec<Location>, Error> {
        let _query = self.discovery_query();
        let fs = self.config_filesystem();
        let (uri, doc) = if let Some(fs) = &fs {
            self.projected_document_entry(fs, uri)?
        } else {
            self.document_entry(uri)?
        };
        if let Some(answer) = self.current_references(uri, position, declaration) {
            return Ok(answer.unwrap_or_default());
        }
        let byte = doc.index.byte(&doc.text, position);
        let query = |a: &mut crate::semantic::Analysis, file| {
            let fs = self
                .config_filesystem()
                .expect("semantic provider owns filesystem");
            a.reference_index()
                .references(file, byte as u32)
                .into_iter()
                .filter(|(_, decl)| declaration || !decl)
                .filter_map(|(span, _)| a.location(span))
                .filter_map(|(path, text, start, end)| {
                    self.projected_location(&fs, &path, &text, start, end)
                })
                .collect::<Vec<_>>()
        };
        // An exploratory query may have enriched the kept compiler since publication. Only
        // after it is gone do independently retained facts become the exact-answer fallback.
        let found = match self.with_cached_semantic(uri, &doc.text, |a, path| {
            let file = a.file(path)?;
            Some(query(a, file))
        }) {
            // Selection already began a transaction. Its unavailable result is terminal;
            // only a cache miss may choose a retained record or start a first fresh check.
            Some(answer) => answer.unwrap_or_default(),
            None => self
                .retained_references(uri, position, declaration)
                .or_else(|| self.checked(uri, query))
                .unwrap_or_default(),
        };
        Ok(found)
    }

    /// The ranges that are edited together when the name at `position` is: a local variable,
    /// parameter or loop name and every use of it, which the check recorded. Nothing else is
    /// linked (a name with other declarations in scope, a member, a global), and nothing without
    /// a compile of the text, so typing in a name only links it once the check has caught up.
    pub fn linked_editing_ranges(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Range>, Error> {
        self.projected_document_query(uri, |key| self.linked_editing_ranges_at_key(key, position))
    }

    fn linked_editing_ranges_at_key(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Range>, Error> {
        let (key, doc) = self.document_entry(uri)?;
        let fs = self.config_filesystem();
        let rows = match &fs {
            Some(fs) => self.projected_bindings(fs, key, position)?,
            None => self.bindings(key, position)?,
        };
        let [(target, row)] = rows[..] else {
            return Ok(Vec::new());
        };
        if !row.local || target != key {
            return Ok(Vec::new());
        }
        let name = row.name.as_str();
        let found = self.references(key, position, true)?;
        let mut ranges = Vec::with_capacity(found.len());
        for location in &found {
            if location.uri != key.as_str() || location.range.start.line != location.range.end.line
            {
                return Ok(Vec::new());
            }
            let start = doc.index.byte(&doc.text, location.range.start);
            let end = doc.index.byte(&doc.text, location.range.end);
            if doc.text.get(start..end) != Some(name) {
                return Ok(Vec::new());
            }
            ranges.push(location.range);
        }
        let on_name = ranges
            .iter()
            .any(|r| r.start <= position && position <= r.end);
        Ok(if on_name {
            ranges
        } else {
            Vec::new()
        })
    }

    /// The name at `position`, when it can be renamed: the check recorded what it names (a
    /// struct field or enum member only when its declaration is in the project's files).
    pub fn prepare_rename(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<Range>, Error> {
        self.projected_document_query(uri, |key| self.prepare_rename_at_key(key, position))
    }

    fn prepare_rename_at_key(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<Range>, Error> {
        Ok(self
            .rename_plan_at_key(uri, position)?
            .map(|plan| plan.range))
    }

    /// Select declarations and project all uses inside one exact checked transaction.
    /// Preparation uses the range; applying a rename consumes these same owned locations.
    fn rename_plan_at_key(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<RenamePlan>, Error> {
        let _query = self.discovery_query();
        // Rename needs an exact check even when ordinary navigation defers compilation.
        let _background = self.background();
        let doc = self.document(uri)?;
        let Some((_, token)) = self.word(uri, position)? else {
            return Ok(None);
        };
        if token.kind != TokenKind::Ident {
            return Ok(None);
        }
        let range = doc.index.range(&doc.text, token.span)?;
        let byte = doc.index.byte(&doc.text, position);
        let start = doc.index.byte(&doc.text, range.start);
        let end = doc.index.byte(&doc.text, range.end);
        let old_name = doc.text[start..end].to_string();
        Ok(self
            .checked(uri, |analysis, file| {
                if analysis.compiler.ide_member_declared(file, byte as u32) == Some(false) {
                    return None;
                }
                let fs = self
                    .config_filesystem()
                    .expect("checked provider owns filesystem");
                // The member lookup can resolve facts, so acquire the index afterward.
                let references = analysis.reference_index().references(file, byte as u32);
                let mut declared = false;
                let mut locations = Vec::new();
                for (span, declaration) in references {
                    let Some((path, text, start, end)) = analysis.location(span) else {
                        continue;
                    };
                    // Builtins have no matching declaration; aliases spelling a different name
                    // must not authorize edits of this spelling.
                    declared |= declaration && text.get(start..end) == Some(old_name.as_str());
                    if let Some(location) = self.projected_location(&fs, &path, &text, start, end) {
                        locations.push(location);
                    }
                }
                if !declared
                    || !locations
                        .iter()
                        .any(|location| location.uri == uri.as_str() && location.range == range)
                {
                    return None;
                }
                Some(RenamePlan {
                    range,
                    old_name,
                    locations,
                })
            })
            .flatten())
    }

    /// Edits renaming what the name at `position` names, in every file the check recorded,
    /// grouped by document; `None` when it cannot be renamed.
    pub fn rename(
        &self,
        uri: &DocumentUri,
        position: Position,
        new_name: &str,
    ) -> Result<Option<RenameEdits>, Error> {
        self.projected_document_query(uri, |key| self.rename_at_key(key, position, new_name))
    }

    fn rename_at_key(
        &self,
        uri: &DocumentUri,
        position: Position,
        new_name: &str,
    ) -> Result<Option<RenameEdits>, Error> {
        let _query = self.discovery_query();
        let _background = self.background();
        if !jaic::lexicon::is_identifier(new_name) || jaic::lexicon::is_reserved(new_name) {
            return Err(Error::InvalidEdit("the new name is not an identifier"));
        }
        let Some(plan) = self.rename_plan_at_key(uri, position)? else {
            return Ok(None);
        };
        let mut edits: Vec<(String, Vec<TextEdit>)> = Vec::new();
        for location in plan.locations {
            // In an open document, edit only spans that spell the old name.
            if let Ok(target) = DocumentUri::parse(&location.uri)
                && let Some(open) = self.documents.get(&target)
            {
                let start = open.index.byte(&open.text, location.range.start);
                let end = open.index.byte(&open.text, location.range.end);
                if open.text.get(start..end) != Some(plan.old_name.as_str()) {
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
        self.projected_document_query(uri, |key| self.document_highlights_at_key(key, position))
    }

    fn document_highlights_at_key(
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
        let _query = self.discovery_query();
        let fs = self.config_filesystem();
        let (uri, doc) = if let Some(fs) = &fs {
            self.projected_document_entry(fs, uri)?
        } else {
            self.document_entry(uri)?
        };
        let byte = doc.index.byte(&doc.text, position);
        Ok(self
            .checked(uri, |a, f| {
                let fs = self
                    .config_filesystem()
                    .expect("semantic provider owns filesystem");
                let found = a
                    .compiler
                    .ide_type_definition(f, byte as u32)
                    .into_iter()
                    .filter_map(|span| a.location(span))
                    .collect::<Vec<_>>();
                found
                    .into_iter()
                    .filter_map(|(path, text, start, end)| {
                        // Narrow `Name :: struct {...}` to the name when the span starts there.
                        let line_start = jaic::source::line_start(&text, start);
                        let before = &text[line_start..start];
                        let name = before.split("::").next().unwrap_or("").trim();
                        if jaic::lexicon::is_identifier(name) {
                            let at =
                                line_start + before.find(name).expect("name is a piece of before");
                            return self.projected_location(&fs, &path, &text, at, at + name.len());
                        }
                        self.projected_location(
                            &fs,
                            &path,
                            &text,
                            start,
                            end.min(start + 1).max(start),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}
