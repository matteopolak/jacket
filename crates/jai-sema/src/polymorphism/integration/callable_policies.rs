//! Source callback policies are collected before borrowing the specialization queue.
use crate::Resolver;
use crate::overloads::{Argument, Candidate, CandidateVariadic, Match};
use crate::polymorphism::CallablePolicyBinding;
use jai_source::{Diagnostic, Span};
use jai_syntax as syntax;

impl Resolver<'_> {
    pub(crate) fn refine_source_declaration_match(
        &mut self,
        candidate: &Candidate,
        source: &[syntax::CallArgument],
        arguments: &[Argument],
        mut matched: Match,
        span: Span,
    ) -> Result<Match, Diagnostic> {
        self.validate_matched_source_policies(candidate, source, &matched)?;
        self.collect_callable_policies(candidate, source, &mut matched, span)?;
        let mut matched = self.refine_declaration_match(candidate, arguments, matched, span)?;
        // A modifier can change a contextual type. The final key describes the
        // accepted bindings, rather than the initially inferred context.
        self.collect_callable_policies(candidate, source, &mut matched, span)?;
        Ok(matched)
    }

    pub(crate) fn validate_matched_source_policies<Origin>(
        &mut self,
        candidate: &Candidate<Origin>,
        source: &[syntax::CallArgument],
        matched: &Match<Origin>,
    ) -> Result<(), Diagnostic> {
        for binding in &matched.bindings {
            let parameter = &candidate.parameters[binding.parameter];
            let argument = &source[binding.argument].value;
            if parameter.baking == syntax::ParameterBaking::Optional
                && self.optional_baking_needs_materialization(argument)?
            {
                return Err(Diagnostic::new(
                    argument.span,
                    "optional baking requires selected constant materialization",
                ));
            }
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                self.validate_discarded_expression(argument)?;
            }
        }
        Ok(())
    }

    fn collect_callable_policies(
        &mut self,
        candidate: &Candidate,
        source: &[syntax::CallArgument],
        matched: &mut Match,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                span,
                "callable specialization requires a defining module scope",
            )
        })?;
        if scope.concrete_signature(matched.declaration).is_some() {
            return Ok(());
        }
        let mut policies = Vec::new();
        for (parameter_index, parameter) in candidate.parameters.iter().enumerate() {
            let target = scope.materialize_pattern(
                &parameter.ty,
                &matched.substitution,
                self.types,
                &mut self.meta.record_specializations,
                span,
            )?;
            for (occurrence, binding) in matched
                .bindings
                .iter()
                .filter(|binding| binding.parameter == parameter_index)
                .enumerate()
            {
                let argument = source.get(binding.argument).ok_or_else(|| {
                    Diagnostic::new(span, "callable policy has no original source argument")
                })?;
                let policy = if parameter.is_baked(&matched.substitution) {
                    match matched
                        .substitution
                        .constant(parameter.name)
                        .and_then(crate::polymorphism::BakedValue::as_type)
                    {
                        Some(actual) => {
                            match crate::procedure_values::contracts::callback_source_type(
                                &argument.value,
                            ) {
                                Some(annotation) => self.specialization_type_callable_policy(
                                    &annotation,
                                    actual,
                                    argument.value.span,
                                )?,
                                None => None,
                            }
                        }
                        None => self.specialization_callable_policy(
                            &argument.value,
                            target,
                            argument.value.span,
                        )?,
                    }
                } else {
                    let expected = if matches!(
                        candidate.variadic,
                        CandidateVariadic::Jai { parameter } if parameter == parameter_index
                    ) && (argument.spread || argument.name.is_some())
                    {
                        self.types.slice(target).map_err(|error| {
                            Diagnostic::new(argument.value.span, error.to_string())
                        })?
                    } else {
                        target
                    };
                    self.specialization_callable_policy(
                        &argument.value,
                        expected,
                        argument.value.span,
                    )?
                };
                if let Some(policy) = policy {
                    policies.push(CallablePolicyBinding {
                        name: parameter.name,
                        occurrence,
                        policy,
                    });
                }
            }
        }
        matched.substitution.callables = policies;
        Ok(())
    }
}
