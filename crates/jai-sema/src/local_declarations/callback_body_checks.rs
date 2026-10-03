//! Source-contract revisions invalidate bodies without replacing their identity.
use super::*;

#[derive(Default)]
pub(super) struct CallbackBodyChecks {
    revision: usize,
    body_revisions: HashMap<ProcedureId, usize>,
    pending: HashSet<ProcedureId>,
}

/// An actual body identity and the source-contract revision checked by that bind.
#[derive(Clone, Copy)]
pub(crate) struct LocalBodyCheck {
    procedure: ProcedureId,
    revision: usize,
}

impl LocalDeclarationRegistry {
    pub(crate) fn callback_body_revision(&self, procedure: ProcedureId) -> Option<usize> {
        self.callback_checks.body_revisions.get(&procedure).copied()
    }

    pub(crate) fn callback_readiness_revision(&self) -> usize {
        self.callback_checks.revision
    }

    pub(crate) fn pending_callback_rechecks(&self) -> Vec<ProcedureId> {
        let mut ids = self
            .callback_checks
            .pending
            .iter()
            .copied()
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| id.index());
        ids
    }

    pub(crate) fn callback_body_ready(&self, procedure: ProcedureId) -> Option<bool> {
        self.callback_checks
            .body_revisions
            .contains_key(&procedure)
            .then(|| {
                !self.callback_checks.pending.contains(&procedure)
                    && self.procedures.contains_key(&procedure)
            })
    }

    pub(crate) fn callback_body_failure(&self, procedure: ProcedureId) -> Option<&Diagnostic> {
        self.generic_procedures.callback_failure(procedure)
    }

    pub(crate) fn begin_callback_body_check(
        &mut self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<LocalBodyCheck, Diagnostic> {
        if !self.signatures.contains_key(&procedure) {
            return Err(Diagnostic::new(
                span,
                "local body check has no actual source signature",
            ));
        }
        let revision = *self
            .callback_checks
            .body_revisions
            .entry(procedure)
            .or_default();
        Ok(LocalBodyCheck {
            procedure,
            revision,
        })
    }

    /// The caller has already proved a strict strengthening of source contracts.
    /// A first contract publication before any body check needs no invalidation.
    pub(crate) fn invalidate_callback_body(
        &mut self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        let Some(&version) = self.callback_checks.body_revisions.get(&procedure) else {
            return Ok(false);
        };
        let revision = self
            .callback_checks
            .revision
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new(span, "local callback readiness revision exhausted"))?;
        let version = version
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new(span, "local callback body revision exhausted"))?;
        self.callback_checks.revision = revision;
        self.callback_checks
            .body_revisions
            .insert(procedure, version);
        self.callback_checks.pending.insert(procedure);
        self.procedures.remove(&procedure);
        Ok(true)
    }

    /// False retains the exact pending ID, even if a stronger contract appeared
    /// during recursive binding. Other bodies' revisions do not affect this token.
    pub(crate) fn publish_callback_body(
        &mut self,
        token: LocalBodyCheck,
        procedure: Procedure,
        span: Span,
    ) -> Result<bool, Diagnostic> {
        let signature = self.signatures.get(&token.procedure).ok_or_else(|| {
            Diagnostic::new(
                span,
                "local body publication has no actual source signature",
            )
        })?;
        if procedure.id != token.procedure || procedure.signature != signature.ty {
            return Err(Diagnostic::new(
                span,
                "local body publication differs from its checked signature",
            ));
        }
        if self.callback_checks.body_revisions.get(&token.procedure) != Some(&token.revision) {
            self.callback_checks.pending.insert(token.procedure);
            self.procedures.remove(&token.procedure);
            return Ok(false);
        }
        self.procedures.insert(procedure.id, procedure);
        self.callback_checks.pending.remove(&token.procedure);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (LocalDeclarationRegistry, Procedure, Procedure) {
        let mut types = jai_types::TypeRegistry::new();
        let ty = types
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let mut registry = LocalDeclarationRegistry::default();
        let first = body(ProcedureId::new(4), ty);
        let second = body(ProcedureId::new(7), ty);
        for procedure in [&first, &second] {
            registry.signatures.insert(
                procedure.id,
                Signature {
                    ty,
                    id: procedure.id,
                    parameters: vec![],
                    source_variadic: crate::overloads::CandidateVariadic::None,
                    results: vec![],
                },
            );
        }
        (registry, first, second)
    }

    fn body(id: ProcedureId, signature: TypeId) -> Procedure {
        Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
        }
    }

    #[test]
    fn a_contract_before_first_binding_does_not_invalidate_a_body() {
        let (mut registry, first, _) = fixture();
        assert!(
            !registry
                .invalidate_callback_body(first.id, Span::default())
                .unwrap()
        );
        assert_eq!(registry.callback_readiness_revision(), 0);
        assert_eq!(registry.callback_body_ready(first.id), None);
        assert!(registry.pending_callback_rechecks().is_empty());
        let token = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        assert!(
            registry
                .publish_callback_body(token, first.clone(), Span::default())
                .unwrap()
        );
        assert_eq!(registry.callback_body_ready(first.id), Some(true));
    }

    #[test]
    fn unrelated_contract_changes_do_not_reject_a_checked_body() {
        let (mut registry, first, second) = fixture();
        let first_check = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        let second_check = registry
            .begin_callback_body_check(second.id, Span::default())
            .unwrap();
        assert!(
            registry
                .publish_callback_body(second_check, second.clone(), Span::default())
                .unwrap()
        );
        assert!(
            registry
                .invalidate_callback_body(second.id, Span::default())
                .unwrap()
        );
        assert!(
            registry
                .publish_callback_body(first_check, first.clone(), Span::default())
                .unwrap()
        );
        assert_eq!(registry.callback_body_ready(first.id), Some(true));
        assert_eq!(registry.pending_callback_rechecks(), [second.id]);
    }

    #[test]
    fn strengthening_during_binding_rejects_the_stale_publication() {
        let (mut registry, first, _) = fixture();
        let stale = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        assert!(
            registry
                .invalidate_callback_body(first.id, Span::default())
                .unwrap()
        );
        assert!(
            !registry
                .publish_callback_body(stale, first.clone(), Span::default())
                .unwrap()
        );
        assert!(registry.ready_procedure(first.id).is_none());
        assert_eq!(registry.callback_body_ready(first.id), Some(false));
        assert_eq!(registry.pending_callback_rechecks(), [first.id]);
        let current = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        assert!(
            registry
                .publish_callback_body(current, first.clone(), Span::default())
                .unwrap()
        );
        assert_eq!(registry.callback_body_ready(first.id), Some(true));
        assert!(registry.pending_callback_rechecks().is_empty());
        assert_eq!(registry.signatures.len(), 2);
        assert_eq!(registry.ready_snapshot().len(), 1);
    }

    #[test]
    fn a_failed_recheck_keeps_pending_ids_in_allocator_order() {
        let (mut registry, first, second) = fixture();
        for procedure in [&second, &first] {
            registry
                .begin_callback_body_check(procedure.id, Span::default())
                .unwrap();
            registry
                .invalidate_callback_body(procedure.id, Span::default())
                .unwrap();
        }
        // The failing body returns without publication; starting that check
        // must not make its earlier checked IR available again.
        let _failed = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        assert_eq!(registry.pending_callback_rechecks(), [first.id, second.id]);
        assert_eq!(registry.callback_body_ready(first.id), Some(false));
        assert_eq!(registry.callback_body_ready(second.id), Some(false));
        assert_eq!(registry.callback_readiness_revision(), 2);
    }

    #[test]
    fn publication_cannot_change_the_actual_procedure_identity() {
        let (mut registry, first, second) = fixture();
        let token = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        let error = registry
            .publish_callback_body(token, second, Span::new(12, 18))
            .unwrap_err();
        assert_eq!(error.span, Span::new(12, 18));
        assert!(error.message.contains("checked signature"));
        assert!(registry.ready_procedure(first.id).is_none());
    }

    #[test]
    fn exhausted_revisions_do_not_partially_invalidate_an_actual_body() {
        let (mut registry, first, _) = fixture();
        let token = registry
            .begin_callback_body_check(first.id, Span::default())
            .unwrap();
        registry
            .publish_callback_body(token, first.clone(), Span::default())
            .unwrap();
        registry
            .callback_checks
            .body_revisions
            .insert(first.id, usize::MAX);
        let error = registry
            .invalidate_callback_body(first.id, Span::default())
            .unwrap_err();
        assert!(error.message.contains("revision exhausted"));
        assert_eq!(registry.callback_readiness_revision(), 0);
        assert_eq!(registry.callback_body_ready(first.id), Some(true));
        assert!(registry.pending_callback_rechecks().is_empty());
    }
}
