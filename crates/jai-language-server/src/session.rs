use crate::analysis::{KEYWORDS, Span, Token, TokenKind};
use crate::semantic::{self, Environment};
use crate::{
    CompletionItem, CompletionKind, CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity,
    DocumentSymbol, DocumentUri, Error, Hover, Limits, Location, MarkupContent, Position,
    SemanticToken, SemanticTokenKind, SymbolKind, TextChange, VirtualSources,
    analysis::{Analysis, SymbolRow},
    position::LineIndex,
};
use jaic::intern::Sym;
use jaic::sema::ide::{IdeKind, IdeName};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

struct Document {
    version: i32,
    text: String,
    index: LineIndex,
}
pub struct Session {
    limits: Limits,
    documents: BTreeMap<DocumentUri, Document>,
    analyses: BTreeMap<DocumentUri, Analysis>,
    /// Last analysis of each document that parsed: completion keeps offering its names while
    /// the text being typed does not parse.
    parsed: BTreeMap<DocumentUri, Analysis>,
    /// Type-checked answers (absent: syntax only).
    environment: Option<Environment>,
    semantic: RefCell<semantic::Cache>,
}
impl Session {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            documents: BTreeMap::new(),
            analyses: BTreeMap::new(),
            parsed: BTreeMap::new(),
            environment: None,
            semantic: RefCell::default(),
        }
    }
    /// A session that also type-checks the open documents against `environment`'s modules.
    pub fn with_environment(limits: Limits, environment: Environment) -> Self {
        Self {
            environment: Some(environment),
            ..Self::new(limits)
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
        for (uri, analysis) in &analyses {
            if analysis.complete {
                self.parsed.insert(uri.clone(), analysis.clone());
            }
        }
        self.parsed
            .retain(|uri, _| self.documents.contains_key(uri));
        self.analyses = analyses;
    }
    /// The document a check of `uri` starts from: an open document that `#load`s it (directly or
    /// not) and is loaded by none, else `uri` itself.
    fn root(&self, uri: &DocumentUri) -> DocumentUri {
        let loaded: BTreeSet<&DocumentUri> = self
            .analyses
            .values()
            .flat_map(|a| a.loads.iter().map(|(target, _)| target))
            .collect();
        if !loaded.contains(uri) {
            return uri.clone();
        }
        self.documents
            .keys()
            .find(|d| !loaded.contains(d) && self.reachable(d).contains(&uri))
            .unwrap_or(uri)
            .clone()
    }
    /// Run `query` on the type-checked program containing `uri`, its text replaced by `text`.
    fn with_semantic<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        let environment = self.environment.as_ref()?;
        let files: BTreeMap<PathBuf, Rc<[u8]>> = self
            .documents
            .iter()
            .map(|(u, d)| {
                let text = if u == uri {
                    text
                } else {
                    d.text.as_str()
                };
                (PathBuf::from(u.path()), Rc::from(text.as_bytes()))
            })
            .collect();
        let root = PathBuf::from(self.root(uri).path());
        let mut cache = self.semantic.borrow_mut();
        let analysis = cache.analyze(environment, &root, files);
        query(analysis, Path::new(uri.path()))
    }
    fn source_detail<'a>(&'a self, uri: &DocumentUri, row: &SymbolRow) -> &'a str {
        let text = &self.documents[uri].text;
        let span = row.location;
        // Rows of an older parse may point past the current text.
        let start = text.floor_char_boundary(span.start.min(text.len()));
        let end =
            text.floor_char_boundary(span.end.min(span.start.saturating_add(256)).min(text.len()));
        text[start..end.max(start)].trim()
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
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some((start, end, value)) = self.semantic_hover(uri, &doc.text, byte) {
            return Ok(Some(Hover {
                contents: MarkupContent {
                    value,
                },
                range: doc.index.range(
                    &doc.text,
                    Span {
                        start,
                        end,
                    },
                )?,
            }));
        }
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
    /// Hover from the type checker: the text as typed if it parses, else with the cursor's line
    /// blanked (offsets elsewhere stay the same).
    fn semantic_hover(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<(usize, usize, String)> {
        let probe = repair(text, None)?;
        let (start, end, value) = self.with_semantic(uri, &probe, |a, path| a.hover(path, byte))?;
        (end <= text.len() && text.is_char_boundary(start) && text.is_char_boundary(end))
            .then_some((start, end, value))
    }
    /// Completion from the type checker. The word being typed (and `a.b.` before it) is cut
    /// out, so the probe text is the same while a word is typed and its compile is reused.
    fn semantic_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<(String, Vec<IdeName>)> {
        let bytes = text.as_bytes();
        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80;
        let mut start = byte;
        while start > 0 && ident(bytes[start - 1]) {
            start -= 1;
        }
        let prefix = text[start..byte].to_owned();
        let mut chain = Vec::new();
        let mut at = start;
        while at > 0 && bytes[at - 1] == b'.' {
            let end = at - 1;
            let mut s = end;
            while s > 0 && ident(bytes[s - 1]) {
                s -= 1;
            }
            if s == end || bytes[s].is_ascii_digit() {
                // `.Member` with an inferred type, `f().x`, a number: nothing to offer.
                return Some((prefix, Vec::new()));
            }
            chain.push(&text[s..end]);
            at = s;
        }
        chain.reverse();
        let probe = repair(&format!("{}{}", &text[..at], &text[byte..]), Some(at))?;
        let names = self.with_semantic(uri, &probe, |a, path| a.complete(path, at, &chain))?;
        Some((prefix, names))
    }
    pub fn completion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<CompletionList, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some((prefix, names)) = self.semantic_completion(uri, &doc.text, byte) {
            let lower = prefix.to_lowercase();
            let mut items = BTreeMap::new();
            let mut incomplete = false;
            let keywords = KEYWORDS.iter().map(|k| IdeName {
                name: (*k).into(),
                kind: IdeKind::Constant,
                detail: "Jai keyword".into(),
            });
            let member = doc.text[..byte - prefix.len()].ends_with('.');
            let all = names.into_iter().map(|n| (n, false));
            let all: Vec<(IdeName, bool)> = if member {
                all.collect()
            } else {
                all.chain(keywords.map(|k| (k, true))).collect()
            };
            for (name, keyword) in all {
                if !name.name.to_lowercase().starts_with(&lower) {
                    continue;
                }
                if items.len() >= self.limits.symbols {
                    incomplete = true;
                    break;
                }
                let kind = match name.kind {
                    _ if keyword => CompletionKind::Keyword,
                    IdeKind::Variable => CompletionKind::Variable,
                    IdeKind::Constant => CompletionKind::Constant,
                    IdeKind::Function => CompletionKind::Function,
                    IdeKind::Type => CompletionKind::Struct,
                    IdeKind::Module => CompletionKind::Module,
                    IdeKind::Field => CompletionKind::Field,
                    IdeKind::EnumMember => CompletionKind::EnumMember,
                };
                items.entry(name.name.clone()).or_insert(CompletionItem {
                    label: name.name,
                    kind,
                    detail: name.detail,
                });
            }
            return Ok(CompletionList {
                is_incomplete: incomplete,
                items: items.into_values().collect(),
            });
        }
        let analysis = match self.analyses.get(uri) {
            Some(a) if a.complete => a,
            _ => self.parsed.get(uri).unwrap_or(&self.analyses[uri]),
        };
        let prefix = self
            .word(uri, position)?
            .filter(|(_, token)| token.kind == TokenKind::Ident)
            .map_or("", |(_, token)| {
                &doc.text[token.span.start..byte.min(token.span.end)]
            });
        if self.word(uri, position)?.is_some_and(|(at, _)| {
            at > 0 && self.analyses[uri].tokens[at - 1].kind == TokenKind::Dot
        }) {
            return Ok(CompletionList {
                is_incomplete: true,
                items: vec![],
            });
        }
        let mut items = BTreeMap::new();
        let mut incomplete = !analysis.complete;
        for candidate in self.reachable(uri) {
            let rows = if candidate == uri {
                &analysis.rows
            } else {
                &self.analyses[candidate].rows
            };
            for row in rows {
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
/// `text` made to parse by blanking the lines the parser stops at (an edit in progress): first
/// the line at `first` (the cursor's, when completing), then each reported line, or the one
/// before it when that is empty (a missing `;` is reported on the next line). Byte offsets are
/// kept.
fn repair(text: &str, first: Option<usize>) -> Option<String> {
    let mut text = text.to_owned();
    for attempt in 0..6 {
        let error = match semantic::parse_error(&text) {
            None => return Some(text),
            Some(error) => error,
        };
        let mut at = error.min(text.len());
        if let (0, Some(first)) = (attempt, first) {
            at = first;
        } else {
            let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
            let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
            if text[start..end].trim().is_empty() || text[start..end].trim() == "}" {
                // Reported where the statement would have ended: blank the last non-empty line.
                let before = text[..start].trim_end();
                if before.is_empty() {
                    return None;
                }
                at = before.len();
            }
        }
        text = blank_line(&text, at);
    }
    semantic::parse_error(&text).is_none().then_some(text)
}
/// `text` with the line holding `byte` replaced by spaces (byte offsets are kept).
fn blank_line(text: &str, byte: usize) -> String {
    let start = text[..byte].rfind('\n').map_or(0, |i| i + 1);
    let end = text[byte..].find('\n').map_or(text.len(), |i| byte + i);
    format!(
        "{}{}{}",
        &text[..start],
        " ".repeat(end - start),
        &text[end..]
    )
}
fn contains(span: Span, byte: usize) -> bool {
    span.start <= byte && byte <= span.end
}
