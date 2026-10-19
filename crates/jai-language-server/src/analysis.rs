//! Source facts come from the `jaic` lexer and parser; no evaluator runs here.
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Limits, SymbolKind,
    position::LineIndex,
};
use jaic::intern::Sym;
use jaic::lexer::{P, Tok};
use jaic::parser::ide_facts::{self};
use jaic::source::{Diagnostic as CompilerDiagnostic, FileId, Severity};
use std::rc::Rc;

mod symbols;

/// Byte range into the document text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Ident,
    Keyword,
    String,
    Number,
    /// `#directive` or `@note`.
    Directive,
    Dot,
    Punctuation,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    pub fn spelling<'a>(&self, text: &'a str) -> &'a str {
        &text[self.span.start..self.span.end]
    }
}

/// Where a declaration's symbols live: the span they are visible in, their parent symbol and
/// whether they are local or private to the file.
pub(crate) struct Placement {
    pub scope: Span,
    pub parent: Option<usize>,
    pub local: bool,
    pub private: bool,
}

#[derive(Clone)]
pub(crate) struct SymbolRow {
    pub name: Sym,
    pub kind: SymbolKind,
    pub location: Span,
    pub selection: Span,
    pub scope: Span,
    pub parent: Option<usize>,
    pub local: bool,
    pub file_private: bool,
    pub readonly: bool,
}

#[derive(Clone)]
pub(crate) struct Analysis {
    pub tokens: Vec<Token>,
    pub rows: Vec<SymbolRow>,
    pub diagnostics: Vec<Diagnostic>,
    pub loads: Vec<(DocumentUri, Span)>,
    pub opaque_scopes: Vec<Span>,
    /// Calls with a literal format string (candidates; see `Session::format_calls`).
    pub format_calls: Vec<crate::format::FormatCall>,
    /// Names this document declares with the `@PrintLike` note.
    pub print_like: jaic::fxhash::HashSet<String>,
    /// The `$` names of each procedure this document declares.
    pub type_parameters: Vec<TypeParameter>,
    /// The compiler's tokens of the text (after parse recovery), for the features that read
    /// syntax the rows do not keep, and the lexical errors among them.
    pub syntax_tokens: Rc<[jaic::lexer::Token]>,
    pub lex_errors: Vec<jaic::source::Span>,
    /// `#load` / `#import` strings.
    pub links: Vec<jaic::project::DirectiveLink>,
    pub complete: bool,
    /// Hash and length of the text this was built from, so a session can keep it while a
    /// document is unchanged (`Session::rebuild`).
    pub key: (u64, usize),
}

/// A name a procedure introduces with `$` (`$T`, `$$n`): it is a type parameter throughout the
/// procedure's declaration.
#[derive(Clone)]
pub(crate) struct TypeParameter {
    pub name: Sym,
    /// The name after its sigil.
    pub declaration: Span,
    pub scope: Span,
}

fn span_of(span: jaic::source::Span) -> Span {
    Span::new(span.start as usize, span.end as usize)
}

struct Context<'a> {
    text: &'a str,
    limits: Limits,
}

fn editor_tokens(tokens: &[jaic::lexer::Token]) -> Vec<Token> {
    let mut converted = Vec::with_capacity(tokens.len());
    converted.extend(tokens.iter().filter_map(|token| {
        let kind = match &token.tok {
            Tok::Ident(name) if jaic::lexicon::is_reserved(name.as_str()) => TokenKind::Keyword,
            Tok::Ident(_) => TokenKind::Ident,
            Tok::Directive(_) | Tok::Note(_) => TokenKind::Directive,
            Tok::Int(_) | Tok::Float(_) => TokenKind::Number,
            Tok::Str(_) => TokenKind::String,
            Tok::Punct(P::Dot) => TokenKind::Dot,
            Tok::Punct(_) => TokenKind::Punctuation,
            Tok::Eof => return None,
        };
        Some(Token {
            span: span_of(token.span),
            kind,
        })
    }));
    converted
}

impl Analysis {
    pub fn build(text: &str, uri: &DocumentUri, limits: Limits) -> Self {
        let index = LineIndex::new(text);
        let mut result = Self {
            tokens: vec![],
            rows: vec![],
            diagnostics: vec![],
            loads: vec![],
            opaque_scopes: vec![],
            format_calls: vec![],
            print_like: jaic::fxhash::HashSet::default(),
            type_parameters: vec![],
            syntax_tokens: Rc::from([]),
            lex_errors: vec![],
            links: vec![],
            complete: false,
            key: crate::semantic::text_key(text),
        };
        let file = FileId(0);
        let (tokens, lex_errors) = jaic::lexer::lex_ide(file, text);
        for error in &lex_errors {
            result.compiler_diagnostic(&index, text, error, DiagnosticCode::Lexer);
        }
        result.lex_errors = lex_errors.iter().map(|e| e.span).collect();
        // One entry per token but the end marker: sized once, not grown by doubling.
        let mut converted = editor_tokens(&tokens);
        if converted.len() > limits.tokens {
            let at = converted[limits.tokens].span;
            result.diagnostic(
                &index,
                text,
                at,
                DiagnosticSeverity::Warning,
                DiagnosticCode::Limit,
                "Token budget exceeded; syntax navigation is unavailable for this version.",
            );
            converted.truncate(limits.tokens);
            result.tokens = converted;
            return result;
        }
        result.tokens = converted;
        result.format_calls = crate::format::calls(&tokens, text);
        result.links = jaic::project::scan_directives(&tokens);
        if nesting(&tokens) > limits.recursive_tokens {
            result.diagnostic(
                &index,
                text,
                Span::new(0, 0),
                DiagnosticSeverity::Warning,
                DiagnosticCode::Limit,
                "Conservative parser recursion budget exceeded; tokens remain available.",
            );
            result.syntax_tokens = tokens.into();
            return result;
        }
        let expected_rows = (tokens.len() / 16).min(limits.rows);
        let parsed = jaic::parser::parse_tokens_ide(file, text, tokens);
        result.diagnostics.retain(|d| {
            d.code != DiagnosticCode::Lexer
                || index
                    .byte(text, d.range.start)
                    .is_ok_and(|byte| !parsed.recovered_literal_at(byte as u32))
        });
        crate::semantic::note_parse(
            text,
            lex_errors
                .first()
                .or(parsed.diagnostics.first())
                .map(|d| d.span.start as usize),
        );
        for error in &parsed.diagnostics {
            result.compiler_diagnostic(&index, text, error, DiagnosticCode::Parser);
        }
        // Recovery can rescan a multiline token. All syntax consumers must use
        // those current tokens too, or an unaffected name may still look like a string.
        result.tokens = editor_tokens(&parsed.tokens);
        result.format_calls = crate::format::calls(&parsed.tokens, text);
        result.links = jaic::project::scan_directives(&parsed.tokens);
        if result.tokens.len() > limits.tokens {
            let at = result.tokens[limits.tokens].span;
            result.tokens.truncate(limits.tokens);
            result.diagnostic(&index, text, at, DiagnosticSeverity::Warning, DiagnosticCode::Limit,
                "Token budget exceeded after recovery; syntax navigation is unavailable for this version.");
            return result;
        }
        result.procedure_facts(&parsed.file);
        result.syntax_tokens = parsed.tokens.into();
        let parsed = parsed.file;
        let context = Context {
            text,
            limits,
        };
        result.rows.reserve(expected_rows);
        result.top_level(&parsed, uri, &index, &context);
        if result.rows.len() >= limits.rows {
            result.diagnostic(
                &index,
                text,
                Span::new(0, 0),
                DiagnosticSeverity::Warning,
                DiagnosticCode::Limit,
                "Declaration budget reached; navigation results are incomplete.",
            );
        } else {
            result.complete = true;
        }
        result
    }

    fn compiler_diagnostic(
        &mut self,
        index: &LineIndex,
        text: &str,
        error: &CompilerDiagnostic,
        code: DiagnosticCode,
    ) {
        let severity = match error.severity {
            Severity::Error => DiagnosticSeverity::Error,
            Severity::Warning | Severity::Note => DiagnosticSeverity::Warning,
        };
        self.diagnostic(
            index,
            text,
            span_of(error.span),
            severity,
            code,
            &error.message,
        );
    }

    pub fn diagnostic(
        &mut self,
        index: &LineIndex,
        text: &str,
        span: Span,
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: &str,
    ) {
        let span = Span::new(span.start.min(text.len()), span.end.min(text.len()));
        if let Ok(range) = index.range(text, span) {
            let end = message.floor_char_boundary(message.len().min(2048));
            self.diagnostics.push(Diagnostic {
                range,
                severity,
                code,
                message: message[..end].into(),
            });
        }
    }

    fn add(&mut self, row: SymbolRow, context: &Context) -> Option<usize> {
        if self.rows.len() >= context.limits.rows
            || row.selection.end > context.text.len()
            || row.location.end > context.text.len()
        {
            return None;
        }
        let id = self.rows.len();
        self.rows.push(row);
        Some(id)
    }

    /// The `@PrintLike` names and `$` parameters of the document's procedures.
    fn procedure_facts(&mut self, file: &jaic::ast::File) {
        for procedure in ide_facts::procedures(file) {
            if procedure.has_note(jaic::format_string::PRINT_LIKE_NOTE) {
                self.print_like
                    .insert(procedure.name.name.as_str().to_owned());
            }
            let scope = span_of(procedure.span);
            self.type_parameters.extend(
                ide_facts::type_parameters(procedure.header)
                    .into_iter()
                    .map(|name| TypeParameter {
                        name: name.name,
                        declaration: span_of(name.span),
                        scope,
                    }),
            );
        }
    }
}

/// Deepest bracket nesting plus the longest run of prefix operators: the shapes that make a
/// recursive-descent parser recurse deeply on a small input.
fn nesting(tokens: &[jaic::lexer::Token]) -> usize {
    let (mut depth, mut deepest, mut run, mut longest) = (0usize, 0usize, 0usize, 0usize);
    for token in tokens {
        let prefix = match &token.tok {
            Tok::Punct(P::Minus | P::Bang | P::Tilde | P::Star) => true,
            Tok::Ident(name) => {
                matches!(name.as_str(), "cast" | "type_of" | "ifx" | "if" | "while")
            }
            _ => false,
        };
        run = if prefix {
            run + 1
        } else {
            0
        };
        longest = longest.max(run);
        match &token.tok {
            Tok::Punct(P::LParen | P::LBrace | P::LBracket | P::DotBrace | P::DotBracket) => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            Tok::Punct(P::RParen | P::RBrace | P::RBracket) => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest.max(longest)
}
