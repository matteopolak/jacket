//! Original callable source facts follow the actual checked procedure identity.
use super::*;
use crate::local_declarations::ProcedureSourceIdentity;

impl FileScope<'_> {
    pub(crate) fn procedure_source_identity(
        &self,
        procedure: ProcedureId,
    ) -> Option<ProcedureSourceIdentity> {
        let direct = self
            .declarations
            .signatures
            .iter()
            .find_map(|(id, signature)| {
                if signature.id != procedure {
                    return None;
                }
                let declaration = self.declarations.graph.declaration(*id)?;
                matches!(
                    declaration.syntax().kind,
                    FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
                )
                .then_some((*id, declaration.file(), signature.ty))
            });
        let (id, file, ty) = direct.or_else(|| {
            let generics = self.declarations.generics.borrow();
            let (id, file) = generics.callback_source_origin(procedure)?;
            let signature = generics.callback_signature(procedure)?;
            Some((id, file, signature.ty))
        })?;
        let declaration = self.declarations.graph.declaration(id)?;
        if !matches!(
            declaration.syntax().kind,
            FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
        ) {
            return None;
        }
        Some(ProcedureSourceIdentity {
            name: Some(declaration.name()),
            file: Some(file),
            location: declaration.location(),
            ty,
        })
    }
}
