//! Constant policy lookup retains an actual graph declaration identity.
use super::*;

impl FileScope<'_> {
    pub(crate) fn baked_constant_contract_revision(&self, owner: ProcedureId) -> Option<usize> {
        self.declarations
            .generics
            .borrow()
            .callback_body_revision(owner)
    }

    pub(crate) fn baked_constant_contract_failure(&self, owner: ProcedureId) -> Option<Diagnostic> {
        self.declarations
            .generics
            .borrow()
            .callback_body_failure(owner)
            .cloned()
    }
    pub(crate) fn baked_constant_declaration(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<DeclarationId, Diagnostic> {
        let id = declaration_id(self.declarations.graph, self.file, path, span)?;
        if !matches!(
            self.declarations
                .graph
                .declaration(id)
                .map(|declaration| &declaration.syntax().kind),
            Some(syntax::FileDeclarationKind::Constant(_))
        ) {
            return Err(Diagnostic::new(
                span,
                "partial constant policy requires its original constant declaration",
            ));
        }
        Ok(id)
    }
}
