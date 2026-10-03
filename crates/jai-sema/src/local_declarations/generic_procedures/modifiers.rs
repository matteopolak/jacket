//! Lexical modifiers keep their real source owner and checked auxiliary identity.
use super::*;
use crate::modifiers::{ModifierOutcome, ModifierPlan};

#[derive(Clone)]
struct Body {
    signature: Signature,
    source: syntax::Procedure,
    plan: ModifierPlan,
    initial: Substitution,
}

#[derive(Default)]
pub(super) struct LocalModifiers {
    bodies: HashMap<Key, Body>,
    active: HashSet<Key>,
    results: HashMap<Key, Result<Substitution, Diagnostic>>,
}

impl Resolver<'_> {
    pub(super) fn refine_local_operator_modifier(
        &mut self,
        matched: &mut Match<LocalDeclarationId>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let definition = self.local_operator_definition(matched.declaration, span)?;
        let Some(modifier) = &definition.source.modify else {
            return Ok(());
        };
        let key = Key {
            declaration: matched.declaration,
            substitution: matched.substitution.clone(),
        };
        if let Some(result) = self
            .meta
            .local_declarations
            .generic_procedures
            .modifiers
            .results
            .get(&key)
        {
            matched.substitution = result.clone()?;
            return Ok(());
        }
        self.ensure_runtime_type_storage(self.types.meta_type(), modifier.span)?;
        let body = match self
            .meta
            .local_declarations
            .generic_procedures
            .modifiers
            .bodies
            .get(&key)
            .cloned()
        {
            Some(body) => body,
            None => {
                let id = self.reserve_generated_procedure(modifier.span)?;
                let input = crate::polymorphism::integration::procedure_modifier_source(
                    &definition.source,
                    &matched.substitution,
                    self.types,
                )?;
                let (signature, source, plan) =
                    crate::polymorphism::integration::build_modifier_source(input, id, self.types)?;
                self.meta
                    .local_declarations
                    .signatures
                    .insert(id, signature.clone());
                self.meta
                    .local_declarations
                    .header_readiness
                    .insert(id, HeaderReadiness::Complete);
                let body = Body {
                    signature,
                    source,
                    plan,
                    initial: matched.substitution.clone(),
                };
                self.meta
                    .local_declarations
                    .generic_procedures
                    .modifiers
                    .bodies
                    .insert(key.clone(), body.clone());
                body
            }
        };
        if !self
            .meta
            .local_declarations
            .generic_procedures
            .modifiers
            .active
            .insert(key.clone())
        {
            return Err(Diagnostic::new(
                modifier.span,
                "recursive lexical specialization modifier",
            ));
        }
        let result = self.with_local_procedure_substitution(
            &definition.environment,
            &body.initial,
            modifier.span,
            |resolver| {
                resolver.remember_local_procedure_origin(
                    matched.declaration,
                    body.signature.id,
                    modifier.span,
                );
                resolver.with_isolated_callable_source(
                    body.signature.id,
                    modifier.span,
                    |auxiliary| {
                        auxiliary.define_selected_local_procedure(
                            matched.declaration,
                            &body.source,
                            body.signature.clone(),
                        )?;
                        auxiliary.evaluate_checked_modifier_parts(
                            &body.signature,
                            &body.plan,
                            &body.initial,
                            span,
                        )
                    },
                )
            },
        );
        self.meta
            .local_declarations
            .generic_procedures
            .modifiers
            .active
            .remove(&key);
        let result = match result {
            Ok(ModifierOutcome::Accepted(substitution)) => Ok(substitution),
            Ok(ModifierOutcome::Pending(dependencies)) => {
                let context = self.compile_time.ok_or_else(|| {
                    Diagnostic::new(span, "modifier requires checked body readiness")
                })?;
                context.record_pending(dependencies);
                return Err(Diagnostic::new(
                    span,
                    "lexical specialization modifier dependencies are pending",
                ));
            }
            Ok(ModifierOutcome::Rejected { reason }) => Err(Diagnostic::new(
                span,
                if reason.is_empty() {
                    "specialization rejected by #modify".into()
                } else {
                    reason
                },
            )),
            Ok(ModifierOutcome::Failed(error)) => Err(Diagnostic::new(span, error.to_string())),
            Err(error) => Err(error),
        };
        if !self
            .compile_time
            .is_some_and(|context| !context.pending.borrow().is_empty())
        {
            self.meta
                .local_declarations
                .generic_procedures
                .modifiers
                .results
                .insert(key, result.clone());
        }
        matched.substitution = result?;
        Ok(())
    }
}
