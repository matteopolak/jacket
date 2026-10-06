use crate::analysis::{DIRECTIVES, KEYWORDS, Span, Token, TokenKind};
use crate::hover::{Block, HoverText, Inline};
use crate::semantic::{self, Environment};
use crate::{
    CompletionItem, CompletionKind, CompletionList, Diagnostic, DiagnosticCode, DiagnosticSeverity,
    DocumentSymbol, DocumentUri, Error, Hover, Limits, Location, MarkupContent, MarkupKind,
    Position, SemanticToken, SemanticTokenKind, SymbolKind, TextChange, VirtualSources,
    analysis::{Analysis, SymbolRow},
    position::LineIndex,
};
use jaic::intern::Sym;
use jaic::sema::ide::{IdeKind, IdeName};
use jaic::sema::ide_meta::IdeClass;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) struct Document {
    pub(crate) version: i32,
    pub(crate) text: String,
    pub(crate) index: LineIndex,
}

pub struct Session {
    pub(crate) limits: Limits,
    pub(crate) documents: BTreeMap<DocumentUri, Document>,
    pub(crate) analyses: BTreeMap<DocumentUri, Analysis>,
    /// Last analysis of each document that parsed: completion keeps offering its names while
    /// the text being typed does not parse.
    pub(crate) parsed: BTreeMap<DocumentUri, Analysis>,
    /// Type-checked answers (absent: syntax only).
    pub(crate) environment: Option<Environment>,
    pub(crate) semantic: RefCell<semantic::Cache>,
    /// Open `jailint.toml` files: settings for the lints of documents under their directory.
    /// They are not Jai documents, so they get no analysis or diagnostics of their own.
    pub(crate) lint_configs: BTreeMap<DocumentUri, Document>,
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
            lint_configs: BTreeMap::new(),
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
        if self.documents.contains_key(&uri) || self.lint_configs.contains_key(&uri) {
            return Err(Error::AlreadyOpen);
        }
        if self.documents.len() + self.lint_configs.len() >= self.limits.documents {
            return Err(Error::Limit("open document count exceeded"));
        }
        self.admit(&uri, text.len(), 0)?;
        let map = if crate::lints::is_config(&uri) {
            // Lints computed with the old settings are stale.
            self.semantic.borrow_mut().forget_lints();
            &mut self.lint_configs
        } else {
            &mut self.documents
        };
        map.insert(
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
        let document = match self.lint_configs.get(uri) {
            Some(config) => config,
            None => self.document(uri)?,
        };
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
                if let Some(length) = change.range_length
                    && text[start..end].encode_utf16().count() != length as usize
                {
                    return Err(Error::InvalidEdit(
                        "rangeLength disagrees with UTF-16 range",
                    ));
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
        let config = self.lint_configs.contains_key(uri);
        if config {
            self.semantic.borrow_mut().forget_lints();
        }
        let map = if config {
            &mut self.lint_configs
        } else {
            &mut self.documents
        };
        let current = map.get_mut(uri).expect("checked document");
        *current = Document {
            version,
            index: LineIndex::new(&text),
            text,
        };
        self.rebuild();
        Ok(())
    }

    pub fn close(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        if self.lint_configs.remove(uri).is_some() {
            self.semantic.borrow_mut().forget_lints();
            self.rebuild();
            return Ok(());
        }
        self.documents.remove(uri).ok_or(Error::MissingDocument)?;
        self.rebuild();
        Ok(())
    }

    pub fn publications(&self) -> Vec<(String, i32, Vec<Diagnostic>)> {
        self.documents
            .iter()
            .map(|(uri, doc)| {
                let mut diagnostics = self.analyses[uri].diagnostics.clone();
                diagnostics.extend(self.check_diagnostics(uri));
                diagnostics.extend(self.lint_diagnostics(uri));
                (uri.as_str().into(), doc.version, diagnostics)
            })
            .collect()
    }

    /// The type checker's error, when the program `uri` belongs to fails to compile and the
    /// error (or one of its notes) is in `uri`. Only a document that parses is checked; its
    /// syntax error is already reported.
    pub(crate) fn check_diagnostics(&self, uri: &DocumentUri) -> Option<Diagnostic> {
        let doc = self.document(uri).ok()?;
        if semantic::parse_error(&doc.text).is_some() {
            return None;
        }
        let (start, end, message) =
            self.with_semantic(uri, &doc.text, |analysis, path| analysis.check_error(path))?;
        Some(Diagnostic {
            range: self.range_of(uri, Span::new(start, end))?,
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::Check,
            message,
        })
    }

    /// Syntax and source diagnostics of `uri`, then its type error and lints.
    pub fn diagnostics(&self, uri: &DocumentUri) -> Result<Vec<Diagnostic>, Error> {
        self.document(uri)?;
        let mut diagnostics = self.analyses[uri].diagnostics.clone();
        diagnostics.extend(self.check_diagnostics(uri));
        diagnostics.extend(self.lint_diagnostics(uri));
        Ok(diagnostics)
    }

    fn admit(&self, uri: &DocumentUri, bytes: usize, old: usize) -> Result<(), Error> {
        if bytes > self.limits.document_bytes {
            return Err(Error::Limit("document byte budget exceeded"));
        }
        let current = self
            .documents
            .iter()
            .chain(&self.lint_configs)
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

    pub(crate) fn document(&self, uri: &DocumentUri) -> Result<&Document, Error> {
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
                    analysis.diagnostic(
                        &document.index,
                        &document.text,
                        span,
                        DiagnosticSeverity::Warning,
                        DiagnosticCode::Source,
                        "The #load file is not open in this session; \
                         no filesystem fallback is permitted.",
                    );
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
    pub(crate) fn root(&self, uri: &DocumentUri) -> DocumentUri {
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
    pub(crate) fn with_semantic<T>(
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

    pub(crate) fn source_detail<'a>(&'a self, uri: &DocumentUri, row: &SymbolRow) -> &'a str {
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

    pub(crate) fn reachable(&self, uri: &DocumentUri) -> Vec<&DocumentUri> {
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

    pub(crate) fn word(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<(usize, Token)>, Error> {
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
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some(location) = self.link_definition(uri, byte) {
            return Ok(vec![location]);
        }
        if let Some(probe) = repair(&doc.text, None) {
            let found = self
                .with_semantic(uri, &probe, |a, path| Some(a.definition(path, byte)))
                .unwrap_or_default();
            let locations: Vec<Location> = found
                .into_iter()
                .filter_map(|(path, text, start, end)| {
                    let target = DocumentUri::parse(&format!("file://{path}")).ok()?;
                    // An open document's own text and index, else the compiled file's.
                    let range = match self.documents.get(&target) {
                        Some(open) if *open.text == *text => open
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
        if let Some(doc) = self.documents.get(uri) {
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
        let doc = self.document(uri)?;
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
        if let Some((start, end, text)) = self.format_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.directive_hover(uri, byte) {
            return found(start, end, text);
        }
        let Some((_, token)) = self.word(uri, position)? else {
            return match self.expansion_hover(uri, byte) {
                Some((start, end, text)) => found(start, end, text),
                None => Ok(None),
            };
        };
        if let Some((start, end, value)) = self.semantic_hover(uri, &doc.text, byte) {
            let text = self.with_macro_expansion(uri, start, HoverText::code(value));
            return found(start, end, text);
        }
        let rows = self.bindings(uri, position)?;
        let text = if rows.len() == 1 {
            HoverText::new(vec![
                Block::Code(self.source_detail(rows[0].0, rows[0].1).into()),
                Block::Note(vec![Inline::Text(
                    "Source syntax declaration. \
                     Type evaluation and compile-time execution are disabled during editing."
                        .into(),
                )]),
            ])
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

    /// Hover from the type checker: the text as typed if it parses, else with the cursor's line
    /// blanked (offsets elsewhere stay the same).
    pub(crate) fn semantic_hover(
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

    /// Inside the string of `#load "..."` (files and folders relative to the document) or
    /// `#import "..."` (modules on the import path): the entries of the folder typed so far.
    fn path_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<CompletionList> {
        let line = &text[text[..byte].rfind('\n').map_or(0, |n| n + 1)..byte];
        let quote = line.rfind('"')?;
        let before = line[..quote].trim_end();
        // The string must be open: an even number of quotes before it on the line.
        if !line[..quote].matches('"').count().is_multiple_of(2) {
            return None;
        }
        let directive = before
            .rsplit(|c: char| c.is_whitespace() || c == '(')
            .next()?;
        let load = directive.starts_with("#load");
        if !load && !directive.starts_with("#import") {
            return None;
        }
        let typed = &line[quote + 1..];
        let (folder, partial) = typed.rsplit_once('/').unwrap_or(("", typed));
        let here = Path::new(uri.path()).parent().unwrap_or(Path::new("/"));
        let roots: Vec<PathBuf> = if load {
            vec![here.join(folder)]
        } else {
            let options = self
                .environment
                .as_ref()
                .map(|e| (e.options)(Path::new(uri.path())));
            let mut roots: Vec<PathBuf> = options
                .map(|o| o.import_paths)
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.join(folder))
                .collect();
            roots.push(here.join("modules").join(folder));
            roots
        };
        // (name, is_folder): the environment's files and the open documents under each root.
        let mut entries = BTreeSet::new();
        for root in &roots {
            if let Some(environment) = &self.environment {
                entries.extend(environment.fs.list_dir(root));
            }
            for open in self.documents.keys() {
                let Ok(rest) = Path::new(open.path()).strip_prefix(root) else {
                    continue;
                };
                let mut parts = rest.iter();
                let Some(first) = parts.next() else {
                    continue;
                };
                entries.insert((first.to_string_lossy().into_owned(), parts.next().is_some()));
            }
        }
        let items = entries
            .into_iter()
            .filter(|(name, _)| name.starts_with(partial) && !name.starts_with('.'))
            .filter_map(|(name, folder)| {
                let jai = name.strip_suffix(".jai");
                match (folder, jai) {
                    (true, _) => Some(CompletionItem {
                        label: if load {
                            format!("{name}/")
                        } else {
                            name
                        },
                        kind: if load {
                            CompletionKind::Folder
                        } else {
                            CompletionKind::Module
                        },
                        detail: if load {
                            "folder"
                        } else {
                            "module"
                        }
                        .into(),
                    }),
                    (false, Some(stem)) => Some(CompletionItem {
                        label: if load {
                            name.clone()
                        } else {
                            stem.into()
                        },
                        kind: if load {
                            CompletionKind::File
                        } else {
                            CompletionKind::Module
                        },
                        detail: if load {
                            "file"
                        } else {
                            "module"
                        }
                        .into(),
                    }),
                    _ => None,
                }
            })
            // Loading the file being edited would include it twice.
            .filter(|item| !load || here.join(folder).join(&item.label) != Path::new(uri.path()))
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        Some(CompletionList {
            is_incomplete: false,
            items: items
                .into_iter()
                .filter(|i| seen.insert(i.label.clone()))
                .collect(),
        })
    }

    pub fn completion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<CompletionList, Error> {
        let doc = self.document(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        // `#` and the start of a directive: offer directives (labels include the `#`).
        // Past the whole separator: it may be multi-byte (a no-break space).
        let word_start = doc.text[..byte]
            .char_indices()
            .rfind(|&(_, c)| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |(i, c)| i + c.len_utf8());
        if doc.text[..word_start].ends_with('#') {
            let typed = doc.text[word_start..byte].to_lowercase();
            return Ok(CompletionList {
                is_incomplete: false,
                items: DIRECTIVES
                    .iter()
                    .filter(|(name, _)| name.starts_with(&typed))
                    .map(|(name, detail)| CompletionItem {
                        label: format!("#{name}"),
                        kind: CompletionKind::Keyword,
                        detail: (*detail).into(),
                    })
                    .collect(),
            });
        }
        if let Some(list) = self.path_completion(uri, &doc.text, byte) {
            return Ok(list);
        }
        if let Some((prefix, names)) = self.semantic_completion(uri, &doc.text, byte) {
            let lower = prefix.to_lowercase();
            let mut items = BTreeMap::new();
            let mut incomplete = false;
            let keywords = KEYWORDS.iter().map(|k| IdeName {
                name: (*k).into(),
                kind: IdeKind::Constant,
                detail: "keyword".into(),
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
                    detail: "keyword".into(),
                });
            }
        }
        Ok(CompletionList {
            is_incomplete: incomplete,
            items: items.into_values().collect(),
        })
    }

    /// Tokens classified by the syntax layer, refined by the type checker when the session has
    /// an environment: uses of types, procedures, macros, modules, constants and enum members,
    /// `$T` parameters, notes, and the `%` directives of format strings.
    pub fn semantic_tokens(&self, uri: &DocumentUri) -> Result<Vec<SemanticToken>, Error> {
        let doc = self.document(uri)?;
        let text = doc.text.as_str();
        let analysis = &self.analyses[uri];
        let classes: BTreeMap<usize, (usize, IdeClass)> = if self.environment.is_some() {
            self.identifier_classes(uri)
                .into_iter()
                .map(|(span, class)| (span.start, (span.end, class)))
                .collect()
        } else {
            BTreeMap::new()
        };
        // `$T` / `$$T`: the name is a type parameter throughout its procedure.
        let mut poly: Vec<(&str, Span)> = Vec::new();
        for pair in analysis.tokens.windows(2) {
            let (sigil, name) = (pair[0], pair[1]);
            if matches!(sigil.spelling(text), "$" | "$$")
                && name.kind == TokenKind::Ident
                && sigil.span.end == name.span.start
                && let Some(row) = analysis
                    .rows
                    .iter()
                    .filter(|r| {
                        r.kind == SymbolKind::Function && contains(r.location, name.span.start)
                    })
                    .min_by_key(|r| r.location.end - r.location.start)
            {
                poly.push((name.spelling(text), row.location));
            }
        }
        let specs: Vec<Span> = analysis
            .format_calls
            .iter()
            .flat_map(|c| c.specs.iter().map(|s| s.span))
            .collect();
        // (span, kind, declaration, readonly, expand)
        let mut pieces: Vec<(Span, SemanticTokenKind, bool, bool, bool)> = Vec::new();
        for (i, token) in analysis.tokens.iter().enumerate() {
            let row = analysis.rows.iter().find(|row| row.selection == token.span);
            let declaration = row.is_some();
            let readonly = row.is_some_and(|row| row.readonly);
            let piece = |kind| (token.span, kind, declaration, readonly, false);
            match token.kind {
                TokenKind::String => {
                    let mut at = token.span.start;
                    for spec in specs
                        .iter()
                        .filter(|s| token.span.start <= s.start && s.end <= token.span.end)
                    {
                        if spec.start > at {
                            pieces.push((
                                Span::new(at, spec.start),
                                SemanticTokenKind::String,
                                false,
                                false,
                                false,
                            ));
                        }
                        pieces.push((
                            *spec,
                            SemanticTokenKind::FormatSpecifier,
                            false,
                            false,
                            false,
                        ));
                        at = spec.end;
                    }
                    if at < token.span.end {
                        pieces.push((
                            Span::new(at, token.span.end),
                            SemanticTokenKind::String,
                            false,
                            false,
                            false,
                        ));
                    }
                }
                TokenKind::Keyword => pieces.push(piece(SemanticTokenKind::Keyword)),
                TokenKind::Number => pieces.push(piece(SemanticTokenKind::Number)),
                TokenKind::Directive if token.spelling(text).starts_with('@') => {
                    pieces.push(piece(SemanticTokenKind::Decorator))
                }
                TokenKind::Directive => pieces.push(piece(SemanticTokenKind::Macro)),
                TokenKind::Dot | TokenKind::Punctuation => {
                    pieces.push(piece(SemanticTokenKind::Operator))
                }
                TokenKind::Ident => {
                    let spelling = token.spelling(text);
                    if poly.iter().any(|(name, scope)| {
                        *name == spelling && contains(*scope, token.span.start)
                    }) {
                        let sigil = i > 0
                            && matches!(analysis.tokens[i - 1].spelling(text), "$" | "$$")
                            && analysis.tokens[i - 1].span.end == token.span.start;
                        pieces.push((
                            token.span,
                            SemanticTokenKind::TypeParameter,
                            sigil,
                            true,
                            false,
                        ));
                        continue;
                    }
                    let base = match row.map(|row| row.kind) {
                        Some(SymbolKind::Function) => SemanticTokenKind::Function,
                        Some(SymbolKind::Struct | SymbolKind::Enum | SymbolKind::TypeAlias) => {
                            SemanticTokenKind::Type
                        }
                        Some(SymbolKind::Property) => SemanticTokenKind::Property,
                        Some(SymbolKind::Namespace) => SemanticTokenKind::Namespace,
                        Some(SymbolKind::EnumMember) => SemanticTokenKind::EnumMember,
                        Some(SymbolKind::Variable)
                            if row.is_some_and(|row| row.parent.is_some() && row.local) =>
                        {
                            SemanticTokenKind::Parameter
                        }
                        _ => SemanticTokenKind::Variable,
                    };
                    let (kind, constant, expand) = match classes.get(&token.span.start) {
                        Some(&(end, class)) if end == token.span.end => match class {
                            IdeClass::Variable if base == SemanticTokenKind::Parameter => {
                                (base, false, false)
                            }
                            IdeClass::Variable => (SemanticTokenKind::Variable, false, false),
                            IdeClass::Constant => (SemanticTokenKind::Variable, true, false),
                            IdeClass::Function => (SemanticTokenKind::Function, false, false),
                            IdeClass::Macro => (SemanticTokenKind::Function, false, true),
                            IdeClass::Type => (SemanticTokenKind::Type, false, false),
                            IdeClass::Module => (SemanticTokenKind::Namespace, false, false),
                            IdeClass::Field => (SemanticTokenKind::Property, false, false),
                            IdeClass::EnumMember => (SemanticTokenKind::EnumMember, true, false),
                        },
                        _ => (base, false, false),
                    };
                    pieces.push((token.span, kind, declaration, readonly || constant, expand));
                }
            }
        }
        let mut output = vec![];
        for (span, kind, declaration, readonly, expand) in pieces {
            let first = doc.index.position(text, span.start)?.line as usize;
            let last = doc.index.position(text, span.end)?.line as usize;
            for (_, (start, end)) in doc
                .index
                .lines()
                .enumerate()
                .skip(first)
                .take(last - first + 1)
            {
                let start = start.max(span.start);
                let end = end.min(span.end);
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
                    kind,
                    declaration,
                    readonly,
                    expand,
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
pub(crate) fn repair(text: &str, first: Option<usize>) -> Option<String> {
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

pub(crate) fn contains(span: Span, byte: usize) -> bool {
    span.start <= byte && byte <= span.end
}
