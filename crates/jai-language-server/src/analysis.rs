//! Source facts come from the `jaic` lexer and parser; no evaluator runs here.
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Limits, SymbolKind,
    position::LineIndex,
};
use jaic::ast::{Block, Decl, EnumItem, ExprKind, Stmt, StmtKind, StructLit};
use jaic::intern::Sym;
use jaic::lexer::{P, Tok};
use jaic::parser::ide_facts::{self, DeclClass, TopLevelKind, Visibility};
use jaic::source::{Diagnostic as CompilerDiagnostic, FileId, Severity};
use std::rc::Rc;

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

    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        name: Sym,
        kind: SymbolKind,
        location: Span,
        selection: Span,
        scope: Span,
        parent: Option<usize>,
        local: bool,
        private: bool,
        readonly: bool,
        context: &Context,
    ) -> Option<usize> {
        if self.rows.len() >= context.limits.rows
            || selection.end > context.text.len()
            || location.end > context.text.len()
        {
            return None;
        }
        let id = self.rows.len();
        self.rows.push(SymbolRow {
            name,
            kind,
            location,
            selection,
            scope,
            parent,
            local,
            file_private: private,
            readonly,
        });
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

    fn top_level(
        &mut self,
        file: &jaic::ast::File,
        uri: &DocumentUri,
        index: &LineIndex,
        context: &Context,
    ) {
        let whole = Span::new(0, context.text.len());
        for item in ide_facts::top_level(file) {
            let span = span_of(item.span);
            let private = item.visibility == Visibility::File;
            match item.kind {
                TopLevelKind::Decl(decl) => {
                    self.declaration(decl, span, whole, None, false, private, context);
                }
                TopLevelKind::Import(import) => {
                    if let Some(name) = import.name {
                        self.add(
                            name.name,
                            SymbolKind::Namespace,
                            span,
                            span_of(name.span),
                            whole,
                            None,
                            false,
                            private,
                            true,
                            context,
                        );
                    }
                }
                TopLevelKind::Load(path, at) => match uri.load(path) {
                    Ok(target) => self.loads.push((target, span_of(at))),
                    Err(_) => self.diagnostic(
                        index,
                        context.text,
                        span_of(at),
                        DiagnosticSeverity::Warning,
                        DiagnosticCode::Source,
                        "This #load target is unavailable in the closed document VFS.",
                    ),
                },
                TopLevelKind::Assert(_) => {}
            }
        }
    }

    /// The declaration's value decides the symbol kind, and whether it is read-only.
    fn kind_of(decl: &Decl) -> (SymbolKind, bool) {
        match ide_facts::decl_class(decl) {
            DeclClass::Variable => (SymbolKind::Variable, false),
            DeclClass::Procedure => (SymbolKind::Function, false),
            DeclClass::Struct => (SymbolKind::Struct, false),
            DeclClass::Enum => (SymbolKind::Enum, false),
            DeclClass::TypeAlias => (SymbolKind::TypeAlias, false),
            DeclClass::Library => (SymbolKind::Namespace, true),
            DeclClass::Constant => (SymbolKind::Constant, true),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn declaration(
        &mut self,
        decl: &Decl,
        span: Span,
        scope: Span,
        parent: Option<usize>,
        local: bool,
        private: bool,
        context: &Context,
    ) {
        let (kind, readonly) = Self::kind_of(decl);
        for name in &decl.names {
            let Some(id) = self.add(
                name.name,
                kind,
                span,
                span_of(name.span),
                scope,
                parent,
                local,
                private,
                readonly,
                context,
            ) else {
                continue;
            };
            let Some(value) = &decl.value else {
                continue;
            };
            match &value.kind {
                ExprKind::Proc(lit) => {
                    let inner = span_of(lit.header.span.to(value.span));
                    for param in &lit.header.params {
                        if let Some(param_name) = param.name {
                            self.add(
                                param_name.name,
                                SymbolKind::Variable,
                                span_of(param.span),
                                span_of(param_name.span),
                                inner,
                                Some(id),
                                true,
                                true,
                                false,
                                context,
                            );
                        }
                    }
                    if let Some(body) = &lit.body {
                        self.block(body, Some(id), context);
                    }
                }
                ExprKind::Struct(record) => self.record(record, id, context),
                ExprKind::Enum(lit) => {
                    let mut pending: Vec<&EnumItem> = lit.items.iter().rev().collect();
                    while let Some(item) = pending.pop() {
                        match item {
                            EnumItem::Member(member) => {
                                self.add(
                                    member.name.name,
                                    SymbolKind::EnumMember,
                                    span_of(member.name.span),
                                    span_of(member.name.span),
                                    span_of(lit.span),
                                    Some(id),
                                    false,
                                    true,
                                    true,
                                    context,
                                );
                            }
                            EnumItem::If {
                                then_items,
                                else_items,
                                ..
                            } => {
                                pending.extend(else_items.iter().rev());
                                pending.extend(then_items.iter().rev());
                            }
                            EnumItem::Insert(_) => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn record(&mut self, record: &StructLit, parent: usize, context: &Context) {
        let scope = span_of(record.span);
        let mut pending: Vec<&Stmt> = record.body.iter().rev().collect();
        while let Some(stmt) = pending.pop() {
            match &stmt.kind {
                StmtKind::Decl(decl) => {
                    let (kind, readonly) = Self::kind_of(decl);
                    let kind = if kind == SymbolKind::Variable {
                        SymbolKind::Property
                    } else {
                        kind
                    };
                    for name in &decl.names {
                        let Some(id) = self.add(
                            name.name,
                            kind,
                            span_of(stmt.span),
                            span_of(name.span),
                            scope,
                            Some(parent),
                            false,
                            true,
                            readonly,
                            context,
                        ) else {
                            continue;
                        };
                        if let Some(ExprKind::Struct(nested)) = decl.value.as_ref().map(|v| &v.kind)
                        {
                            self.record(nested, id, context);
                        }
                    }
                }
                StmtKind::StaticIf(boxed) => {
                    let jaic::ast::StaticIfStmt {
                        then_branch,
                        else_branch,
                        ..
                    } = &**boxed;
                    pending.extend(else_branch.iter().rev());
                    pending.extend(then_branch.iter().rev());
                }
                _ => {}
            }
        }
    }

    fn block(&mut self, block: &Block, parent: Option<usize>, context: &Context) {
        self.statements(&block.stmts, span_of(block.span), parent, context);
    }

    fn statements(&mut self, body: &[Stmt], scope: Span, parent: Option<usize>, context: &Context) {
        for stmt in body {
            self.statement(stmt, scope, parent, context);
        }
    }

    fn nested(&mut self, stmt: &Stmt, parent: Option<usize>, context: &Context) {
        let scope = span_of(stmt.span);
        match &stmt.kind {
            StmtKind::Block(block) => self.statements(&block.stmts, scope, parent, context),
            _ => self.statement(stmt, scope, parent, context),
        }
    }

    fn statement(&mut self, stmt: &Stmt, scope: Span, parent: Option<usize>, context: &Context) {
        match &stmt.kind {
            StmtKind::Decl(decl) => {
                self.declaration(decl, span_of(stmt.span), scope, parent, true, true, context);
            }
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.nested(then_branch, parent, context);
                if let Some(branch) = else_branch {
                    self.nested(branch, parent, context);
                }
            }
            StmtKind::Block(block) => {
                self.statements(&block.stmts, span_of(stmt.span), parent, context);
            }
            StmtKind::While(w) => self.nested(&w.body, parent, context),
            StmtKind::Defer {
                body, ..
            } => self.nested(body, parent, context),
            StmtKind::PushContext(p) => self.nested(&p.body, parent, context),
            StmtKind::For(for_loop) => {
                let span = span_of(stmt.span);
                for name in [for_loop.it, for_loop.index].into_iter().flatten() {
                    self.add(
                        name.name,
                        SymbolKind::Variable,
                        span,
                        span_of(name.span),
                        span,
                        parent,
                        true,
                        true,
                        false,
                        context,
                    );
                }
                self.nested(&for_loop.body, parent, context);
            }
            StmtKind::Switch(boxed) => {
                let jaic::ast::SwitchStmt {
                    cases, ..
                } = &**boxed;
                for case in cases {
                    self.statements(&case.body, span_of(case.span), parent, context);
                }
            }
            StmtKind::Expr(_)
            | StmtKind::Assign {
                ..
            }
            | StmtKind::Return {
                ..
            }
            | StmtKind::Break(_)
            | StmtKind::Continue(_)
            | StmtKind::Remove(_)
            | StmtKind::Empty => {}
            // Unknown scope producers must not let a global masquerade as a local binding.
            _ => self.opaque_scopes.push(span_of(stmt.span)),
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
