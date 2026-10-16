//! Doc comments in the editor features: what hover, completion and signature help show of a
//! declaration's docs, and the intra-doc links inside the comments (go to definition, hover,
//! `textDocument/documentLink`, semantic tokens). The text-level work is in `docs`; this
//! resolves link names through the type checker and renders for the client's format.
use crate::analysis::{KEYWORDS, Span};
use crate::docs::{self, Site};
use crate::hover::{Block, HoverText};
use crate::model::{CompletionData, Location};
use crate::semantic::{self, Decl};
use crate::session::{Session, repair};
use crate::{DocumentUri, MarkupKind, Range, SemanticTokenKind};
use jaic::sema::ide_meta::IdeSignature;
use std::collections::HashMap;
use std::path::Path;

/// Completion lists this short carry their documentation; longer ones leave it to
/// `completionItem/resolve`, so a list of thousands of names costs no more than before.
pub(crate) const EAGER_DOCS: usize = 24;

/// `file:///path#L<line>` for the byte `start` of `text`.
pub(crate) fn url(path: &str, text: &str, start: usize) -> Option<String> {
    let start = text.floor_char_boundary(start.min(text.len()));
    let line = text[..start].matches('\n').count() + 1;
    Some(format!(
        "{}#L{line}",
        DocumentUri::from_path(Path::new(path)).ok()?.as_str()
    ))
}

/// How an editor should color the declaration a link leads to.
fn target_kind(names: usize, text: &str, start: usize, end: usize) -> (SemanticTokenKind, bool) {
    let after = text.get(end..).unwrap_or("").trim_start();
    if let Some(value) = after.strip_prefix("::") {
        let value = value.trim_start();
        let word = value
            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '#'))
            .next()
            .unwrap_or("");
        return match word {
            "struct" | "enum" | "enum_flags" | "union" | "#type" => {
                (SemanticTokenKind::Type, false)
            }
            "#import" => (SemanticTokenKind::Namespace, false),
            "inline" | "no_inline" | "#expand" => (SemanticTokenKind::Function, false),
            _ if value.starts_with('(') => (SemanticTokenKind::Function, false),
            _ if names > 1 && (value.is_empty() || value.starts_with(';')) => {
                (SemanticTokenKind::EnumMember, true)
            }
            _ => (SemanticTokenKind::Variable, true),
        };
    }
    let _ = start;
    if after.starts_with(';') || after.starts_with(',') || after.is_empty() {
        return if names > 1 {
            (SemanticTokenKind::EnumMember, true)
        } else {
            (SemanticTokenKind::Variable, false)
        };
    }
    if names > 1 {
        (SemanticTokenKind::Property, false)
    } else {
        (SemanticTokenKind::Variable, false)
    }
}

impl Session {
    /// URLs of the declarations `names` lead to from `scope_at` in the file at `path`.
    fn link_urls(
        &self,
        uri: &DocumentUri,
        path: &str,
        scope_at: usize,
        names: &[Vec<String>],
    ) -> HashMap<Vec<String>, String> {
        if names.is_empty() {
            return HashMap::new();
        }
        let Ok(doc) = self.document(uri) else {
            return HashMap::new();
        };
        let resolve = |a: &mut semantic::Analysis, _: &Path| {
            Some(
                names
                    .iter()
                    .filter_map(|n| {
                        let (p, t, s, _) = a
                            .resolve_names(Path::new(path), scope_at, n)
                            .into_iter()
                            .next()?;
                        Some((n.clone(), url(&p, &t, s)?))
                    })
                    .collect(),
            )
        };
        // While a call or block is still open the document does not repair; the last compile
        // still knows the declarations the links name.
        match repair(&doc.text, None) {
            Some(probe) => self.with_semantic(uri, &probe, resolve),
            None => self.with_compiled_before(uri, &doc.text, 0, resolve),
        }
        .unwrap_or_default()
    }

    /// Doc comments written for `kind`: Markdown has untagged fences tagged `jai` and the links
    /// that resolve (from `scope_at` in the file at `path`) turned into real ones; plain text
    /// is the comment as written.
    pub(crate) fn render_docs(
        &self,
        uri: &DocumentUri,
        path: &str,
        scope_at: usize,
        raws: &[String],
        kind: MarkupKind,
    ) -> Vec<String> {
        if kind == MarkupKind::PlainText {
            return raws.to_vec();
        }
        let mut names: Vec<Vec<String>> = Vec::new();
        for raw in raws {
            for name in docs::link_names(raw) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        let urls = self.link_urls(uri, path, scope_at, &names);
        raws.iter()
            .map(|raw| docs::markdown(raw, &|n| urls.get(n).cloned()))
            .collect()
    }

    /// The docs of the declaration whose name is `decl`.
    pub(crate) fn declaration_docs(
        &self,
        uri: &DocumentUri,
        decl: &Decl,
        kind: MarkupKind,
    ) -> Option<String> {
        let (path, text, start, _) = decl;
        let raw = docs::declaration(text, *start)?;
        self.render_docs(uri, path, *start, &[raw], kind).pop()
    }

    /// A hover's blocks: the description as code, then the docs of what it describes. Each
    /// overload shows its own docs under its own line.
    pub(crate) fn hover_with_docs(
        &self,
        uri: &DocumentUri,
        value: String,
        layout: Option<String>,
        decls: &[Decl],
        kind: MarkupKind,
    ) -> HoverText {
        let lines: Vec<&str> = value.lines().collect();
        let docs: Vec<Option<String>> = decls
            .iter()
            .map(|d| self.declaration_docs(uri, d, kind))
            .collect();
        let mut blocks = Vec::new();
        if decls.len() > 1 && lines.len() == decls.len() && docs.iter().any(Option::is_some) {
            for (line, doc) in lines.iter().zip(docs) {
                blocks.push(Block::Code((*line).into()));
                blocks.extend(doc.map(Block::Doc));
            }
            return HoverText::new(blocks);
        }
        blocks.push(Block::Code(value.clone()));
        if let Some(layout) = layout {
            blocks.push(Block::Para(vec![crate::hover::Inline::Text(layout)]));
        }
        blocks.extend(docs.into_iter().flatten().next().map(Block::Doc));
        HoverText::new(blocks)
    }

    // ---------------------------------------------------------------------------------------
    // Links inside doc comments
    // ---------------------------------------------------------------------------------------

    /// What each link of `sites` (in the document `uri`) leads to.
    fn resolve_sites(&self, uri: &DocumentUri, sites: &[Site]) -> Vec<Vec<Decl>> {
        let none = || vec![Vec::new(); sites.len()];
        if sites.is_empty() || self.environment.is_none() {
            return none();
        }
        let Ok(doc) = self.document(uri) else {
            return none();
        };
        let Some(probe) = repair(&doc.text, None) else {
            return none();
        };
        self.with_semantic(uri, &probe, |a, path| {
            Some(
                sites
                    .iter()
                    .map(|s| a.resolve_names(path, s.scope_at, &s.names))
                    .collect(),
            )
        })
        .unwrap_or_else(none)
    }

    /// The link whose label holds `byte`, with the declarations it leads to.
    fn link_at(&self, uri: &DocumentUri, byte: usize) -> Option<(Site, Vec<Decl>)> {
        let doc = self.document(uri).ok()?;
        let site = doc
            .sites()
            .iter()
            .find(|s| s.label.0 <= byte && byte <= s.label.1)?
            .clone();
        let decls = self.resolve_sites(uri, std::slice::from_ref(&site)).pop()?;
        (!decls.is_empty()).then_some((site, decls))
    }

    /// Hover on a link in a doc comment: the declaration it leads to, with its docs.
    pub(crate) fn doc_link_hover(
        &self,
        uri: &DocumentUri,
        byte: usize,
        kind: MarkupKind,
    ) -> Option<(usize, usize, HoverText)> {
        let (site, decls) = self.link_at(uri, byte)?;
        let target = decls.first()?;
        let header = docs::signature_line(&target.1, target.2);
        let text = self.hover_with_docs(uri, header, None, std::slice::from_ref(target), kind);
        Some((site.label.0, site.label.1, text))
    }

    /// Go to definition on a link in a doc comment.
    pub(crate) fn doc_link_definition(
        &self,
        uri: &DocumentUri,
        byte: usize,
    ) -> Option<Vec<Location>> {
        let (_, decls) = self.link_at(uri, byte)?;
        let found: Vec<Location> = decls
            .iter()
            .filter_map(|(path, text, start, end)| self.location_in(path, text, *start, *end))
            .collect();
        (!found.is_empty()).then_some(found)
    }

    /// The links of the document's doc comments that lead somewhere: (label, declaration).
    fn resolved_sites(&self, uri: &DocumentUri) -> Vec<(Site, Decl)> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let sites = doc.sites().to_vec();
        let resolved = self.resolve_sites(uri, &sites);
        sites
            .into_iter()
            .zip(resolved)
            .filter_map(|(site, decls)| Some((site, decls.into_iter().next()?)))
            .collect()
    }

    /// `textDocument/documentLink` for doc comment links: the label, and the file with the
    /// declaration's line as a `#L` fragment.
    pub(crate) fn doc_comment_links(&self, uri: &DocumentUri) -> Vec<(Range, String)> {
        self.resolved_sites(uri)
            .into_iter()
            .filter_map(|(site, (path, text, start, _))| {
                let range = self.range_of(uri, Span::new(site.label.0, site.label.1))?;
                Some((range, url(&path, &text, start)?))
            })
            .collect()
    }

    /// Semantic tokens for resolved links: the label colored like the declaration it names.
    pub(crate) fn doc_link_tokens(
        &self,
        uri: &DocumentUri,
    ) -> Vec<(Span, SemanticTokenKind, bool)> {
        self.resolved_sites(uri)
            .into_iter()
            .map(|(site, (_, text, start, end))| {
                let (kind, readonly) = target_kind(site.names.len(), &text, start, end);
                (Span::new(site.label.0, site.label.1), kind, readonly)
            })
            .collect()
    }

    /// Semantic tokens for the code in Jai code blocks of comments, lexed as code: names a call
    /// follows are functions, capitalized names types (nothing is resolved).
    pub(crate) fn fence_tokens(text: &str) -> Vec<(Span, SemanticTokenKind)> {
        use jaic::lexer::{P, Tok};
        let mut out = Vec::new();
        for (start, end) in docs::fenced_code(text) {
            let Ok(tokens) = jaic::lexer::lex(jaic::source::FileId(0), &text[start..end]) else {
                continue;
            };
            for (i, token) in tokens.iter().enumerate() {
                let kind = match &token.tok {
                    Tok::Ident(name) if KEYWORDS.contains(&name.as_str()) => {
                        SemanticTokenKind::Keyword
                    }
                    Tok::Ident(_)
                        if tokens
                            .get(i + 1)
                            .is_some_and(|t| t.tok == Tok::Punct(P::LParen)) =>
                    {
                        SemanticTokenKind::Function
                    }
                    Tok::Ident(name)
                        if name.as_str().starts_with(|c: char| c.is_ascii_uppercase()) =>
                    {
                        SemanticTokenKind::Type
                    }
                    Tok::Ident(_) => SemanticTokenKind::Variable,
                    Tok::Directive(_) => SemanticTokenKind::Macro,
                    Tok::Note(_) => SemanticTokenKind::Decorator,
                    Tok::Int(_) | Tok::Float(_) => SemanticTokenKind::Number,
                    Tok::Str(_) => SemanticTokenKind::String,
                    Tok::Punct(_) => SemanticTokenKind::Operator,
                    Tok::Eof => continue,
                };
                let span = Span::new(
                    start + token.span.start as usize,
                    start + token.span.end as usize,
                );
                out.push((span, kind));
            }
        }
        out
    }

    // ---------------------------------------------------------------------------------------
    // Signature help and completion
    // ---------------------------------------------------------------------------------------

    /// The docs of each signature and of each of its parameters, for the call being typed at
    /// `byte` (its line blanked when the call is still open).
    pub(crate) fn signature_docs(
        &self,
        uri: &DocumentUri,
        byte: usize,
        signatures: &[IdeSignature],
        kind: MarkupKind,
    ) -> Vec<(Option<String>, Vec<Option<String>>)> {
        let none = || {
            signatures
                .iter()
                .map(|s| (None, vec![None; s.params.len()]))
                .collect()
        };
        let Ok(doc) = self.document(uri) else {
            return none();
        };
        let Some(probe) = repair(&doc.text, Some(byte)) else {
            return none();
        };
        // (file, offset of the declaration, the procedure's docs, each parameter's docs)
        type Raw = Option<(String, usize, Option<String>, Vec<Option<String>>)>;
        let raws: Vec<Raw> = self
            .with_semantic(uri, &probe, |a, _| {
                Some(
                    signatures
                        .iter()
                        .map(|s| {
                            let (path, text, start, _) = a.location(s.span)?;
                            let own = docs::header_open(&text, start)
                                .map(|open| docs::parameters(&text, open))
                                .unwrap_or_default();
                            let params = s
                                .param_names
                                .iter()
                                .map(|n| {
                                    let n = n.trim_start_matches('$');
                                    own.iter().find(|(f, _)| f == n).map(|(_, d)| d.clone())
                                })
                                .collect();
                            Some((path, start, docs::above(&text, start), params))
                        })
                        .collect(),
                )
            })
            .unwrap_or_default();
        if raws.len() != signatures.len() {
            return none();
        }
        raws.into_iter()
            .zip(signatures)
            .map(|(raw, s)| {
                let Some((path, start, proc_doc, params)) = raw else {
                    return (None, vec![None; s.params.len()]);
                };
                let mut all: Vec<String> = Vec::new();
                all.extend(proc_doc.clone());
                all.extend(params.iter().flatten().cloned());
                let mut rendered = self.render_docs(uri, &path, start, &all, kind).into_iter();
                let proc_doc = proc_doc.and_then(|_| rendered.next());
                let params = params
                    .iter()
                    .map(|p| p.as_ref().and_then(|_| rendered.next()))
                    .collect();
                (proc_doc, params)
            })
            .collect()
    }

    /// The documentation of the declaration a completion item stands for.
    pub fn completion_documentation(&self, data: &CompletionData) -> Option<String> {
        let document = DocumentUri::parse(&data.document).ok()?;
        let file = DocumentUri::from_path(Path::new(&data.path)).ok()?;
        let text = self.source(&file)?;
        let raw = docs::declaration(&text, data.offset)?;
        self.render_docs(
            &document,
            &data.path,
            data.offset,
            &[raw],
            MarkupKind::Markdown,
        )
        .pop()
    }
}
