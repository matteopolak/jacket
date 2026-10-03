//! Execute checked modifier jobs before final candidate ranking or reservation.
use super::ModifierReadiness;
use super::modifier_jobs::ModifierBody;
use crate::Resolver;
use crate::Signature;
use crate::compile_time::ReadyProcedures;
use crate::modifiers::{ModifierOutcome, ModifierPlan, ModifierSlot};
use crate::overloads::{Argument, Candidate, Match};
use crate::polymorphism::{BakedValue, Substitution};
use jai_ir::{Call, ParameterId, ValueExpr};
use jai_source::{Diagnostic, Span};

pub(crate) enum ModifierExecution {
    Accepted(Substitution),
    Pending(Vec<jai_vm::Dependency>),
    Failed(Diagnostic),
}

impl Resolver<'_> {
    /// The source identity and initial substitution select one auxiliary job.
    /// Only accepted final bindings reach ordinary specialization reservation.
    pub(crate) fn refine_declaration_match(
        &mut self,
        candidate: &Candidate,
        arguments: &[Argument],
        mut matched: Match,
        span: Span,
    ) -> Result<Match, Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "modifier requires a defining module scope"))?;
        let Some(source) = scope.modifier_source(matched.declaration) else {
            return Ok(matched);
        };
        let context = self
            .compile_time
            .ok_or_else(|| Diagnostic::new(span, "modifier requires checked body readiness"))?;
        self.ensure_runtime_type_storage(self.types.meta_type(), source.span)?;
        let readiness = context
            .generics
            .borrow_mut()
            .request_modifier(source, &matched, self.types, span)?;
        let substitution = match readiness {
            ModifierReadiness::Pending(id) => {
                context.record_pending(vec![jai_vm::Dependency::Procedure(id)]);
                return Err(Diagnostic::new(
                    span,
                    "specialization modifier body is pending",
                ));
            }
            ModifierReadiness::Finished(result) => result?,
            ModifierReadiness::BodyReady(body) => match self.execute_modifier_body(&body, span) {
                super::ModifierExecution::Accepted(substitution) => substitution,
                super::ModifierExecution::Pending(dependencies) => {
                    context.record_pending(dependencies);
                    return Err(Diagnostic::new(
                        span,
                        "specialization modifier dependencies are pending",
                    ));
                }
                super::ModifierExecution::Failed(error) => return Err(error),
            },
        };
        matched.substitution = substitution;
        crate::overloads::recheck_match_with_nominals(
            self.types, self, candidate, arguments, matched, span,
        )
    }

    /// Execute an actual record or procedure auxiliary job without an invented
    /// overload match. Pending execution publishes neither bindings nor effects.
    pub(crate) fn execute_modifier_body(
        &mut self,
        body: &ModifierBody,
        span: Span,
    ) -> ModifierExecution {
        let outcome = match self.evaluate_modifier_body(body, span) {
            Ok(ModifierOutcome::Pending(dependencies)) => {
                return ModifierExecution::Pending(dependencies);
            }
            Ok(ModifierOutcome::Accepted(substitution)) => Ok(substitution),
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
        if let Some(context) = self.compile_time
            && let Err(error) = context
                .generics
                .borrow_mut()
                .finish_modifier(&body.key, outcome.clone())
        {
            return ModifierExecution::Failed(error);
        }
        match outcome {
            Ok(substitution) => ModifierExecution::Accepted(substitution),
            Err(error) => ModifierExecution::Failed(error),
        }
    }

    fn evaluate_modifier_body(
        &mut self,
        body: &ModifierBody,
        span: Span,
    ) -> Result<ModifierOutcome, Diagnostic> {
        self.evaluate_checked_modifier_parts(&body.signature, &body.plan, &body.initial, span)
    }

    /// Evaluate a checked auxiliary procedure using its actual signature and slots.
    pub(crate) fn evaluate_checked_modifier_parts(
        &mut self,
        signature: &Signature,
        plan: &ModifierPlan,
        initial: &Substitution,
        span: Span,
    ) -> Result<ModifierOutcome, Diagnostic> {
        let context = self
            .compile_time
            .ok_or_else(|| Diagnostic::new(span, "modifier requires checked body readiness"))?;
        let parameters: Vec<_> = signature
            .parameters
            .iter()
            .filter(|parameter| parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate)
            .collect();
        if parameters.len() != plan.slots().len() {
            return Err(Diagnostic::new(
                span,
                "modifier evaluated parameters do not match its checked slot plan",
            ));
        }
        let mut values = Vec::new();
        for (index, (slot, parameter)) in plan.slots().iter().zip(parameters).enumerate() {
            let ty = parameter.ty;
            let value = match *slot {
                ModifierSlot::Type { name } => match initial.ty(name) {
                    Some(represented) => {
                        let expression =
                            self.runtime_type_expression(crate::Expr::Type(represented), span)?;
                        self.coerce_value(expression, ty, span)?
                    }
                    // Accepted result introductions require canonical descriptors;
                    // a rejected execution never decodes its initially empty slots.
                    None => ValueExpr::Zero(ty),
                },
                ModifierSlot::Baked { name, .. } => {
                    let value = initial
                        .constant(name)
                        .cloned()
                        .ok_or_else(|| Diagnostic::new(span, "modifier input binding is absent"))?;
                    if matches!(value, BakedValue::Type(_) | BakedValue::Code(_)) {
                        return Err(Diagnostic::new(
                            span,
                            "modifier baked slot is not a runtime value",
                        ));
                    }
                    value
                        .into_runtime(ty, self.types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .into_expression()
                }
            };
            values.push((ParameterId::new(index), value));
        }
        let call = Call::new(signature.id, values);
        let mut signatures = context.signatures.clone();
        signatures.extend(context.generics.borrow().signature_snapshot());
        signatures.extend(self.meta.local_declarations.signature_snapshot());
        let mut procedures = context.procedures.clone();
        procedures.extend(self.meta.local_declarations.ready_snapshot());
        let bindings = crate::compile_time::prototypes::snapshot(context, self.meta);
        let places = self.places.snapshot();
        let globals = self.meta.external_globals.snapshot(context.globals)?;
        let provider = ReadyProcedures::new_with_context(
            self.types,
            &procedures,
            &signatures,
            &globals,
            &places,
            context.context,
        )
        .map(|provider| {
            provider
                .with_foreign(&bindings.foreign)
                .with_storage_alignments(&self.meta.storage_alignments)
                .with_pending_global_alignments(context.pending_global_alignments)
                .with_generic_readiness(context.generics)
                .with_local_readiness(&self.meta.local_declarations)
        })
        .and_then(|provider| provider.with_compiler(context.compiler))
        .and_then(|provider| provider.with_runtime(&bindings.runtime))
        .and_then(|provider| provider.with_file_abi(&bindings.file_abi))
        .and_then(|provider| provider.with_heap_abi(&bindings.heap_abi))
        .and_then(|provider| provider.with_process_abi(&bindings.process_abi))
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let mut vm = jai_vm::Vm::new_with_target(
            &provider,
            jai_vm::NoEffects,
            context.limits,
            context.target.unwrap_or_default(),
        )
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(crate::modifiers::execute(
            &mut vm,
            &call,
            plan,
            initial,
            self.types,
            context.limits.evaluation_depth,
        ))
    }
}
