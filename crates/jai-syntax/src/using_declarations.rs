//! A declaration promotion retains its original child and target name token.
use super::*;

impl Statement {
    /// The declared source symbol, without reserving a second identity.
    pub fn declared_name(&self) -> Option<Symbol> {
        match &self.kind {
            StatementKind::Declare(value) => Some(value.name()),
            StatementKind::Constant(value) => Some(value.name),
            StatementKind::Procedure(value) => Some(value.name),
            StatementKind::ProcedurePrototype(value) => Some(value.name),
            StatementKind::Record(value) => Some(value.name),
            StatementKind::Enum(value) => Some(value.name),
            StatementKind::TypeAlias(value) => Some(value.name),
            StatementKind::Library(value) => Some(value.name),
            _ => None,
        }
    }

    /// Build a target from the actual child declaration; its initializer stays owned by the child.
    pub fn using_declaration_directive(&self) -> Option<UsingDirective> {
        let StatementKind::UsingDeclaration {
            declaration,
            selection,
            target_span,
        } = &self.kind
        else {
            return None;
        };
        Some(declaration_target(
            declaration.declared_name()?,
            *target_span,
            selection,
            self.span,
        ))
    }
}

impl FileDeclaration {
    pub fn declared_name(&self) -> Symbol {
        match &self.kind {
            FileDeclarationKind::Library(value) => value.name,
            FileDeclarationKind::Procedure(value) => value.name,
            FileDeclarationKind::OperatorAlias(value) => value.name,
            FileDeclarationKind::ProcedurePrototype(value) => value.name,
            FileDeclarationKind::Global(value) => value.declaration.name(),
            FileDeclarationKind::Constant(value) => value.name,
            FileDeclarationKind::Record(value) => value.name,
            FileDeclarationKind::Enum(value) => value.name,
            FileDeclarationKind::TypeAlias(value) => value.name,
            FileDeclarationKind::Placeholder(value) => value.name,
        }
    }
}

impl FileItem {
    pub fn using_declaration_directive(&self) -> Option<UsingDirective> {
        let FileItem::UsingDeclaration {
            declaration,
            selection,
            target_span,
            location,
        } = self
        else {
            return None;
        };
        Some(declaration_target(
            declaration.declared_name(),
            *target_span,
            selection,
            location.span,
        ))
    }
}

fn declaration_target(
    name: Symbol,
    target_span: Span,
    selection: &UsingSelection,
    span: Span,
) -> UsingDirective {
    UsingDirective {
        target: Expression {
            kind: ExpressionKind::Name(name),
            span: target_span,
        },
        selection: selection.clone(),
        span,
    }
}

impl Parser<'_> {
    pub(super) fn using_declaration_prefix(&self) -> bool {
        self.named_prefix(Punct::Colon)
            || self.named_prefix(Punct::Infer)
            || self.named_prefix(Punct::Constant)
    }

    pub(super) fn using_statement(&mut self) -> Result<StatementKind, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        let selection = self.using_selection()?;
        if !self.using_declaration_prefix() {
            return Ok(StatementKind::Using(self.using_target(start, selection)?));
        }
        let target_span = self.token().span;
        let declaration = self.statement()?;
        if declaration.declared_name().is_none() {
            return Err(Diagnostic::new(
                declaration.span,
                "using requires one named declaration",
            ));
        }
        Ok(StatementKind::UsingDeclaration {
            declaration: Box::new(declaration),
            selection,
            target_span,
        })
    }
}
