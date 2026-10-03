//! Retain real checked temporary owners without publishing runtime procedures.
use super::*;

impl crate::Resolver<'_> {
    pub(super) fn remember_anonymous_owner(
        &mut self,
        procedure: &Procedure,
        signatures: &HashMap<ProcedureId, TypeId>,
        globals: &[Global],
        places: &jai_ir::SourceProcedurePlaces,
        location: SourceSpan,
    ) -> Result<(), jai_source::Diagnostic> {
        if !self.meta.external_globals.has_owner(procedure.id)
            && !self
                .meta
                .local_declarations
                .has_foreign_library_owner(procedure.id)
        {
            return Ok(());
        }
        let fail = |message: String| jai_source::Diagnostic::at_source(location, message);
        let source = self
            .graph_scope
            .and_then(|scope| scope.source_record(location.source))
            .ok_or_else(|| fail("temporary procedure owner requires retained source".into()))?;
        let checked = jai_ir::verify_procedure_with_context(
            self.types,
            procedure,
            signatures,
            globals,
            places.places(),
            self.context.map(|schema| &schema.definition),
        )
        .map_err(|error| fail(error.to_string()))?;
        if let Some(previous) = self.meta.source_procedure_owners.get(procedure.id) {
            if previous.signature() != procedure.signature
                || !previous.identity().matches_source(source, location)
            {
                return Err(fail(
                    "temporary procedure owner changed its checked identity".into(),
                ));
            }
            return Ok(());
        }
        let prefix = self
            .meta
            .external_globals
            .checked_file_prefix(self.types, signatures)
            .map_err(|error| error.with_fallback_source(location.source))?;
        let identity = jai_ir::SourceProcedureIdentity::new(source, location)
            .map_err(|error| fail(error.to_string()))?;
        let owner = jai_ir::CheckedSourceProcedureOwner::new(
            checked,
            jai_types::ProcedureExecution::CompileTimeOnly,
            identity,
            prefix,
            places.clone(),
        )
        .map_err(|error| fail(error.to_string()))?;
        self.meta
            .source_procedure_owners
            .insert(owner)
            .map_err(|error| fail(error.to_string()))
    }
}
