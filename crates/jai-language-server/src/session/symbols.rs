//! Document symbols, reachable documents and name bindings.
use super::{Document, Session, contains, tokens_at};
use crate::analysis::{Token, TokenKind};
use crate::{DocumentSymbol, DocumentUri, Error, Position, analysis::SymbolRow};
use jaic::intern::Sym;
use std::collections::BTreeSet;

impl Session {
    pub(crate) fn source_detail<'a>(doc: &'a Document, row: &SymbolRow) -> &'a str {
        let text = &doc.text;
        let span = row.location;
        // Rows of an older parse may point past the current text.
        let start = text.floor_char_boundary(span.start.min(text.len()));
        let end =
            text.floor_char_boundary(span.end.min(span.start.saturating_add(256)).min(text.len()));
        text[start..end.max(start)].trim()
    }

    pub fn document_symbols(&self, uri: &DocumentUri) -> Result<Vec<DocumentSymbol>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let rows = &self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .rows;
        // Children by parent, so the tree is built in one pass over the rows.
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
        for (id, row) in rows.iter().enumerate() {
            if let Some(parent) = row.parent {
                children[parent].push(id);
            }
        }
        fn tree(
            id: usize,
            rows: &[SymbolRow],
            children: &[Vec<usize>],
            doc: &Document,
        ) -> Result<DocumentSymbol, Error> {
            let row = &rows[id];
            Ok(DocumentSymbol {
                name: row.name.as_str().into(),
                detail: Session::source_detail(doc, row).into(),
                kind: row.kind,
                range: doc.index.range(&doc.text, row.location)?,
                selection_range: doc.index.range(&doc.text, row.selection)?,
                children: children[id]
                    .iter()
                    .map(|&child| tree(child, rows, children, doc))
                    .collect::<Result<_, _>>()?,
            })
        }
        rows.iter()
            .enumerate()
            .filter(|(_, row)| row.parent.is_none())
            .map(|(id, _)| tree(id, rows, &children, doc))
            .collect()
    }

    pub(crate) fn reachable(&self, uri: &DocumentUri) -> Vec<&DocumentUri> {
        let mut pending = vec![uri.clone()];
        let mut seen = BTreeSet::new();
        let mut out = vec![];
        while let Some(next) = pending.pop() {
            let next = self.document_uri(&next).unwrap_or(next);
            if !seen.insert(next.clone()) {
                continue;
            }
            if let Some((key, analysis)) = self.analyses.get_key_value(&next) {
                out.push(key);
                pending.extend(analysis.loads.iter().map(|(uri, _)| uri.clone()));
            }
        }
        out
    }

    pub(crate) fn word(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<(usize, Token)>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position);
        Ok(tokens_at(
            &self
                .analysis_for(uri)
                .expect("opened document has analysis")
                .tokens,
            byte,
        )
        .find(|(_, token)| matches!(token.kind, TokenKind::Ident | TokenKind::Keyword))
        .map(|(at, token)| (at, *token)))
    }

    pub(super) fn bindings(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<(&DocumentUri, &SymbolRow)>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position);
        let Some((at, token)) = self.word(uri, position)? else {
            return Ok(vec![]);
        };
        let analysis = &self
            .analysis_for(uri)
            .expect("opened document has analysis");
        if let Some(row) = analysis
            .rows
            .iter()
            .find(|row| contains(row.selection, byte))
        {
            return Ok(vec![(uri, row)]);
        }
        if at > 0 && analysis.tokens[at - 1].kind == TokenKind::Dot {
            return Ok(vec![]);
        }
        if analysis
            .opaque_scopes
            .iter()
            .any(|span| contains(*span, byte))
        {
            return Ok(vec![]);
        }
        let name = Sym::intern(token.spelling(&doc.text));
        let mut locals = analysis
            .rows
            .iter()
            .filter(|row| {
                row.local
                    && row.name == name
                    && contains(row.scope, byte)
                    && row.selection.start <= byte
            })
            .collect::<Vec<_>>();
        locals.sort_by_key(|row| {
            (
                row.scope.end - row.scope.start,
                std::cmp::Reverse(row.selection.start),
            )
        });
        if let Some(row) = locals.first() {
            return Ok(vec![(uri, *row)]);
        }
        let mut out = vec![];
        for candidate in self.reachable(uri) {
            for row in &self.analyses[candidate].rows {
                if row.parent.is_none()
                    && !row.local
                    && row.name == name
                    && (candidate == uri || !row.file_private)
                {
                    out.push((candidate, row));
                }
            }
        }
        Ok(out)
    }
}
