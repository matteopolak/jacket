//! Checked callback contracts follow actual arguments into a lexical template.
use super::*;
use crate::procedure_values::contracts::ValueContract;

impl Resolver<'_> {
    pub(super) fn local_operator_argument_contracts(
        &mut self,
        definition: &Definition,
        matched: &Match<LocalDeclarationId>,
        signature: &Signature,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<bool, Diagnostic> {
        // A newly introduced local variable must not inherit an outer
        // procedure's contract merely because its interned spelling matches.
        let mut bindings = matched
            .substitution
            .types
            .iter()
            .map(|binding| (binding.name, None))
            .collect::<HashMap<Symbol, Option<ValueContract>>>();
        let mut omitted = Vec::new();
        for (ordinal, source) in definition.source.parameters.iter().enumerate() {
            let Some(annotation) = parameter_annotation(source) else {
                continue;
            };
            let parameter = signature
                .parameters
                .iter()
                .find(|parameter| parameter.name == source.name)
                .ok_or_else(|| {
                    Diagnostic::new(source.span, "local operator source formal is unavailable")
                })?;
            let argument = matched
                .bindings
                .iter()
                .find(|binding| binding.parameter == ordinal)
                .map(|binding| {
                    arguments.get(binding.argument).ok_or_else(|| {
                        Diagnostic::new(
                            source.span,
                            "local operator contract lacks its checked source argument",
                        )
                    })
                })
                .transpose()?;
            if let Some(argument) = argument {
                let contract = self.preview_expected_callback_contract(
                    &argument.value,
                    parameter.ty,
                    argument.value.span,
                )?;
                self.capture_generic_callback_variables(
                    annotation,
                    contract.as_ref(),
                    &mut bindings,
                    argument.value.span,
                )?;
            } else {
                omitted.push((source, parameter));
            }
        }
        self.with_local_procedure_substitution(
            &definition.environment,
            &matched.substitution,
            span,
            |resolver| {
                for (source, parameter) in omitted {
                    let expression = match &source.binding {
                        syntax::ParameterBinding::Defaulted { expression, .. }
                        | syntax::ParameterBinding::DefaultedType { expression, .. } => expression,
                        _ => continue,
                    };
                    let contract = match &expression.kind {
                        syntax::ExpressionKind::TypeCast { ty, .. } => {
                            let retained =
                                resolver.retained_callback_syntax(ty, expression.span)?;
                            resolver.bound_generic_callback_contract(
                                parameter.ty,
                                &retained,
                                &bindings,
                                expression.span,
                            )?
                        }
                        _ => resolver.preview_expected_callback_contract(
                            expression,
                            parameter.ty,
                            expression.span,
                        )?,
                    };
                    resolver.capture_generic_callback_variables(
                        parameter_annotation(source).expect("omitted annotation retained"),
                        contract.as_ref(),
                        &mut bindings,
                        expression.span,
                    )?;
                }
                let mut changed = resolver.merge_generic_callback_bindings(signature.id, &bindings);
                for source in &definition.source.parameters {
                    let Some(annotation) = parameter_annotation(source) else {
                        continue;
                    };
                    let parameter = signature
                        .parameters
                        .iter()
                        .find(|parameter| parameter.name == source.name)
                        .expect("source formals were checked above");
                    let retained = resolver.retained_callback_syntax(annotation, source.span)?;
                    if let Some(contract) = resolver.bound_generic_callback_contract(
                        parameter.ty,
                        &retained,
                        &bindings,
                        source.span,
                    )? {
                        changed |= resolver.merge_generic_callback_parameter(
                            signature.id,
                            source.name,
                            contract,
                        );
                    }
                }
                let results = resolver
                    .bound_local_operator_result_contracts(definition, signature, &bindings)?;
                changed |= resolver.merge_generic_callback_results(signature.id, &results);
                Ok(changed)
            },
        )
    }

    pub(super) fn bound_local_operator_result_contracts(
        &mut self,
        definition: &Definition,
        signature: &Signature,
        bindings: &HashMap<Symbol, Option<ValueContract>>,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        let mut results = Vec::with_capacity(signature.results.len());
        for (source, result) in definition.source.results.iter().zip(&signature.results) {
            let contract = match &source.binding {
                syntax::ResultBinding::Typed { ty, .. } => {
                    let retained = self.retained_callback_syntax(ty, source.span)?;
                    self.bound_generic_callback_contract(
                        result.ty,
                        &retained,
                        bindings,
                        source.span,
                    )?
                }
                syntax::ResultBinding::InferredDefault(expression) => match &expression.kind {
                    syntax::ExpressionKind::TypeCast { ty, .. } => {
                        let retained = self.retained_callback_syntax(ty, expression.span)?;
                        self.bound_generic_callback_contract(
                            result.ty,
                            &retained,
                            bindings,
                            expression.span,
                        )?
                    }
                    _ => self.preview_expected_callback_contract(
                        expression,
                        result.ty,
                        expression.span,
                    )?,
                },
            };
            results.push(contract);
        }
        Ok(results)
    }
}

pub(super) fn parameter_annotation(source: &syntax::Parameter) -> Option<&syntax::TypeSyntax> {
    match &source.binding {
        syntax::ParameterBinding::RequiredType(ty)
        | syntax::ParameterBinding::DefaultedType { ty: Some(ty), .. } => Some(ty),
        _ => None,
    }
}
