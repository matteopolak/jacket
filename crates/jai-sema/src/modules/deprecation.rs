//! A specialization adopts attributes from its actual selected source declaration.
use super::*;
impl FileScope<'_> {
    pub(crate) fn remember_deprecation(
        &self,
        meta: &mut crate::reflection::MetaContext,
        procedure: ProcedureId,
        declaration: DeclarationId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let declaration = self
            .declarations
            .graph
            .declaration(declaration)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "deprecated procedure declaration identity is unavailable",
                )
            })?;
        let attribute = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(source) => source.deprecation.as_ref(),
            FileDeclarationKind::ProcedurePrototype(source) => source.deprecation.as_ref(),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "deprecation source is not a procedure declaration",
                ));
            }
        };
        let location = declaration.location();
        let source = self
            .source_record(location.source)
            .ok_or_else(|| Diagnostic::new(span, "deprecation source record is not retained"))?;
        meta.remember_deprecation(
            crate::deprecation_warnings::DeprecationKey::Procedure(procedure),
            source,
            self.declarations.graph.symbols().name(declaration.name()),
            attribute,
            location.span,
        )
    }
}
impl Resolver<'_> {
    pub(crate) fn register_graph_deprecation(
        &mut self,
        procedure: ProcedureId,
        declaration: DeclarationId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.graph_scope
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "deprecation registration requires a defining module scope",
                )
            })?
            .remember_deprecation(self.meta, procedure, declaration, span)
    }
}
