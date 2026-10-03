//! A partial callable preview contains checked header facts and no invented ID.
use super::baked_arguments::*;
use super::*;
use crate::procedure_values::bindings::CallbackSignature;

impl Resolver<'_> {
    pub(crate) fn preview_baked_procedure(
        &mut self,
        callee: &syntax::Expression,
        supplied: &[syntax::CallArgument],
        span: Span,
    ) -> Result<CallbackSignature, Diagnostic> {
        if let Some(BakedGenericMatch {
            candidate,
            slots,
            matched,
            descriptions,
        }) = self.match_baked_generic_procedure(callee, supplied, span, false)?
        {
            let scope = self
                .graph_scope
                .expect("generic header retains its actual defining module");
            let matched =
                self.refine_declaration_match(&candidate, &descriptions, matched, span)?;
            return scope.preview_baked_generic_header(
                &matched,
                &slots
                    .iter()
                    .enumerate()
                    .filter_map(|(formal, argument)| argument.is_some().then_some(formal))
                    .collect::<Vec<_>>(),
                self.types,
                &mut self.meta.record_specializations,
                span,
            );
        }
        let info = self.describe_argument(callee)?;
        let Some(crate::overloads::ConstantArgument::Value(
            crate::polymorphism::BakedValue::Value(value),
        )) = info.constant
        else {
            return Err(Diagnostic::new(
                callee.span,
                "baked preview requires its checked constant source target",
            ));
        };
        let ConstantKind::Procedure(procedure) = value.kind else {
            return Err(Diagnostic::new(
                callee.span,
                "baked preview target is not a checked procedure",
            ));
        };
        let mut target = self
            .meta
            .local_declarations
            .baked_procedure_target(procedure)
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.baked_procedure_target(procedure))
            })
            .ok_or_else(|| {
                Diagnostic::new(
                    callee.span,
                    "baked preview has no original source declaration",
                )
            })?;
        if target.signature.ty != value.ty {
            return Err(Diagnostic::new(
                callee.span,
                "baked preview target differs from its checked source signature",
            ));
        }
        target.metadata = self
            .preview_expected_callback_contract(callee, value.ty, callee.span)?
            .and_then(|contract| contract.callback().cloned())
            .ok_or_else(|| {
                Diagnostic::new(
                    callee.span,
                    "baked preview has no checked callable use policy",
                )
            })?;
        checked_use_metadata(&target, &[], value.ty, self.types)?;
        if target.metadata.argument_policy != super::bindings::CallbackArgumentPolicy::Established {
            return Err(Diagnostic::new(
                callee.span,
                "baked target's checked parameter names are ambiguous",
            ));
        }
        let mut assigned = vec![false; target.signature.parameters.len()];
        for argument in supplied {
            let name = argument.name.ok_or_else(|| {
                Diagnostic::new(
                    argument.value.span,
                    "#bake_arguments requires named constant arguments",
                )
            })?;
            if argument.spread {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "#bake_arguments cannot spread a runtime pack",
                ));
            }
            let formal = target
                .metadata
                .parameters
                .iter()
                .position(|parameter| parameter.name == Some(name))
                .ok_or_else(|| Diagnostic::new(argument.value.span, "unknown baked parameter"))?;
            if std::mem::replace(&mut assigned[formal], true) {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate baked parameter",
                ));
            }
            let parameter = &target.signature.parameters[formal];
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "#bake_arguments cannot supply a discarded parameter",
                ));
            }
            let info = self.describe_argument(&argument.value)?;
            if !info.is_compile_time_constant() {
                if !self.optional_baking_needs_materialization(&argument.value)? {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "baked argument must be constant",
                    ));
                }
                crate::overloads::contextual_conversion(
                    self.types,
                    self,
                    parameter.ty,
                    &info.ty,
                    argument.value.span,
                )?;
            } else {
                crate::overloads::bake(
                    self.types,
                    &crate::overloads::TypePattern::Concrete(parameter.ty),
                    &info,
                    &crate::polymorphism::Substitution::default(),
                    argument.value.span,
                )?;
            }
            self.preview_expected_callback_contract(
                &argument.value,
                parameter.ty,
                argument.value.span,
            )?;
        }
        let descriptor = self
            .types
            .procedure_definition(target.signature.ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        if descriptor.variadic != jai_types::Variadic::None
            || target.signature.source_variadic != crate::overloads::CandidateVariadic::None
        {
            return Err(Diagnostic::new(
                span,
                "baking a variadic procedure requires retained source pack binding",
            ));
        }
        let mut metadata = target.metadata;
        metadata.parameters = metadata
            .parameters
            .into_iter()
            .enumerate()
            .filter_map(|(formal, parameter)| (!assigned[formal]).then_some(parameter))
            .collect();
        metadata.ty = self
            .types
            .procedure(ProcedureType {
                parameters: metadata
                    .parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .map(|parameter| parameter.ty)
                    .collect(),
                results: descriptor.results,
                return_abi: descriptor.return_abi,
                convention: descriptor.convention,
                context: descriptor.context,
                variadic: descriptor.variadic,
            })
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(metadata)
    }
}
