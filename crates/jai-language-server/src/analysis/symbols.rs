//! The document symbol tree: declarations, their children and the scopes of local bindings,
//! built from the parsed file.
use super::{Analysis, Context, Span, span_of};
use crate::{DiagnosticCode, DiagnosticSeverity, DocumentUri, SymbolKind, position::LineIndex};
use jaic::ast::{Block, Decl, EnumItem, ExprKind, Stmt, StmtKind, StructLit};
use jaic::parser::ide_facts::{self, DeclClass, TopLevelKind, Visibility};

impl Analysis {
    pub(super) fn top_level(
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
    pub(super) fn kind_of(decl: &Decl) -> (SymbolKind, bool) {
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
    pub(super) fn declaration(
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

    pub(super) fn record(&mut self, record: &StructLit, parent: usize, context: &Context) {
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

    pub(super) fn block(&mut self, block: &Block, parent: Option<usize>, context: &Context) {
        self.statements(&block.stmts, span_of(block.span), parent, context);
    }

    pub(super) fn statements(
        &mut self,
        body: &[Stmt],
        scope: Span,
        parent: Option<usize>,
        context: &Context,
    ) {
        for stmt in body {
            self.statement(stmt, scope, parent, context);
        }
    }

    pub(super) fn nested(&mut self, stmt: &Stmt, parent: Option<usize>, context: &Context) {
        let scope = span_of(stmt.span);
        match &stmt.kind {
            StmtKind::Block(block) => self.statements(&block.stmts, scope, parent, context),
            _ => self.statement(stmt, scope, parent, context),
        }
    }

    pub(super) fn statement(
        &mut self,
        stmt: &Stmt,
        scope: Span,
        parent: Option<usize>,
        context: &Context,
    ) {
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
