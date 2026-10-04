use crate::analysis::{KEYWORDS, Span, Token, TokenKind};
use crate::{
    CompletionItem, CompletionKind, CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity,
    DocumentSymbol, DocumentUri, Error, Hover, Limits, Location, MarkupContent, Position,
    SemanticToken, SemanticTokenKind, SymbolKind, TextChange, VirtualSources,
    analysis::{Analysis, SymbolRow},
    position::LineIndex,
};
use jaic::intern::Sym;
use std::collections::{BTreeMap, BTreeSet};

struct Document {
    version: i32,
    text: String,
    index: LineIndex,
}
pub struct Session {
    limits: Limits,
    documents: BTreeMap<DocumentUri, Document>,
    analyses: BTreeMap<DocumentUri, Analysis>,
}
impl Session {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            documents: BTreeMap::new(),
            analyses: BTreeMap::new(),
        }
    }
    pub fn limits(&self) -> Limits {
        self.limits
    }
    pub fn source_snapshot(&self) -> VirtualSources {
        VirtualSources {
            files: self
                .documents
                .iter()
                .map(|(uri, doc)| (uri.path().to_owned(), doc.text.clone()))
                .collect(),
        }
    }
    pub fn document_text(&self, uri: &DocumentUri) -> Result<&str, Error> {
        Ok(self.document(uri)?.text.as_str())
    }
    pub fn version(&self, uri: &DocumentUri) -> Result<i32, Error> {
        Ok(self.document(uri)?.version)
    }
    pub fn open(&mut self, uri: DocumentUri, version: i32, text: String) -> Result<(), Error> {
        if self.documents.contains_key(&uri) {
            return Err(Error::AlreadyOpen);
        }
        if self.documents.len() >= self.limits.documents {
            return Err(Error::Limit("open document count exceeded"));
        }
        self.admit(&uri, text.len(), 0)?;
        self.documents.insert(
            uri,
            Document {
                version,
                index: LineIndex::new(&text),
                text,
            },
        );
        self.rebuild();
        Ok(())
    }
    pub fn change(
        &mut self,
        uri: &DocumentUri,
        version: i32,
        changes: &[TextChange],
    ) -> Result<(), Error> {
        let document = self.document(uri)?;
        if version <= document.version {
            return Err(Error::StaleVersion);
        }
        if changes.len() > self.limits.edits {
            return Err(Error::Limit("incremental edit count exceeded"));
        }
        let old_bytes = document.text.len() + uri.as_str().len();
        let mut text = document.text.to_owned();
        for change in changes {
            let index = LineIndex::new(&text);
            let (start, end) = if let Some(range) = change.range {
                if range.start > range.end {
                    return Err(Error::InvalidEdit("edit range is reversed"));
                }
                let start = index.byte(&text, range.start)?;
                let end = index.byte(&text, range.end)?;
                if let Some(length) = change.range_length {
                    if text[start..end].encode_utf16().count() != length as usize {
                        return Err(Error::InvalidEdit(
                            "rangeLength disagrees with UTF-16 range",
                        ));
                    }
                }
                (start, end)
            } else {
                if change.range_length.is_some() {
                    return Err(Error::InvalidEdit(
                        "full replacement cannot have rangeLength",
                    ));
                }
                (0, text.len())
            };
            let bytes = text
                .len()
                .checked_sub(end - start)
                .and_then(|n| n.checked_add(change.text.len()))
                .ok_or(Error::Limit("edit size overflow"))?;
            self.admit(uri, bytes, old_bytes)?;
            text.replace_range(start..end, &change.text);
        }
        let current = self.documents.get_mut(uri).expect("checked document");
        *current = Document {
            version,
            index: LineIndex::new(&text),
            text,
        };
        self.rebuild();
        Ok(())
    }
    pub fn close(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        self.documents.remove(uri).ok_or(Error::MissingDocument)?;
        self.rebuild();
        Ok(())
    }
    pub fn publications(&self) -> Vec<(String, i32, Vec<Diagnostic>)> {
        self.documents
            .iter()
            .map(|(uri, doc)| {
                (
                    uri.as_str().into(),
                    doc.version,
                    self.analyses[uri].diagnostics.clone(),
                )
            })
            .collect()
    }
    pub fn diagnostics(&self, uri: &DocumentUri) -> Result<&[Diagnostic], Error> {
        self.document(uri)?;
        Ok(&self.analyses[uri].diagnostics)
    }
    fn admit(&self, uri: &DocumentUri, bytes: usize, old: usize) -> Result<(), Error> {
        if bytes > self.limits.document_bytes {
            return Err(Error::Limit("document byte budget exceeded"));
        }
        let current = self
            .documents
            .iter()
            .try_fold(0usize, |n, (name, doc)| {
                n.checked_add(name.as_str().len() + doc.text.len())
            })
            .ok_or(Error::Limit("workspace byte overflow"))?;
        let total = current
            .checked_sub(old)
            .and_then(|n| n.checked_add(uri.as_str().len() + bytes))
            .ok_or(Error::Limit("workspace byte overflow"))?;
        if total > self.limits.workspace_bytes {
            return Err(Error::Limit("workspace byte budget exceeded"));
        }
        Ok(())
    }
    fn document(&self, uri: &DocumentUri) -> Result<&Document, Error> {
        self.documents.get(uri).ok_or(Error::MissingDocument)
    }
    fn rebuild(&mut self) {
        let mut analyses = BTreeMap::new();
        for (uri, document) in &self.documents {
            analyses.insert(
                uri.clone(),
                Analysis::build(&document.text, uri, self.limits),
            );
        }
        for (uri, analysis) in &mut analyses {
            let document = &self.documents[uri];
            for (target, span) in analysis.loads.clone() {
                if !self.documents.contains_key(&target) {
                    analysis.diagnostic(&document.index,&document.text,span,DiagnosticSeverity::Warning,DiagnosticCode::Source,"The #load file is not open in this session; no filesystem fallback is permitted.");
                }
            }
        }
        self.analyses = analyses;
    }
    fn source_detail<'a>(&'a self, uri: &DocumentUri, row: &SymbolRow) -> &'a str {
        let text = &self.documents[uri].text;
        let span = row.location;
        let end =
            text.floor_char_boundary(span.end.min(span.start.saturating_add(256)).min(text.len()));
        text[span.start..end.max(span.start)].trim()
    }
    pub fn document_symbols(&self, uri: &DocumentUri) -> Result<Vec<DocumentSymbol>, Error> {
        let doc = self.document(uri)?;
        let rows = &self.analyses[uri].rows;
        fn tree(
            id: usize,
            rows: &[SymbolRow],
            doc: &Document,
            session: &Session,
            uri: &DocumentUri,
        ) -> Result<DocumentSymbol, Error> {
            let row = &rows[id];
            Ok(DocumentSymbol {
                name: row.name.as_str().into(),
                detail: session.source_detail(uri, row).into(),
                kind: row.kind,
                range: doc.index.range(&doc.text, row.location)?,
                selection_range: doc.index.range(&doc.text, row.selection)?,
                children: rows
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| row.parent == Some(id))
                    .map(|(child, _)| tree(child, rows, doc, session, uri))
                    .collect::<Result<_, _>>()?,
            })
        }
        rows.iter()
            .enumerate()
            .filter(|(_, row)| row.parent.is_none())
            .map(|(id, _)| tree(id, rows, doc, self, uri))
            .collect()
    }
    fn reachable(&self, uri: &DocumentUri) -> Vec<&DocumentUri> {
        let mut pending = vec![uri.clone()];
        let mut seen = BTreeSet::new();
        let mut out = vec![];
        while let Some(next) = pending.pop() {
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
    fn word(&self, uri: &DocumentUri, position: Position) -> Result<Option<(usize, Token)>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        Ok(self.analyses[uri]
            .tokens
            .iter()
            .enumerate()
            .find(|(_, token)| {
                token.span.start <= byte
                    && byte <= token.span.end
                    && matches!(token.kind, TokenKind::Ident | TokenKind::Keyword)
            })
            .map(|(at, token)| (at, *token)))
    }
    fn bindings(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<(&DocumentUri, &SymbolRow)>, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let Some((at, token)) = self.word(uri, position)? else {
            return Ok(vec![]);
        };
        let analysis = &self.analyses[uri];
        if let Some(row) = analysis
            .rows
            .iter()
            .find(|row| contains(row.selection, byte))
        {
            return Ok(vec![(
                self.documents.get_key_value(uri).expect("open URI").0,
                row,
            )]);
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
            return Ok(vec![(
                self.documents.get_key_value(uri).expect("open URI").0,
                *row,
            )]);
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
    pub fn definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
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
    pub fn hover(&self, uri: &DocumentUri, position: Position) -> Result<Option<Hover>, Error> {
        let doc = self.document(uri)?;
        let Some((_, token)) = self.word(uri, position)? else {
            return Ok(None);
        };
        let rows = self.bindings(uri, position)?;
        let value = if rows.len() == 1 {
            format!(
                "{}\n\nSource syntax declaration. Type evaluation and compile-time execution are disabled during editing.",
                self.source_detail(rows[0].0, rows[0].1)
            )
        } else if token.kind == TokenKind::Keyword {
            format!("Jai keyword {}", token.spelling(&doc.text))
        } else {
            return Ok(None);
        };
        Ok(Some(Hover {
            contents: MarkupContent {
                value,
            },
            range: doc.index.range(&doc.text, token.span)?,
        }))
    }
    pub fn completion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<CompletionList, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let analysis = &self.analyses[uri];
        let prefix = self
            .word(uri, position)?
            .filter(|(_, token)| token.kind == TokenKind::Ident)
            .map_or("", |(_, token)| {
                &doc.text[token.span.start..byte.min(token.span.end)]
            });
        if self
            .word(uri, position)?
            .is_some_and(|(at, _)| at > 0 && analysis.tokens[at - 1].kind == TokenKind::Dot)
        {
            return Ok(CompletionList {
                is_incomplete: true,
                items: vec![],
            });
        }
        let mut items = BTreeMap::new();
        let mut incomplete = !analysis.complete;
        for candidate in self.reachable(uri) {
            for row in &self.analyses[candidate].rows {
                let visible = if row.local {
                    candidate == uri && contains(row.scope, byte) && row.selection.start <= byte
                } else {
                    row.parent.is_none() && (candidate == uri || !row.file_private)
                };
                let name = row.name.as_str();
                if visible && name.starts_with(prefix) && items.len() < self.limits.symbols {
                    let kind = match row.kind {
                        SymbolKind::Function => CompletionKind::Function,
                        SymbolKind::Struct => CompletionKind::Struct,
                        SymbolKind::Enum => CompletionKind::Enum,
                        SymbolKind::Constant => CompletionKind::Constant,
                        SymbolKind::TypeAlias => CompletionKind::TypeAlias,
                        _ => CompletionKind::Variable,
                    };
                    items.entry(name.to_owned()).or_insert(CompletionItem {
                        label: name.into(),
                        kind,
                        detail: self.source_detail(candidate, row).into(),
                    });
                } else if visible && items.len() >= self.limits.symbols {
                    incomplete = true;
                }
            }
        }
        for name in KEYWORDS {
            if name.starts_with(prefix) && items.len() < self.limits.symbols {
                items.entry((*name).into()).or_insert(CompletionItem {
                    label: (*name).into(),
                    kind: CompletionKind::Keyword,
                    detail: "Jai keyword".into(),
                });
            }
        }
        Ok(CompletionList {
            is_incomplete: incomplete,
            items: items.into_values().collect(),
        })
    }
    pub fn semantic_tokens(&self, uri: &DocumentUri) -> Result<Vec<SemanticToken>, Error> {
        let doc = self.document(uri)?;
        let text = doc.text.as_str();
        let analysis = &self.analyses[uri];
        let mut output = vec![];
        for token in &analysis.tokens {
            let row = analysis.rows.iter().find(|row| row.selection == token.span);
            let ty = match token.kind {
                TokenKind::Keyword => SemanticTokenKind::Keyword,
                TokenKind::String => SemanticTokenKind::String,
                TokenKind::Number => SemanticTokenKind::Number,
                TokenKind::Ident => match row.map(|row| row.kind) {
                    Some(SymbolKind::Function) => SemanticTokenKind::Function,
                    Some(SymbolKind::Struct | SymbolKind::Enum | SymbolKind::TypeAlias) => {
                        SemanticTokenKind::Type
                    }
                    Some(SymbolKind::Property) => SemanticTokenKind::Property,
                    Some(SymbolKind::Variable)
                        if row.is_some_and(|row| row.parent.is_some() && row.local) =>
                    {
                        SemanticTokenKind::Parameter
                    }
                    _ => SemanticTokenKind::Variable,
                },
                TokenKind::Directive => SemanticTokenKind::Macro,
                TokenKind::Dot | TokenKind::Punctuation => SemanticTokenKind::Operator,
            };
            let first = doc.index.position(text, token.span.start)?.line as usize;
            let last = doc.index.position(text, token.span.end)?.line as usize;
            for (_, (start, end)) in doc
                .index
                .lines()
                .enumerate()
                .skip(first)
                .take(last - first + 1)
            {
                let start = start.max(token.span.start);
                let end = end.min(token.span.end);
                if start >= end {
                    continue;
                }
                if output.len() >= self.limits.tokens {
                    return Err(Error::Limit("semantic token line budget exceeded"));
                }
                let position = doc.index.position(text, start)?;
                let length = text[start..end].encode_utf16().count() as u32;
                output.push(SemanticToken {
                    position,
                    length,
                    kind: ty,
                    declaration: row.is_some(),
                    readonly: row.is_some_and(|row| row.readonly),
                });
            }
        }
        Ok(output)
    }
}
fn contains(span: Span, byte: usize) -> bool {
    span.start <= byte && byte <= span.end
}
