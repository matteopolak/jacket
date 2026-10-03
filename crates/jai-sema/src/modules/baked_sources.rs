//! Checked procedure aliases keep their actual source definition and file.
use super::*;
use crate::procedure_values::baked_arguments::{BakedCallableOrigin, BakedProcedureTarget};

impl FileScope<'_> {
    pub(crate) fn baked_procedure_target(
        &self,
        procedure: ProcedureId,
    ) -> Option<BakedProcedureTarget> {
        let signature = self.callback_procedure_signature(procedure)?;
        let declaration = self
            .declarations
            .signatures
            .iter()
            .find_map(|(&id, signature)| {
                if signature.id != procedure {
                    return None;
                }
                let declaration = self.declarations.graph.declaration(id)?;
                matches!(
                    declaration.syntax().kind,
                    FileDeclarationKind::Procedure(_) | FileDeclarationKind::ProcedurePrototype(_)
                )
                .then_some(id)
            })
            .or_else(|| {
                self.declarations
                    .generics
                    .borrow()
                    .callback_source_origin(procedure)
                    .map(|(id, _)| id)
            })?;
        let source = self.declarations.graph.declaration(declaration)?;
        Some(BakedProcedureTarget {
            origin: BakedCallableOrigin::Module {
                declaration,
                file: source.file(),
            },
            source: source.location(),
            signature,
        })
    }
}
