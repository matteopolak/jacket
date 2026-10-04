//! Execute the real child initializer once, then alias checked target storage.
use super::*;

impl Resolver<'_> {
    pub(crate) fn imported_storage_member_value(
        &mut self,
        id: jai_modules::SourceStorageMemberId,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "imported storage members require a source graph")
        })?;
        let (binding, fields) = scope.imported_storage_member_source(id, span)?;
        let Binding::Storage(storage) = binding else {
            return Err(Diagnostic::new(
                span,
                "imported storage member has no physical owner",
            ));
        };
        let mut place = storage.place();
        for field in fields {
            if matches!(
                self.types.kind(place.ty()),
                Ok(jai_types::TypeKind::Pointer(_))
            ) {
                return Err(Diagnostic::new(
                    span,
                    "file using through mutable pointer storage requires an initialized pointer capture",
                ));
            }
            place = self.member_place(place, field, span)?;
        }
        Storage::from_place(place, self.types)
            .map(Binding::Storage)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }

    pub(crate) fn using_declaration(
        &mut self,
        wrapper: &syntax::Statement,
    ) -> Result<Statement, Diagnostic> {
        let syntax::StatementKind::UsingDeclaration {
            declaration,
            target_span,
            ..
        } = &wrapper.kind
        else {
            return Err(Diagnostic::new(
                wrapper.span,
                "expected a using declaration",
            ));
        };
        let directive = wrapper
            .using_declaration_directive()
            .ok_or_else(|| Diagnostic::new(wrapper.span, "using requires one named declaration"))?;
        let name = declaration
            .declared_name()
            .ok_or_else(|| Diagnostic::new(*target_span, "using requires one named declaration"))?;
        if self.symbols.name(name) == "_" {
            return Err(Diagnostic::new(
                *target_span,
                "using declaration requires a stored or named target",
            ));
        }
        let mut statements = Vec::with_capacity(2);
        match &declaration.kind {
            syntax::StatementKind::Declare(_) | syntax::StatementKind::Import(_) => {
                statements.push(self.statement(declaration)?);
                self.debug.attach_completed_statement_in_block(
                    &[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::Block)],
                    0,
                );
            }
            syntax::StatementKind::Constant(constant) => {
                if matches!(
                    constant.initializer.kind,
                    syntax::ExpressionKind::ShortLambda(_)
                ) {
                    self.remember_local_constant_source(constant.name);
                } else {
                    self.resolve_local_name(name, *target_span)?;
                }
            }
            syntax::StatementKind::Procedure(_)
            | syntax::StatementKind::ProcedurePrototype(_)
            | syntax::StatementKind::Library(_)
            | syntax::StatementKind::Record(_)
            | syntax::StatementKind::Enum(_)
            | syntax::StatementKind::TypeAlias(_) => {
                self.resolve_local_name(name, *target_span)?;
            }
            _ => {
                return Err(Diagnostic::new(
                    declaration.span,
                    "using requires one named declaration",
                ));
            }
        }
        statements.push(self.using_directive(&directive)?);
        Ok(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        }))
    }
}
