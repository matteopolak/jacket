//! Per-use constant policies bind to an actual local declaration, never a name.
use super::*;

impl Resolver<'_> {
    pub(crate) fn local_baked_constant_origin(
        &self,
        source: &syntax::ConstantDeclaration,
    ) -> Option<LocalDeclarationId> {
        let declaration = self
            .local_scopes
            .frames
            .last()?
            .declarations
            .get(&source.name)?;
        let DeclarationSyntax::Constant(original) = &declaration.syntax else {
            return None;
        };
        (original.span == source.span && original.initializer.span == source.initializer.span)
            .then_some(declaration.id)
    }
}
