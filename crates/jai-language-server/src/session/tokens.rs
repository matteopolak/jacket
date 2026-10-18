//! Semantic tokens.
use super::{Session, contains};
use crate::analysis::{Span, TokenKind};
use crate::{DocumentUri, Error, SemanticToken, SemanticTokenKind, SymbolKind};
use jaic::fxhash::{HashMap, HashSet};
use jaic::sema::ide_meta::IdeClass;
use std::collections::BTreeMap;

impl Session {
    /// Tokens classified by the syntax layer, refined by the type checker when the session has
    /// an environment: uses of types, procedures, macros, modules, constants and enum members,
    /// `$T` parameters, notes, and the `%` directives of format strings.
    pub fn semantic_tokens(&self, uri: &DocumentUri) -> Result<Vec<SemanticToken>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let text = doc.text.as_str();
        let analysis = &self
            .analysis_for(uri)
            .expect("opened document has analysis");
        let classes: BTreeMap<usize, (usize, IdeClass)> = if self.environment.is_some() {
            self.identifier_classes(uri)
                .into_iter()
                .map(|(span, class)| (span.start, (span.end, class)))
                .collect()
        } else {
            BTreeMap::new()
        };
        // `$T` / `$$T`: the name is a type parameter throughout its procedure.
        let mut poly: HashMap<&str, Vec<Span>> = HashMap::default();
        let mut poly_declarations: HashSet<Span> = HashSet::default();
        for parameter in &analysis.type_parameters {
            poly.entry(parameter.name.as_str())
                .or_default()
                .push(parameter.scope);
            poly_declarations.insert(parameter.declaration);
        }
        let format_calls = self.format_calls(uri);
        let mut specs: Vec<Span> = format_calls
            .iter()
            .flat_map(|c| c.specs.iter().map(|s| s.span))
            .collect();
        specs.sort_by_key(|s| s.start);
        // The first row declared at each token (a row per token would be a scan per token).
        let mut declared: HashMap<Span, usize> = HashMap::default();
        declared.reserve(analysis.rows.len());
        for (at, row) in analysis.rows.iter().enumerate() {
            declared.entry(row.selection).or_insert(at);
        }
        // (span, kind, declaration, readonly, expand)
        let mut pieces: Vec<(Span, SemanticTokenKind, bool, bool, bool)> = Vec::new();
        for token in &analysis.tokens {
            let row = declared.get(&token.span).map(|&at| &analysis.rows[at]);
            let declaration = row.is_some();
            let readonly = row.is_some_and(|row| row.readonly);
            let piece = |kind| (token.span, kind, declaration, readonly, false);
            match token.kind {
                // `#string WGSL`: the editor highlights the body as that language.
                TokenKind::String if crate::here_string::names_language(token.spelling(text)) => {}
                TokenKind::String => {
                    let mut at = token.span.start;
                    let first = specs.partition_point(|s| s.start < token.span.start);
                    for spec in specs[first..]
                        .iter()
                        .take_while(|s| s.start < token.span.end)
                        .filter(|s| s.end <= token.span.end)
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
                    pieces.push(piece(SemanticTokenKind::Decorator));
                }
                TokenKind::Directive => pieces.push(piece(SemanticTokenKind::Macro)),
                TokenKind::Dot | TokenKind::Punctuation => {
                    pieces.push(piece(SemanticTokenKind::Operator));
                }
                TokenKind::Ident => {
                    let spelling = token.spelling(text);
                    if poly.get(spelling).is_some_and(|scopes| {
                        scopes
                            .iter()
                            .any(|scope| contains(*scope, token.span.start))
                    }) {
                        pieces.push((
                            token.span,
                            SemanticTokenKind::TypeParameter,
                            poly_declarations.contains(&token.span),
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
        // Code in the Jai code blocks of comments.
        let fences = Self::fence_tokens(text);
        if !fences.is_empty() {
            pieces.extend(
                fences
                    .into_iter()
                    .map(|(span, kind)| (span, kind, false, false, false)),
            );
            pieces.sort_by_key(|p| p.0.start);
        }
        // Links in doc comments, colored like what they name (a comment has no token of its own).
        if self.environment.is_some() {
            let links = self.doc_link_tokens(uri);
            if !links.is_empty() {
                pieces.extend(
                    links
                        .into_iter()
                        .map(|(span, kind, readonly)| (span, kind, false, readonly, false)),
                );
                pieces.sort_by_key(|p| p.0.start);
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
