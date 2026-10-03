//! Dependent lambda contexts use argument type facts before body specialization.
use crate::overloads::{Argument, ArgumentInfo, ArgumentType, Candidate, TypePattern};
use crate::{Resolver, polymorphism::Substitution};
use jai_source::{Diagnostic, Span};
use jai_syntax as syntax;
use jai_types::TypeKind;

impl Resolver<'_> {
    pub(crate) fn describe_candidate_callbacks(
        &mut self,
        candidate: &Candidate,
        source: &[syntax::CallArgument],
        arguments: &[Argument],
        span: Span,
    ) -> Result<Vec<Argument>, Diagnostic> {
        let mut arguments = arguments.to_vec();
        if !arguments
            .iter()
            .any(|argument| matches!(argument.info.ty, ArgumentType::ContextualProcedure { .. }))
        {
            return Ok(arguments);
        }
        let prepared = crate::overloads::prepare_candidate_arguments(
            self.types, self, candidate, &arguments, span,
        )?;
        let mut substitution = prepared.substitution;
        for binding in prepared.bindings {
            if !matches!(
                arguments[binding.argument].info.ty,
                ArgumentType::ContextualProcedure { .. }
            ) {
                continue;
            }
            let lambda = match &source[binding.argument].value.kind {
                syntax::ExpressionKind::ShortLambda(lambda) => Some(lambda),
                _ => None,
            };
            let named = match &source[binding.argument].value.kind {
                syntax::ExpressionKind::Name(name) => Some(syntax::NamePath {
                    root: *name,
                    members: vec![],
                }),
                syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
                _ => None,
            };
            if lambda.is_none() && named.is_none() {
                continue;
            }
            let argument_span = arguments[binding.argument].span;
            let parameter = &candidate.parameters[binding.parameter];
            if parameter.baking == syntax::ParameterBaking::Required {
                return Err(Diagnostic::new(
                    argument_span,
                    "a baked callback requires a checked procedure identity",
                ));
            }
            let target = match &parameter.ty {
                TypePattern::Procedure(pattern)
                    if !pattern.results.iter().all(|result| {
                        crate::overloads::pattern_bindings_ready(
                            result,
                            &substitution,
                            argument_span,
                        )
                    }) =>
                {
                    let mut parameter_pattern = pattern.as_ref().clone();
                    parameter_pattern.results.clear();
                    let parameter_type = self.materialize_callback_pattern(
                        &TypePattern::Procedure(Box::new(parameter_pattern)),
                        &substitution,
                        argument_span,
                    )?;
                    let parameters = self
                        .types
                        .procedure_definition(parameter_type)
                        .map_err(|error| Diagnostic::new(argument_span, error.to_string()))?
                        .clone();
                    let result = if let Some(lambda) = lambda {
                        self.preview_inferred_short_lambda_result(
                            lambda,
                            &parameters,
                            argument_span,
                        )?
                    } else {
                        self.preview_inferred_named_short_lambda_result(
                            named.as_ref().expect("named lambda source"),
                            &parameters,
                            argument_span,
                        )?
                        .ok_or_else(|| {
                            Diagnostic::new(
                                argument_span,
                                "named callback source is not a short lambda",
                            )
                        })?
                    };
                    crate::overloads::infer_lambda_result(
                        self.types,
                        self,
                        pattern,
                        result.as_ref(),
                        &mut substitution,
                        lambda.map_or(argument_span, |lambda| lambda.body.span),
                    )?;
                    self.materialize_callback_pattern(&parameter.ty, &substitution, argument_span)?
                }
                _ => {
                    self.materialize_callback_pattern(&parameter.ty, &substitution, argument_span)?
                }
            };
            if !matches!(self.types.kind(target), Ok(TypeKind::Procedure(_))) {
                return Err(Diagnostic::new(
                    argument_span,
                    "short lambda requires a callback parameter type",
                ));
            }
            arguments[binding.argument].info = if let Some(lambda) = lambda {
                self.preview_short_lambda(lambda, target, argument_span)?;
                ArgumentInfo {
                    ty: ArgumentType::ContextualProcedure {
                        compatible_signatures: vec![target].into_boxed_slice(),
                    },
                    constant: None,
                }
            } else {
                self.describe_contextual_named_short_lambda(
                    named.as_ref().expect("named lambda source"),
                    &[target],
                    argument_span,
                )?
                .ok_or_else(|| {
                    Diagnostic::new(argument_span, "named callback source is not a short lambda")
                })?
            };
        }
        Ok(arguments)
    }

    fn materialize_callback_pattern(
        &mut self,
        pattern: &TypePattern,
        substitution: &Substitution,
        span: Span,
    ) -> Result<jai_types::TypeId, Diagnostic> {
        let scope = self
            .graph_scope
            .ok_or_else(|| Diagnostic::new(span, "callback pattern requires a definition scope"))?;
        scope.materialize_pattern(
            pattern,
            substitution,
            self.types,
            &mut self.meta.record_specializations,
            span,
        )
    }
}
