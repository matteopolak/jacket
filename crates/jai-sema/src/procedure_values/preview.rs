//! Validate calls from checked source contracts without expressions or runtime identities.
use super::bindings::CallbackSignature;
use super::*;
use jai_types::Variadic;

#[derive(Clone, Copy)]
enum ArgumentChecks {
    All,
    Discarded,
}
impl ArgumentChecks {
    fn parameter(self, metadata: Option<&CallbackSignature>, index: usize) -> bool {
        matches!(self, Self::All)
            || metadata.is_some_and(|metadata| {
                metadata.parameters[index].evaluation == syntax::ParameterEvaluation::Discard
            })
    }
    fn c_variadic(self, runtime: &ProcedureType) -> bool {
        matches!(self, Self::All) || !matches!(runtime.variadic, Variadic::C { .. })
    }
}

impl Resolver<'_> {
    pub(crate) fn preview_context_call_results(
        &mut self,
        callee: &syntax::Expression,
        arguments: &[syntax::CallArgument],
        overrides: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        let record = self.require_context(span)?.definition.record_type;
        if overrides.is_empty() {
            return Err(Diagnostic::new(
                span,
                "context call requires at least one override",
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for override_value in overrides {
            if override_value.spread {
                return Err(Diagnostic::new(
                    override_value.value.span,
                    "context override cannot spread a variadic pack",
                ));
            }
            let name = override_value
                .name
                .or_else(|| self.symbols.find("allocator"))
                .ok_or_else(|| {
                    Diagnostic::new(
                        override_value.value.span,
                        "unnamed context override requires an allocator field",
                    )
                })?;
            let fields = self
                .field_path(record, name, override_value.value.span)
                .map_err(|_| {
                    Diagnostic::new(
                        override_value.value.span,
                        "unknown, ambiguous or constant context override field",
                    )
                })?;
            if !seen.insert(fields.clone()) {
                return Err(Diagnostic::new(
                    override_value.value.span,
                    "duplicate context override field",
                ));
            }
            let field = fields.last().copied().ok_or_else(|| {
                Diagnostic::new(override_value.value.span, "empty context override path")
            })?;
            let ty = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(override_value.value.span, error.to_string()))?;
            self.check_discarded_argument(&override_value.value, ty)?;
        }
        match &callee.kind {
            syntax::ExpressionKind::Name(root) => self.describe_call_results(
                &syntax::NamePath {
                    root: *root,
                    members: vec![],
                },
                arguments,
                span,
            ),
            syntax::ExpressionKind::QualifiedName(path) => {
                self.describe_call_results(path, arguments, span)
            }
            _ => {
                let info = self.describe_argument(callee)?;
                let ty = self.argument_type(&info, callee.span)?;
                let signature = self
                    .types
                    .procedure_definition(ty)
                    .map_err(|error| Diagnostic::new(callee.span, error.to_string()))?
                    .clone();
                let metadata = self
                    .preview_expected_callback_contract(callee, ty, callee.span)?
                    .and_then(|contract| contract.callback().cloned());
                self.preview_callback_call_results(&signature, metadata.as_ref(), arguments, span)
            }
        }
    }

    pub(crate) fn preview_indirect_call_results(
        &mut self,
        signature: &ProcedureType,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        self.preview_callback_call_results(signature, None, arguments, span)
    }

    pub(crate) fn preview_callback_call_results(
        &mut self,
        signature: &ProcedureType,
        metadata: Option<&CallbackSignature>,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        self.check_callback_source_arguments(
            signature,
            metadata,
            arguments,
            span,
            ArgumentChecks::All,
        )?;
        Ok(signature.results.to_vec())
    }

    pub(crate) fn preflight_discarded_callable_arguments(
        &mut self,
        signature: &ProcedureType,
        metadata: &CallbackSignature,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.check_callback_source_arguments(
            signature,
            Some(metadata),
            arguments,
            span,
            ArgumentChecks::Discarded,
        )
    }

    fn check_callback_source_arguments(
        &mut self,
        signature: &ProcedureType,
        metadata: Option<&CallbackSignature>,
        arguments: &[syntax::CallArgument],
        span: Span,
        checks: ArgumentChecks,
    ) -> Result<(), Diagnostic> {
        self.check_call_context(signature, span)?;
        let descriptor = self.callback_source_descriptor(signature, metadata, span)?;
        let mut bound = vec![false; descriptor.parameters.len()];
        let mut positional = 0;
        let mut named = false;
        let mut pack_supplied = false;
        for argument in arguments {
            if argument.spread {
                let Variadic::Jai {
                    parameter, ..
                } = descriptor.variadic
                else {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "spread arguments require a Jai variadic parameter",
                    ));
                };
                let selected = argument.name.map_or(Some(positional), |name| {
                    metadata.and_then(|metadata| {
                        metadata
                            .parameters
                            .iter()
                            .position(|candidate| candidate.name == Some(name))
                    })
                });
                if selected != Some(parameter)
                    || bound[parameter]
                    || (argument.name.is_none() && named)
                {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "spread argument must bind the variadic parameter once before trailing fixed arguments",
                    ));
                }
                if checks.parameter(metadata, parameter) {
                    self.check_discarded_argument(
                        &argument.value,
                        descriptor.parameters[parameter],
                    )?;
                }
                bound[parameter] = true;
                positional = parameter + 1;
                named |= argument.name.is_some();
                pack_supplied = false;
                continue;
            }
            if argument.name.is_some() && pack_supplied {
                let Variadic::Jai {
                    parameter, ..
                } = descriptor.variadic
                else {
                    unreachable!()
                };
                bound[parameter] = true;
                pack_supplied = false;
            }
            if argument.name.is_none() && !named {
                match descriptor.variadic {
                    Variadic::Jai {
                        parameter,
                        element,
                    } if positional >= parameter && !bound[parameter] => {
                        if checks.parameter(metadata, parameter) {
                            self.check_discarded_argument(&argument.value, element)?;
                        }
                        pack_supplied = true;
                        continue;
                    }
                    Variadic::C {
                        fixed_parameters,
                    } if positional >= fixed_parameters => {
                        if checks.c_variadic(signature) {
                            self.check_discarded_c_variadic_argument(&argument.value)?;
                        }
                        positional += 1;
                        continue;
                    }
                    _ => {}
                }
            }
            let index = if let Some(name) = argument.name {
                named = true;
                metadata
                    .ok_or_else(|| {
                        Diagnostic::new(
                            argument.value.span,
                            "named indirect arguments require a named procedure binding",
                        )
                    })?
                    .parameters
                    .iter()
                    .position(|parameter| parameter.name == Some(name))
                    .ok_or_else(|| Diagnostic::new(argument.value.span, "unknown named argument"))?
            } else {
                if named {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "positional argument cannot follow a named argument",
                    ));
                }
                let index = positional;
                positional += 1;
                index
            };
            let expected = descriptor
                .parameters
                .get(index)
                .copied()
                .ok_or_else(|| Diagnostic::new(argument.value.span, "too many arguments"))?;
            if bound[index] {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate argument for parameter",
                ));
            }
            if checks.parameter(metadata, index) {
                self.check_discarded_argument(&argument.value, expected)?;
            }
            bound[index] = true;
        }
        if let Variadic::Jai {
            parameter, ..
        } = descriptor.variadic
        {
            bound[parameter] = true;
        }
        for (index, supplied) in bound.iter().enumerate() {
            if !supplied
                && metadata
                    .and_then(|metadata| metadata.parameters[index].default.as_ref())
                    .is_none()
            {
                return Err(Diagnostic::new(span, "missing required callback argument"));
            }
        }
        Ok(())
    }
}
