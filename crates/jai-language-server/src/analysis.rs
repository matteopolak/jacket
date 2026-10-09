//! Source facts come from the `jaic` lexer and parser; no evaluator runs here.
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Limits, SymbolKind,
    position::LineIndex,
};
use jaic::ast::{Block, Decl, DeclKind, EnumItem, ExprKind, ScopeKind, Stmt, StmtKind, StructLit};
use jaic::intern::Sym;
use jaic::lexer::{P, Tok};
use jaic::source::{Diagnostic as CompilerDiagnostic, FileId, Severity};

/// Byte range into the document text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Directives offered after `#`, with a one-line description.
pub const DIRECTIVES: &[(&str, &str)] = &[
    ("add_context", "Add a member to the Context"),
    ("align", "Alignment of a member or variable"),
    ("as", "Allow implicit conversion to this member"),
    ("asm", "Inline assembly"),
    ("assert", "Compile-time assertion"),
    ("bake_arguments", "A procedure with some arguments fixed"),
    (
        "bake_constants",
        "A procedure with polymorphic constants fixed",
    ),
    ("bytes", "Literal bytes in the instruction stream"),
    ("c_call", "C calling convention"),
    ("caller_code", "The call site as a Code value"),
    ("caller_location", "The call site's source location"),
    ("char", "The value of a one-character string"),
    ("code", "A piece of code as a Code value"),
    ("compile_time", "True while running at compile time"),
    ("compiler", "Implemented by the compiler"),
    ("complete", "Require every enum value in an if-case"),
    ("cpp_method", "C++ method calling convention"),
    ("cpp_return_type_is_non_pod", "C++ non-POD return type"),
    ("deprecated", "Warn when used"),
    ("discard", "Ignore the value"),
    ("elsewhere", "Defined in another compilation unit"),
    ("exists", "Whether a name is defined"),
    ("expand", "A macro, expanded at the call site"),
    ("file", "The path of this file"),
    ("filepath", "The directory of this file"),
    ("foreign", "A procedure from a native library"),
    ("if", "Compile-time condition"),
    ("import", "Import a module"),
    ("insert", "Insert code (a Code value or a string) here"),
    ("intrinsic", "A compiler intrinsic"),
    ("library", "Declare a native library"),
    ("line", "The current line number"),
    ("load", "Load another source file into this scope"),
    ("location", "The source location of a Code value"),
    ("modify", "Check or change polymorphic parameters"),
    ("module_parameters", "Parameters of this module"),
    ("must", "The return value must be used"),
    ("no_abc", "No array bounds checks"),
    ("no_aoc", "No arithmetic overflow checks"),
    ("no_call", "Not callable at run time"),
    ("no_context", "The procedure takes no context"),
    ("no_debug", "No debug information for this procedure"),
    ("no_padding", "Struct without padding"),
    ("placeholder", "A name a metaprogram will define"),
    ("procedure_name", "The name of the enclosing procedure"),
    ("procedure_of_call", "The procedure a call resolves to"),
    ("program_export", "Export a symbol from the executable"),
    ("run", "Run code at compile time"),
    ("runtime_support", "Part of the runtime support"),
    ("scope_export", "Following declarations are exported"),
    (
        "scope_file",
        "Following declarations are private to this file",
    ),
    (
        "scope_module",
        "Following declarations are private to this module",
    ),
    ("specified", "Enum values must be given explicitly"),
    ("string", "A raw multi-line string literal"),
    ("symmetric", "Operator with either argument order"),
    ("system_library", "Declare a system library"),
    ("this", "The enclosing procedure, struct or scope"),
    ("through", "Fall through to the next case"),
    ("type", "Parse what follows as a type"),
    ("type_info_no_size_complaint", "Allow large type info"),
    ("type_info_none", "Emit no type info for this struct"),
    (
        "type_info_procedures_are_void_pointers",
        "Procedure members' type info as void pointers",
    ),
];

/// Words the `jaic` lexer emits as plain identifiers but that the language reserves.
pub const KEYWORDS: &[&str] = &[
    "for",
    "if",
    "ifx",
    "then",
    "else",
    "case",
    "return",
    "struct",
    "while",
    "break",
    "continue",
    "remove",
    "using",
    "defer",
    "size_of",
    "type_of",
    "code_of",
    "initializer_of",
    "type_info",
    "null",
    "enum",
    "true",
    "false",
    "inline",
    "no_inline",
    "cast",
    "xx",
    "context",
    "push_context",
    "operator",
    "is_constant",
    "enum_flags",
    "union",
    "interface",
];

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
    /// Print-family calls with a literal format string.
    pub format_calls: Vec<crate::format::FormatCall>,
    /// `#load` / `#import` strings.
    pub links: Vec<crate::links::Link>,
    pub complete: bool,
}

fn span_of(span: jaic::source::Span) -> Span {
    Span::new(span.start as usize, span.end as usize)
}

struct Context<'a> {
    text: &'a str,
    limits: Limits,
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
            links: vec![],
            complete: false,
        };
        let file = FileId(0);
        let tokens = match jaic::lexer::lex(file, text) {
            Ok(tokens) => tokens,
            Err(error) => {
                result.compiler_diagnostic(&index, text, &error, DiagnosticCode::Lexer);
                return result;
            }
        };
        let mut converted: Vec<Token> = tokens
            .iter()
            .filter_map(|token| {
                let span = span_of(token.span);
                let kind = match &token.tok {
                    Tok::Ident(name) if KEYWORDS.contains(&name.as_str()) => TokenKind::Keyword,
                    Tok::Ident(_) => TokenKind::Ident,
                    Tok::Directive(_) | Tok::Note(_) => TokenKind::Directive,
                    Tok::Int(_) | Tok::Float(_) => TokenKind::Number,
                    Tok::Str(_) => TokenKind::String,
                    Tok::Punct(P::Dot) => TokenKind::Dot,
                    Tok::Punct(_) => TokenKind::Punctuation,
                    Tok::Eof => return None,
                };
                Some(Token {
                    span,
                    kind,
                })
            })
            .collect();
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
        result.links = crate::links::links(&tokens);
        if nesting(&tokens) > limits.recursive_tokens {
            result.diagnostic(
                &index,
                text,
                Span::new(0, 0),
                DiagnosticSeverity::Warning,
                DiagnosticCode::Limit,
                "Conservative parser recursion budget exceeded; tokens remain available.",
            );
            return result;
        }
        let parsed = match jaic::parser::parse_file(file, text) {
            Ok(parsed) => parsed,
            Err(error) => {
                result.compiler_diagnostic(&index, text, &error, DiagnosticCode::Parser);
                return result;
            }
        };
        let context = Context {
            text,
            limits,
        };
        let whole = Span::new(0, text.len());
        let mut private = false;
        result.top_level(&parsed.stmts, whole, &mut private, uri, &index, &context);
        if result.rows.len() >= limits.symbols {
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
        if self.rows.len() >= context.limits.symbols
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

    fn top_level(
        &mut self,
        stmts: &[Stmt],
        scope: Span,
        private: &mut bool,
        uri: &DocumentUri,
        index: &LineIndex,
        context: &Context,
    ) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::Scope(kind) => *private = *kind == ScopeKind::File,
                StmtKind::Decl(decl) => {
                    self.declaration(
                        decl,
                        span_of(stmt.span),
                        scope,
                        None,
                        false,
                        *private,
                        context,
                    );
                }
                StmtKind::Import(import) => {
                    if let Some(name) = import.name {
                        self.add(
                            name.name,
                            SymbolKind::Namespace,
                            span_of(stmt.span),
                            span_of(name.span),
                            scope,
                            None,
                            false,
                            *private,
                            true,
                            context,
                        );
                    }
                }
                StmtKind::StaticIf {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    let mut inner = *private;
                    self.top_level(then_branch, scope, &mut inner, uri, index, context);
                    let mut inner = *private;
                    self.top_level(else_branch, scope, &mut inner, uri, index, context);
                }
                StmtKind::Load {
                    path,
                    span,
                } => {
                    let span = span_of(*span);
                    if let Ok(target) = uri.load(path) {
                        self.loads.push((target, span));
                    } else {
                        self.diagnostic(
                            index,
                            context.text,
                            span,
                            DiagnosticSeverity::Warning,
                            DiagnosticCode::Source,
                            "This #load target is unavailable in the closed document VFS.",
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// The declaration's value decides the symbol kind: a procedure, struct, enum, type, library
    /// or plain constant.
    fn kind_of(decl: &Decl) -> (SymbolKind, bool) {
        if decl.kind == DeclKind::Var {
            return (SymbolKind::Variable, false);
        }
        let kind = match decl.value.as_ref().map(|value| &value.kind) {
            Some(
                ExprKind::Proc(_)
                | ExprKind::Lambda {
                    ..
                }
                | ExprKind::ProcType(_),
            ) => SymbolKind::Function,
            Some(ExprKind::Struct(_)) => SymbolKind::Struct,
            Some(ExprKind::Enum(_)) => SymbolKind::Enum,
            Some(
                ExprKind::TypeDirective {
                    ..
                }
                | ExprKind::ArrayType {
                    ..
                }
                | ExprKind::Unary(jaic::ast::UnOp::Star, _),
            ) => SymbolKind::TypeAlias,
            Some(ExprKind::UnknownDirective {
                name, ..
            }) if matches!(
                name.name.as_str(),
                "library" | "system_library" | "foreign_library" | "foreign_system_library"
            ) =>
            {
                SymbolKind::Namespace
            }
            _ => SymbolKind::Constant,
        };
        (
            kind,
            matches!(kind, SymbolKind::Constant | SymbolKind::Namespace),
        )
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
                StmtKind::StaticIf {
                    then_branch,
                    else_branch,
                    ..
                } => {
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
            StmtKind::While {
                body, ..
            }
            | StmtKind::Defer {
                body, ..
            }
            | StmtKind::PushContext {
                body, ..
            } => self.nested(body, parent, context),
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
            StmtKind::Switch {
                cases, ..
            } => {
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
