//! Source facts come from compiler tokens and parsed nodes; no evaluator runs here.
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Limits, SymbolKind,
    position::LineIndex,
};
use jai_lexer::{Keyword, Kind, Punct, Token};
use jai_source::{SourceId, SourceRecord, SourceSpan, Span, Symbol, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, RecordMember, Statement, StatementKind};

pub(crate) struct SymbolRow {
    pub name: Symbol,
    pub kind: SymbolKind,
    pub location: SourceSpan,
    pub selection: Span,
    pub scope: Span,
    pub parent: Option<usize>,
    pub local: bool,
    pub file_private: bool,
    pub readonly: bool,
}
pub(crate) struct Analysis {
    pub source: SourceId,
    pub tokens: Vec<Token>,
    pub rows: Vec<SymbolRow>,
    pub diagnostics: Vec<Diagnostic>,
    pub loads: Vec<(DocumentUri, Span)>,
    pub opaque_scopes: Vec<Span>,
    pub complete: bool,
}
impl Analysis {
    pub fn build(
        record: &SourceRecord,
        uri: &DocumentUri,
        symbols: &mut Symbols,
        limits: Limits,
    ) -> Self {
        let text = record.text();
        let index = LineIndex::new(text);
        let mut result = Self {
            source: record.id(),
            tokens: vec![],
            rows: vec![],
            diagnostics: vec![],
            loads: vec![],
            opaque_scopes: vec![],
            complete: false,
        };
        let tokens = match jai_lexer::lex(text) {
            Ok(tokens) => tokens,
            Err(error) => {
                result.diagnostic(
                    &index,
                    text,
                    error.span,
                    DiagnosticSeverity::Error,
                    DiagnosticCode::Lexer,
                    &error.message,
                );
                return result;
            }
        };
        if tokens.len() > limits.tokens {
            let at = tokens[limits.tokens].span;
            result.diagnostic(
                &index,
                text,
                at,
                DiagnosticSeverity::Warning,
                DiagnosticCode::Limit,
                "Token budget exceeded; syntax navigation is unavailable for this version.",
            );
            result.tokens = tokens.into_iter().take(limits.tokens).collect();
            return result;
        }
        let recursive = tokens.iter().filter(|token| recursive(token.kind)).count();
        result.tokens = tokens;
        if recursive > limits.recursive_tokens {
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
        let file = match jai_syntax::parse_file(record, symbols) {
            Ok(file) => file,
            Err(error) => {
                result.diagnostic(
                    &index,
                    text,
                    error.location.span,
                    DiagnosticSeverity::Error,
                    DiagnosticCode::Parser,
                    &error.message,
                );
                return result;
            }
        };
        for item in file.items() {
            match item {
                FileItem::Declaration(declaration)
                | FileItem::UsingDeclaration {
                    declaration, ..
                } => {
                    let span = declaration.location.span;
                    let private = declaration.visibility == jai_syntax::Visibility::File;
                    result.declaration(
                        &declaration.kind,
                        span,
                        Span::new(0, text.len()),
                        None,
                        false,
                        private,
                        text,
                        symbols,
                        limits,
                    );
                }
                FileItem::Load(load) => {
                    if let Ok(target) = uri.load(&load.target) {
                        result.loads.push((target, load.location.span));
                    } else {
                        result.diagnostic(
                            &index,
                            text,
                            load.location.span,
                            DiagnosticSeverity::Warning,
                            DiagnosticCode::Source,
                            "This #load target is unavailable in the closed document VFS.",
                        );
                    }
                }
                _ => {}
            }
        }
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
    pub fn diagnostic(
        &mut self,
        index: &LineIndex,
        text: &str,
        span: Span,
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: &str,
    ) {
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
        name: Symbol,
        kind: SymbolKind,
        span: Span,
        scope: Span,
        parent: Option<usize>,
        local: bool,
        private: bool,
        readonly: bool,
        text: &str,
        symbols: &Symbols,
        limits: Limits,
    ) -> Option<usize> {
        if self.rows.len() >= limits.symbols {
            return None;
        }
        let spelling = symbols.name(name);
        let selection = self
            .tokens
            .iter()
            .find(|token| {
                token.kind == Kind::Ident
                    && token.span.start >= span.start
                    && token.span.end <= span.end
                    && token.spelling(text) == spelling
            })?
            .span;
        let id = self.rows.len();
        self.rows.push(SymbolRow {
            name,
            kind,
            location: SourceSpan {
                source: self.source,
                span,
            },
            selection,
            scope,
            parent,
            local,
            file_private: private,
            readonly,
        });
        Some(id)
    }
    #[allow(clippy::too_many_arguments)]
    fn declaration(
        &mut self,
        declaration: &FileDeclarationKind,
        span: Span,
        scope: Span,
        parent: Option<usize>,
        local: bool,
        private: bool,
        text: &str,
        symbols: &Symbols,
        limits: Limits,
    ) {
        match declaration {
            FileDeclarationKind::Procedure(p) => {
                if let Some(id) = self.add(
                    p.name,
                    SymbolKind::Function,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    false,
                    text,
                    symbols,
                    limits,
                ) {
                    for parameter in &p.parameters {
                        self.add(
                            parameter.name,
                            SymbolKind::Variable,
                            parameter.span,
                            span,
                            Some(id),
                            true,
                            true,
                            false,
                            text,
                            symbols,
                            limits,
                        );
                    }
                    self.statements(&p.body, span, Some(id), text, symbols, limits);
                }
            }
            FileDeclarationKind::ProcedurePrototype(p) => {
                self.add(
                    p.name,
                    SymbolKind::Function,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    false,
                    text,
                    symbols,
                    limits,
                );
            }
            FileDeclarationKind::Record(r) => {
                if let Some(id) = self.add(
                    r.name,
                    SymbolKind::Struct,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    false,
                    text,
                    symbols,
                    limits,
                ) {
                    self.record(&r.members, r.span, id, text, symbols, limits);
                }
            }
            FileDeclarationKind::Enum(e) => {
                if let Some(id) = self.add(
                    e.name,
                    SymbolKind::Enum,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    false,
                    text,
                    symbols,
                    limits,
                ) {
                    let mut enum_items: Vec<_> = e.members.iter().rev().collect();
                    while let Some(item) = enum_items.pop() {
                        let member = match item {
                            jai_syntax::EnumBodyItem::Member(member) => member,
                            jai_syntax::EnumBodyItem::Conditional {
                                then_items,
                                else_items,
                                ..
                            } => {
                                enum_items.extend(else_items.iter().rev());
                                enum_items.extend(then_items.iter().rev());
                                continue;
                            }
                            jai_syntax::EnumBodyItem::Insert(_) => continue,
                        };
                        self.add(
                            member.name,
                            SymbolKind::EnumMember,
                            member.span,
                            e.span,
                            Some(id),
                            false,
                            true,
                            true,
                            text,
                            symbols,
                            limits,
                        );
                    }
                }
            }
            FileDeclarationKind::Global(g) => {
                self.add(
                    g.declaration.name(),
                    SymbolKind::Variable,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    false,
                    text,
                    symbols,
                    limits,
                );
            }
            FileDeclarationKind::Constant(c) => {
                self.add(
                    c.name,
                    SymbolKind::Constant,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    true,
                    text,
                    symbols,
                    limits,
                );
            }
            FileDeclarationKind::TypeAlias(t) => {
                self.add(
                    t.name,
                    SymbolKind::TypeAlias,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    false,
                    text,
                    symbols,
                    limits,
                );
            }
            FileDeclarationKind::Library(l) => {
                self.add(
                    l.name,
                    SymbolKind::Namespace,
                    span,
                    scope,
                    parent,
                    local,
                    private,
                    true,
                    text,
                    symbols,
                    limits,
                );
            }
            // Placeholder/operator aliases need authentic semantic publication before binding.
            _ => {}
        }
    }
    fn record(
        &mut self,
        members: &[RecordMember],
        scope: Span,
        parent: usize,
        text: &str,
        symbols: &Symbols,
        limits: Limits,
    ) {
        for member in members {
            match member {
                RecordMember::Field(f) => {
                    self.add(
                        f.name,
                        SymbolKind::Property,
                        f.span,
                        scope,
                        Some(parent),
                        false,
                        true,
                        false,
                        text,
                        symbols,
                        limits,
                    );
                }
                RecordMember::Constant(c) => {
                    self.add(
                        c.name,
                        SymbolKind::Constant,
                        c.span,
                        scope,
                        Some(parent),
                        false,
                        true,
                        true,
                        text,
                        symbols,
                        limits,
                    );
                }
                RecordMember::Record(r) => {
                    if let Some(id) = self.add(
                        r.name,
                        SymbolKind::Struct,
                        r.span,
                        scope,
                        Some(parent),
                        false,
                        true,
                        false,
                        text,
                        symbols,
                        limits,
                    ) {
                        self.record(&r.members, r.span, id, text, symbols, limits);
                    }
                }
                RecordMember::TypeAlias(t) => {
                    self.add(
                        t.name,
                        SymbolKind::TypeAlias,
                        t.span,
                        scope,
                        Some(parent),
                        false,
                        true,
                        false,
                        text,
                        symbols,
                        limits,
                    );
                }
                _ => {}
            }
        }
    }
    fn statements(
        &mut self,
        body: &[Statement],
        scope: Span,
        parent: Option<usize>,
        text: &str,
        symbols: &Symbols,
        limits: Limits,
    ) {
        for statement in body {
            match &statement.kind {
                StatementKind::Declare(d) => {
                    self.add(
                        d.name(),
                        SymbolKind::Variable,
                        statement.span,
                        scope,
                        parent,
                        true,
                        true,
                        false,
                        text,
                        symbols,
                        limits,
                    );
                }
                StatementKind::Constant(c) => {
                    self.add(
                        c.name,
                        SymbolKind::Constant,
                        statement.span,
                        scope,
                        parent,
                        true,
                        true,
                        true,
                        text,
                        symbols,
                        limits,
                    );
                }
                StatementKind::If(_, yes, no) => {
                    for branch in [yes, no] {
                        if let (Some(first), Some(last)) = (branch.first(), branch.last()) {
                            self.statements(
                                branch,
                                Span::new(first.span.start, last.span.end),
                                parent,
                                text,
                                symbols,
                                limits,
                            );
                        }
                    }
                }
                StatementKind::Block(body)
                | StatementKind::CheckScope {
                    body, ..
                }
                | StatementKind::PushContext {
                    body, ..
                } => {
                    self.statements(body, statement.span, parent, text, symbols, limits);
                }
                StatementKind::Return(_)
                | StatementKind::ReturnValues(_)
                | StatementKind::Expression(_)
                | StatementKind::Assign(_, _)
                | StatementKind::Update(_, _, _)
                | StatementKind::AssignPlace {
                    ..
                }
                | StatementKind::UpdatePlace {
                    ..
                } => {}
                // Unknown scope producers must not let a global masquerade as a local binding.
                _ => self.opaque_scopes.push(statement.span),
            }
        }
    }
}
fn recursive(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Punctuation(
            Punct::OpenParen
                | Punct::OpenBrace
                | Punct::OpenBracket
                | Punct::StructLiteral
                | Punct::ArrayLiteral
                | Punct::Sub
                | Punct::Not
                | Punct::Complement
                | Punct::Mul
        ) | Kind::Keyword(
            Keyword::If
                | Keyword::Ifx
                | Keyword::While
                | Keyword::For
                | Keyword::Struct
                | Keyword::Union
                | Keyword::Cast
                | Keyword::TypeOf
        )
    )
}
