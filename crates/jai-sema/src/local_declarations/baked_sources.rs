//! Partial application looks up the original concrete callable declaration.
use super::*;
use crate::procedure_values::baked_arguments::{BakedCallableOrigin, BakedProcedureTarget};

impl LocalDeclarationRegistry {
    pub(crate) fn baked_procedure_target(
        &self,
        procedure: ProcedureId,
    ) -> Option<BakedProcedureTarget> {
        let signature = self.signature(procedure)?.clone();
        let source = self.procedure_source_identity(procedure)?;
        let declaration = self
            .callable_declarations
            .get(&procedure)
            .copied()
            .or_else(|| {
                self.entries
                    .iter()
                    .find_map(|(&id, entry)| (entry.procedure == Some(procedure)).then_some(id))
            })
            .or_else(|| {
                self.anonymous_procedures
                    .iter()
                    .find_map(|(callable, &id)| (id == procedure).then_some(callable.declaration))
            })?;
        let source_formals = self
            .generic_procedures
            .baked_source_formals(procedure)
            .or_else(|| {
                (!self.procedure_substitutions.contains_key(&procedure))
                    .then(|| (0..signature.parameters.len()).collect())
            })?;
        Some(BakedProcedureTarget {
            origin: BakedCallableOrigin::Local(declaration),
            source: source.location,
            source_formals,
            source_arguments: None,
            metadata: crate::procedure_values::bindings::CallbackSignature::source(&signature),
            signature,
        })
    }
}
