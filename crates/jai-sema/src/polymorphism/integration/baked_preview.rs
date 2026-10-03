//! Generic partial previews reserve canonical types, never runtime procedures.
use super::*;
use crate::procedure_values::bindings::{
    CallbackArgumentPolicy, CallbackParameter, CallbackSignature,
};

impl GenericContext {
    pub(crate) fn preview_baked_header(
        &self,
        matched: &Match,
        supplied: &[usize],
        types: &mut TypeRegistry,
        span: Span,
        nominal: &mut impl FnMut(
            &mut TypeRegistry,
            DeclarationId,
            Substitution,
        ) -> Result<TypeId, super::super::SubstitutionError>,
    ) -> Result<CallbackSignature, Diagnostic> {
        let definition = self.templates.get(&matched.declaration).ok_or_else(|| {
            Diagnostic::new(span, "partial preview has no original generic definition")
        })?;
        if definition.template.candidate.variadic != CandidateVariadic::None {
            return Err(Diagnostic::new(
                span,
                "partial preview requires retained source pack binding",
            ));
        }
        let mut parameters = Vec::new();
        for (ordinal, parameter) in definition.template.candidate.parameters.iter().enumerate() {
            if supplied.contains(&ordinal) || parameter.is_baked(&matched.substitution) {
                continue;
            }
            let ty = super::super::materialize_with_nominals(
                types,
                &parameter.ty,
                &matched.substitution,
                nominal,
            )
            .map_err(|error| {
                Diagnostic::new(
                    definition.span,
                    format!("invalid partial parameter: {error:?}"),
                )
            })?;
            let default = parameter
                .default
                .as_ref()
                .map(|info| {
                    if parameter.evaluation == jai_syntax::ParameterEvaluation::Discard {
                        Ok(ParameterDefault::Discarded)
                    } else {
                        match &info.constant {
                            Some(ConstantArgument::RuntimeRead(read)) if read.ty() == ty => {
                                Ok(ParameterDefault::RuntimeRead(read.clone()))
                            }
                            Some(ConstantArgument::CodeNull)
                                if types.kind(ty).is_ok_and(|kind| {
                                    matches!(kind, jai_types::TypeKind::Code)
                                }) =>
                            {
                                Ok(ParameterDefault::CodeNull {
                                    ty,
                                })
                            }
                            Some(ConstantArgument::CallerLocation)
                                if info.ty == ArgumentType::Known(ty) =>
                            {
                                Ok(ParameterDefault::CallerLocation)
                            }
                            _ => constant_for_target(info, ty, types, definition.span)
                                .map(ParameterDefault::Constant),
                        }
                    }
                })
                .transpose()?;
            parameters.push(CallbackParameter {
                name: Some(parameter.name),
                default,
                ty,
                evaluation: parameter.evaluation,
            });
        }
        let mut results = Vec::new();
        for result in &definition.template.results {
            let ty = super::super::materialize_with_nominals(
                types,
                &result.ty,
                &matched.substitution,
                nominal,
            )
            .map_err(|error| {
                Diagnostic::new(
                    definition.span,
                    format!("invalid partial result: {error:?}"),
                )
            })?;
            let default = result
                .default
                .as_ref()
                .map(|info| constant_for_target(info, ty, types, definition.span))
                .transpose()?;
            results.push(ResultSignature {
                name: result.name,
                ty,
                default,
                usage: result.usage,
            });
        }
        if results.len() == 1 && matches!(types.kind(results[0].ty), Ok(jai_types::TypeKind::Void))
        {
            results.clear();
        }
        let ty = types
            .procedure(ProcedureType {
                parameters: parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == jai_syntax::ParameterEvaluation::Evaluate
                    })
                    .map(|parameter| parameter.ty)
                    .collect(),
                results: results.iter().map(|result| result.ty).collect(),
                return_abi: definition.return_abi,
                convention: definition.convention,
                context: definition.context,
                variadic: Variadic::None,
            })
            .map_err(|error| Diagnostic::new(definition.span, error.to_string()))?;
        Ok(CallbackSignature {
            argument_policy: CallbackArgumentPolicy::Established,
            ty,
            parameters,
            source_variadic: CandidateVariadic::None,
            results,
        })
    }
}
