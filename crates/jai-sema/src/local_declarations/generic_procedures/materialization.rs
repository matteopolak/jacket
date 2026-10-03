//! Selected source contracts precede same-ID lexical body publication.
use super::*;

impl Resolver<'_> {
    pub(crate) fn materialize_local_operator_match_with_contracts(
        &mut self,
        matched: Match<LocalDeclarationId>,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Signature, Diagnostic> {
        let key = Key {
            declaration: matched.declaration,
            substitution: matched.substitution.clone(),
        };
        if let Some(error) = self
            .meta
            .local_declarations
            .generic_procedures
            .failures
            .get(&key)
        {
            return Err(error.clone());
        }
        let definition = self.local_operator_definition(matched.declaration, span)?;
        let previous = self
            .meta
            .local_declarations
            .generic_procedures
            .signatures
            .get(&key)
            .cloned();
        let procedure = match previous.as_ref() {
            Some(signature) => signature.id,
            None => match self
                .meta
                .local_declarations
                .generic_procedures
                .reservations
                .get(&key)
            {
                Some(&procedure) => procedure,
                None => {
                    let procedure = self.reserve_generated_procedure(span)?;
                    self.meta
                        .local_declarations
                        .generic_procedures
                        .reservations
                        .insert(key.clone(), procedure);
                    procedure
                }
            },
        };
        let phase = self.selected_local_procedure_phase(matched.declaration, procedure);
        let readiness = if phase == MethodPhase::TypesOnly {
            HeaderReadiness::TypesOnly
        } else {
            HeaderReadiness::Complete
        };
        let signature = match previous {
            Some(signature)
                if self
                    .meta
                    .local_declarations
                    .header_readiness
                    .get(&procedure)
                    == Some(&HeaderReadiness::Complete)
                    || readiness == HeaderReadiness::TypesOnly =>
            {
                signature
            }
            _ => {
                let signature = self.with_local_procedure_substitution(
                    &definition.environment,
                    &matched.substitution,
                    definition.source.span,
                    |resolver| {
                        resolver.local_operator_signature(
                            procedure,
                            &definition,
                            &matched.substitution,
                            readiness,
                        )
                    },
                )?;
                self.meta
                    .local_declarations
                    .signatures
                    .insert(procedure, signature.clone());
                self.meta
                    .local_declarations
                    .header_readiness
                    .insert(procedure, readiness);
                self.meta
                    .local_declarations
                    .generic_procedures
                    .signatures
                    .insert(key.clone(), signature.clone());
                signature
            }
        };
        // Result-use contracts strengthen the source proof without changing
        // executable policy identity or the selected procedure's ABI.
        if self.local_operator_argument_contracts(
            &definition,
            &matched,
            &signature,
            arguments,
            span,
        )? {
            self.meta
                .local_declarations
                .invalidate_callback_body(signature.id, span)?;
            self.meta.debug_sources.clear_procedure(signature.id);
            self.meta.storage_alignments.clear_procedure(signature.id);
        }
        if self
            .meta
            .local_declarations
            .generic_procedures
            .active
            .contains(&key)
            || (self
                .meta
                .local_declarations
                .ready_procedure(signature.id)
                .is_some()
                && self
                    .meta
                    .local_declarations
                    .callback_body_ready(signature.id)
                    != Some(false))
        {
            return Ok(signature);
        }
        self.meta
            .local_declarations
            .generic_procedures
            .active
            .insert(key.clone());
        let result = loop {
            let revision = self.meta.local_declarations.callback_readiness_revision();
            let result = self.with_local_procedure_substitution(
                &definition.environment,
                &matched.substitution,
                definition.source.span,
                |resolver| {
                    resolver.remember_local_procedure_origin(
                        matched.declaration,
                        signature.id,
                        definition.source.span,
                    );
                    resolver.define_selected_local_procedure(
                        matched.declaration,
                        &definition.source,
                        signature.clone(),
                    )
                },
            );
            if result.is_ok()
                && phase == MethodPhase::Bodies
                && self
                    .meta
                    .local_declarations
                    .callback_body_ready(signature.id)
                    == Some(false)
                && revision != self.meta.local_declarations.callback_readiness_revision()
            {
                continue;
            }
            break result;
        };
        self.meta
            .local_declarations
            .generic_procedures
            .active
            .remove(&key);
        if let Err(error) = result {
            if !self
                .compile_time
                .is_some_and(|context| !context.pending.borrow().is_empty())
            {
                self.meta
                    .local_declarations
                    .generic_procedures
                    .failures
                    .insert(key, error.clone());
            }
            return Err(error);
        }
        Ok(signature)
    }
}
