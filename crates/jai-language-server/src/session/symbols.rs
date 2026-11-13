//! Document symbols, reachable documents and name bindings.
use super::{Document, Session, contains, tokens_at};
use crate::analysis::{Token, TokenKind};
use crate::{DocumentSymbol, DocumentUri, Error, Position, analysis::SymbolRow};
use jaic::intern::Sym;
use std::collections::BTreeSet;

/// How much of a declaration its detail and its source-only hover quote, in written bytes
/// (whitespace not counted).
const MAX_DETAIL: usize = 256;

impl Session {
    /// The start of the declaration a row stands for, as its outline detail and its hover when
    /// the type checker has nothing to say: `answer :: () -> int {return 42;}`.
    pub(crate) fn source_detail<'a>(doc: &'a Document, row: &SymbolRow) -> &'a str {
        let text = &doc.text;
        let span = row.location;
        // Rows of an older parse may point past the current text.
        let start = text.floor_char_boundary(span.start.min(text.len()));
        let end = text
            .floor_char_boundary(span.end.min(text.len()))
            .max(start);
        let declaration = &text[start..end];
        // A cut by length counts only what is written: blank lines and indentation would
        // otherwise move it, and the same declaration would be quoted to different lengths.
        let mut written = 0;
        for (at, c) in declaration.char_indices() {
            if !c.is_whitespace() {
                written += c.len_utf8();
                if written > MAX_DETAIL {
                    return declaration[..at].trim();
                }
            }
        }
        declaration.trim()
    }

    /// The outline: declarations nested under the scope sections, `#if`/`#else` blocks and
    /// other directives that cover them, each declaration with its fields, members and locals.
    pub fn document_symbols(&self, uri: &DocumentUri) -> Result<Vec<DocumentSymbol>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let analysis = self
            .analysis_for(uri)
            .expect("opened document has analysis");
        let rows = &analysis.rows;
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
        // The sections and the declarations that have no parent, by where they start (a
        // section before what it holds), so a stack of the sections still open nests them.
        enum Item {
            Section(usize),
            Row(usize),
        }
        let mut items: Vec<(usize, std::cmp::Reverse<usize>, Item)> = Vec::new();
        for (at, section) in analysis.outline.iter().enumerate() {
            items.push((
                section.location.start,
                std::cmp::Reverse(section.location.end),
                Item::Section(at),
            ));
        }
        for (id, row) in rows.iter().enumerate() {
            if row.parent.is_none() {
                items.push((
                    row.location.start,
                    std::cmp::Reverse(row.location.end),
                    Item::Row(id),
                ));
            }
        }
        items.sort_by_key(|(start, end, item)| (*start, *end, matches!(item, Item::Row(_))));
        let mut roots: Vec<DocumentSymbol> = Vec::new();
        let mut open: Vec<(usize, DocumentSymbol)> = Vec::new();
        fn close(open: &mut Vec<(usize, DocumentSymbol)>, roots: &mut Vec<DocumentSymbol>) {
            if let Some((_, done)) = open.pop() {
                match open.last_mut() {
                    Some((_, parent)) => parent.children.push(done),
                    None => roots.push(done),
                }
            }
        }
        for (start, _, item) in items {
            while open.last().is_some_and(|(end, _)| *end <= start) {
                close(&mut open, &mut roots);
            }
            match item {
                Item::Section(at) => {
                    let section = &analysis.outline[at];
                    open.push((
                        section.location.end,
                        DocumentSymbol {
                            name: section.name.clone(),
                            detail: section.detail.clone(),
                            kind: section.kind,
                            range: doc.index.range(&doc.text, section.location)?,
                            selection_range: doc.index.range(&doc.text, section.selection)?,
                            children: Vec::new(),
                        },
                    ));
                }
                Item::Row(id) => {
                    let symbol = tree(id, rows, &children, doc)?;
                    match open.last_mut() {
                        Some((_, parent)) => parent.children.push(symbol),
                        None => roots.push(symbol),
                    }
                }
            }
        }
        while !open.is_empty() {
            close(&mut open, &mut roots);
        }
        Ok(roots)
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
        let found = tokens_at(
            &self
                .analysis_for(uri)
                .expect("opened document has analysis")
                .tokens,
            byte,
        )
        .find(|(_, token)| matches!(token.kind, TokenKind::Ident | TokenKind::Keyword))
        .map(|(at, token)| (at, *token));
        // A name in an `@Interpolates` hole is a word of the string's text.
        Ok(found.or_else(|| self.hole_word(uri, byte)))
    }

    pub(crate) fn bindings(
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
