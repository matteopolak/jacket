//! Service queued type recipes outside the type resolver's registry borrow.
use super::*;
use crate::polymorphism::integration::{ModifierExecution, ModifierReadiness};

/// The retained preparation stage supplies its actual evaluation policy.
pub(crate) struct RecordModifierPolicy {
    pub(crate) context: jai_types::ContextMode,
    pub(crate) checks: syntax::SafetyChecks,
}

pub(crate) struct RecordModifierProgress {
    pub(crate) completed: usize,
    pub(crate) pending: Vec<PendingRecordModifier>,
    pub(crate) dependencies: Vec<jai_vm::Dependency>,
}

impl crate::Resolver<'_> {
    /// Each pass examines the existing queue once. Dependency retries retain
    /// the original recipe, auxiliary procedure, and source wait sites.
    pub(crate) fn service_record_modifiers(
        &mut self,
        policy: RecordModifierPolicy,
    ) -> Result<RecordModifierProgress, LocatedDiagnostic> {
        let mut progress = RecordModifierProgress {
            completed: 0,
            pending: Vec::new(),
            dependencies: Vec::new(),
        };
        let queued = self.meta.record_specializations.modifiers.queued_count();
        if queued == 0 {
            return Ok(progress);
        }
        let location = self
            .meta
            .record_specializations
            .modifiers
            .queued_location()
            .expect("a queued source recipe has a waiter");
        let scope = self.graph_scope.ok_or_else(|| LocatedDiagnostic {
            location,
            message: "record modifier service requires its retained source graph".into(),
        })?;
        let context = self.compile_time.ok_or_else(|| LocatedDiagnostic {
            location,
            message: "record modifier service requires checked body readiness".into(),
        })?;
        let graph = scope.declarations.graph;
        for _ in 0..queued {
            let (id, intent) = self
                .meta
                .record_specializations
                .modifiers
                .next()
                .expect("bounded modifier queue has an entry");
            let span = intent.record.modify.as_ref().unwrap().span;
            let readiness = (|| {
                if !intent.initial.types.is_empty()
                    || intent
                        .initial
                        .constants
                        .iter()
                        .any(|binding| matches!(binding.value, BakedValue::Type(_)))
                {
                    self.ensure_runtime_type_storage(self.types.meta_type(), span)?;
                }
                context.generics.borrow_mut().request_modifier_body(
                    intent.key.template.0,
                    intent.file,
                    &intent.initial,
                    |procedure| {
                        modifier_source::build(
                            modifier_source::RecordModifierSource {
                                record: &intent.record,
                                initial: &intent.initial,
                                context: policy.context,
                                checks: policy.checks,
                            },
                            procedure,
                            self.types,
                        )
                    },
                )
            })();
            let outcome = match readiness {
                Ok(ModifierReadiness::Pending(procedure)) => {
                    ModifierExecution::Pending(vec![jai_vm::Dependency::Procedure(procedure)])
                }
                Ok(ModifierReadiness::BodyReady(body)) => self.execute_modifier_body(&body, span),
                Ok(ModifierReadiness::Finished(Ok(substitution))) => {
                    ModifierExecution::Accepted(substitution)
                }
                Ok(ModifierReadiness::Finished(Err(error))) | Err(error) => {
                    ModifierExecution::Failed(error)
                }
            };
            match outcome {
                ModifierExecution::Accepted(substitution) => {
                    self.meta
                        .record_specializations
                        .modifiers
                        .finish(id, Ok(substitution));
                    progress.completed += 1;
                }
                ModifierExecution::Failed(error) => {
                    let error = located(graph, intent.file, error);
                    self.meta
                        .record_specializations
                        .modifiers
                        .finish(id, Err(error.clone()));
                    return Err(error);
                }
                ModifierExecution::Pending(dependencies) => {
                    context.record_pending(dependencies.clone());
                    for dependency in dependencies {
                        if !progress.dependencies.contains(&dependency) {
                            progress.dependencies.push(dependency);
                        }
                    }
                    let location = self
                        .meta
                        .record_specializations
                        .modifiers
                        .wait_sites(id)
                        .min_by_key(|site| (site.source.index(), site.span.start, site.span.end))
                        .expect("modifier intent has a source waiter");
                    progress
                        .pending
                        .push(PendingRecordModifier { id, location });
                    self.meta.record_specializations.modifiers.retry(id);
                }
            }
        }
        Ok(progress)
    }
}
