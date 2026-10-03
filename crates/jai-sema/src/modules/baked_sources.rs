//! Checked procedure aliases keep their actual source definition and file.
use super::*;
use crate::procedure_values::baked_arguments::{BakedCallableOrigin, BakedProcedureTarget};

impl FileScope<'_> {
    pub(crate) fn preview_baked_generic_header(
        &self,
        matched: &crate::overloads::Match,
        supplied: &[usize],
        types: &mut TypeRegistry,
        records: &mut aggregates::parameterized::RecordSpecializations,
        span: Span,
    ) -> Result<crate::procedure_values::bindings::CallbackSignature, Diagnostic> {
        self.declarations.generics.borrow().preview_baked_header(
            matched,
            supplied,
            types,
            span,
            &mut |types, declaration, substitution| {
                self.materialize_generic_record(declaration, substitution, types, records, span)
            },
        )
    }
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
        let parameters = match &source.syntax().kind {
            FileDeclarationKind::Procedure(source) => &source.parameters,
            FileDeclarationKind::ProcedurePrototype(source) => &source.parameters,
            _ => return None,
        };
        let source_formals = signature
            .parameters
            .iter()
            .map(|parameter| {
                parameters
                    .iter()
                    .position(|source| source.name == parameter.name)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(BakedProcedureTarget {
            origin: BakedCallableOrigin::Module {
                declaration,
                file: source.file(),
            },
            source: source.location(),
            metadata: crate::procedure_values::bindings::CallbackSignature::source(&signature),
            signature,
            source_formals,
            source_arguments: None,
        })
    }
}
