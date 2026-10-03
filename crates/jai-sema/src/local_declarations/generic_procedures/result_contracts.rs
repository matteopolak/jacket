//! Each lexical call derives its result policy from its own checked operands.
use super::*;
use crate::procedure_values::contracts::ValueContract;

struct ContractDefinition {
    key: Key,
    definition: Definition,
    signature: Signature,
}

impl Resolver<'_> {
    fn local_operator_contract_definition(
        &self,
        procedure: ProcedureId,
        span: Span,
    ) -> Result<Option<ContractDefinition>, Diagnostic> {
        let Some(key) = self
            .meta
            .local_declarations
            .generic_procedures
            .reservations
            .iter()
            .find(|(_, reserved)| **reserved == procedure)
            .map(|(key, _)| key.clone())
        else {
            return Ok(None);
        };
        let definition = self.local_operator_definition(key.declaration, span)?;
        let signature = self
            .meta
            .local_declarations
            .signature(procedure)
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(span, "local operator contract lacks its actual signature")
            })?;
        if signature.parameters.len() != definition.source.parameters.len()
            || signature
                .parameters
                .iter()
                .any(|parameter| parameter.evaluation != syntax::ParameterEvaluation::Evaluate)
        {
            return Err(Diagnostic::new(
                span,
                "local operator result contracts require checked source-to-runtime projection",
            ));
        }
        Ok(Some(ContractDefinition {
            key,
            definition,
            signature,
        }))
    }

    pub(crate) fn local_operator_call_result_contracts(
        &mut self,
        procedure: ProcedureId,
        arguments: &[(ParameterId, ValueExpr)],
        source_arguments: Option<&[syntax::CallArgument]>,
        span: Span,
        depth: usize,
    ) -> Result<Option<Vec<Option<ValueContract>>>, Diagnostic> {
        let Some(ContractDefinition {
            key,
            definition,
            signature,
        }) = self.local_operator_contract_definition(procedure, span)?
        else {
            return Ok(None);
        };
        let mut bindings = key
            .substitution
            .types
            .iter()
            .map(|binding| (binding.name, None))
            .collect::<HashMap<Symbol, Option<ValueContract>>>();
        let mut defaults = Vec::new();
        let arity = definition
            .source
            .operator
            .expect("operator template")
            .kind
            .arity();
        for source in &definition.source.parameters {
            let Some(annotation) = contracts::parameter_annotation(source) else {
                continue;
            };
            let index = signature
                .parameters
                .iter()
                .position(|parameter| parameter.name == source.name)
                .ok_or_else(|| {
                    Diagnostic::new(source.span, "local operator source formal is unavailable")
                })?;
            let value = arguments
                .iter()
                .find(|(id, _)| id.index() == index)
                .map(|(_, value)| value)
                .ok_or_else(|| {
                    Diagnostic::new(
                        source.span,
                        "local operator contract lacks its checked runtime argument",
                    )
                })?;
            let source_argument = source_arguments
                .and_then(|arguments| {
                    arguments
                        .iter()
                        .find(|argument| argument.name == Some(source.name))
                })
                .or_else(|| {
                    source_arguments.and_then(|source| {
                        source
                            .iter()
                            .zip(arguments)
                            .find(|(source, (id, _))| source.name.is_none() && id.index() == index)
                            .map(|(source, _)| source)
                    })
                })
                .map(|argument| &argument.value);
            if index >= arity && source_argument.is_none() {
                defaults.push((source, annotation, value));
                continue;
            }
            let contract = self.callback_argument_contract_for_source(
                source_argument,
                value,
                span,
                depth + 1,
            )?;
            self.capture_generic_callback_variables(
                annotation,
                contract.as_ref(),
                &mut bindings,
                source.span,
            )?;
        }
        self.with_local_procedure_substitution(
            &definition.environment,
            &key.substitution,
            span,
            |resolver| {
                for (source, annotation, value) in defaults {
                    let expression = match &source.binding {
                        syntax::ParameterBinding::Defaulted {
                            expression, ..
                        }
                        | syntax::ParameterBinding::DefaultedType {
                            expression, ..
                        } => Some(expression),
                        _ => None,
                    };
                    let contract = match expression.map(|expression| &expression.kind) {
                        Some(syntax::ExpressionKind::TypeCast {
                            ty, ..
                        }) => {
                            let retained = resolver.retained_callback_syntax(ty, source.span)?;
                            resolver.bound_generic_callback_contract(
                                value.type_id(resolver.types),
                                &retained,
                                &bindings,
                                source.span,
                            )?
                        }
                        _ => resolver.callback_argument_contract_for_source(
                            expression,
                            value,
                            source.span,
                            depth + 1,
                        )?,
                    };
                    resolver.capture_generic_callback_variables(
                        annotation,
                        contract.as_ref(),
                        &mut bindings,
                        source.span,
                    )?;
                }
                resolver
                    .bound_local_operator_result_contracts(&definition, &signature, &bindings)
                    .map(Some)
            },
        )
    }

    pub(crate) fn local_operator_expression_result_contracts(
        &mut self,
        source: &syntax::Expression,
        call: &Call,
        span: Span,
        depth: usize,
    ) -> Result<Option<Vec<Option<ValueContract>>>, Diagnostic> {
        let Some(ContractDefinition {
            definition,
            signature,
            ..
        }) = self.local_operator_contract_definition(call.procedure, span)?
        else {
            return Ok(None);
        };
        let operands = match &source.kind {
            syntax::ExpressionKind::Unary(_, value) => vec![value.as_ref()],
            syntax::ExpressionKind::Binary(_, left, right) => vec![left.as_ref(), right.as_ref()],
            syntax::ExpressionKind::Index {
                base,
                index,
            } => vec![base.as_ref(), index.as_ref()],
            _ => return Ok(None),
        };
        let arity = definition
            .source
            .operator
            .expect("operator template")
            .kind
            .arity();
        if operands.len() != arity || call.arguments.len() < arity {
            return Err(Diagnostic::new(
                span,
                "local operator contract lacks its original operands",
            ));
        }
        // The checked binder pushes each supplied operand in source order with
        // its selected destination ID; defaults follow those entries. Symmetry
        // changes these destination IDs, never the source-order vector.
        let mut destinations = HashSet::new();
        let mut arguments = Vec::with_capacity(arity);
        for (operand, (id, _)) in operands.into_iter().zip(&call.arguments) {
            let parameter = signature.parameters.get(id.index()).ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "local operator operand has no checked runtime destination",
                )
            })?;
            if id.index() >= arity || !destinations.insert(*id) {
                return Err(Diagnostic::new(
                    span,
                    "local operator operand destination is outside its original fixed formals",
                ));
            }
            arguments.push(syntax::CallArgument {
                name: Some(parameter.name),
                value: operand.clone(),
                spread: false,
            });
        }
        self.local_operator_call_result_contracts(
            call.procedure,
            &call.arguments,
            Some(&arguments),
            span,
            depth,
        )
    }
}
