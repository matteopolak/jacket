//! Source formals retain checking contracts independently of runtime ABI slots.
use super::bindings::CallbackSignature;
use super::*;
use crate::overloads::CandidateVariadic;
use jai_types::Variadic;

impl Resolver<'_> {
    pub(crate) fn callback_source_descriptor(
        &self,
        runtime: &ProcedureType,
        metadata: Option<&CallbackSignature>,
        span: Span,
    ) -> Result<ProcedureType, Diagnostic> {
        let Some(metadata) = metadata else {
            return Ok(runtime.clone());
        };
        if metadata.argument_policy == super::bindings::CallbackArgumentPolicy::Ambiguous {
            return Err(Diagnostic::new(
                span,
                "callback source parameter policies differ; supply an explicit callback annotation",
            ));
        }
        let runtime_types: Vec<_> = metadata
            .parameters
            .iter()
            .filter(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Evaluate)
            .map(|parameter| parameter.ty)
            .collect();
        if runtime_types.as_slice() != runtime.parameters.as_ref() {
            return Err(Diagnostic::new(
                span,
                "callback parameter metadata differs from its canonical signature",
            ));
        }
        let variadic = match metadata.source_variadic {
            CandidateVariadic::None => Variadic::None,
            CandidateVariadic::C {
                fixed_parameters,
            } => Variadic::C {
                fixed_parameters,
            },
            CandidateVariadic::Jai {
                parameter,
            } => {
                let ty = metadata
                    .parameters
                    .get(parameter)
                    .ok_or_else(|| Diagnostic::new(span, "source variadic parameter is absent"))?
                    .ty;
                let jai_types::TypeKind::Slice(element) = self
                    .types
                    .kind(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    return Err(Diagnostic::new(
                        span,
                        "source variadic parameter requires a slice type",
                    ));
                };
                Variadic::Jai {
                    parameter,
                    element: *element,
                }
            }
        };
        Ok(ProcedureType {
            parameters: metadata
                .parameters
                .iter()
                .map(|parameter| parameter.ty)
                .collect(),
            results: runtime.results.clone(),
            return_abi: runtime.return_abi,
            convention: runtime.convention,
            context: runtime.context,
            variadic,
        })
    }

    pub(crate) fn bind_callable_arguments_with_defaults(
        &mut self,
        runtime: &ProcedureType,
        metadata: Option<&CallbackSignature>,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<crate::runtime_defaults::BoundDefaultArguments, Diagnostic> {
        let descriptor = self.callback_source_descriptor(runtime, metadata, span)?;
        if let Some(metadata) = metadata
            && (metadata
                .parameters
                .iter()
                .any(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Discard)
                || (matches!(metadata.source_variadic, CandidateVariadic::C { .. })
                    && !matches!(runtime.variadic, Variadic::C { .. })))
        {
            self.preflight_discarded_callable_arguments(runtime, metadata, args, span)?;
        }
        let evaluates: Vec<_> = (0..descriptor.parameters.len())
            .map(|index| {
                metadata.is_none_or(|metadata| {
                    metadata.parameters[index].evaluation == syntax::ParameterEvaluation::Evaluate
                })
            })
            .collect();
        let mut runtime_count = 0;
        let runtime_ids: Vec<_> = evaluates
            .iter()
            .map(|evaluate| {
                evaluate.then(|| {
                    let id = ParameterId::new(runtime_count);
                    runtime_count += 1;
                    id
                })
            })
            .collect();
        let mut pack = Vec::new();
        let mut bound = vec![false; descriptor.parameters.len()];
        let mut arguments = Vec::with_capacity(runtime_count);
        let mut omitted_reads = HashMap::new();
        let mut positional = 0;
        let mut named = false;
        for argument in args {
            if argument.spread {
                let Variadic::Jai {
                    parameter,
                    element,
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
                let slice = descriptor.parameters[parameter];
                if let Some(id) = runtime_ids[parameter] {
                    let value = if pack.is_empty() {
                        let value = self.expr_expected(&argument.value, slice)?;
                        self.coerce_value(value, slice, argument.value.span)?
                    } else {
                        self.concat_variadic_pack(&pack, argument, element, slice)?
                    };
                    arguments.push((id, value));
                } else {
                    for value in &pack {
                        self.check_discarded_argument(&value.value, element)?;
                    }
                    self.check_discarded_argument(&argument.value, slice)?;
                }
                pack.clear();
                bound[parameter] = true;
                positional = parameter + 1;
                named |= argument.name.is_some();
                continue;
            }
            if argument.name.is_some() && !pack.is_empty() {
                let Variadic::Jai {
                    parameter,
                    element,
                } = descriptor.variadic
                else {
                    unreachable!()
                };
                if let Some(id) = runtime_ids[parameter] {
                    arguments.push((
                        id,
                        self.jai_variadic_pack(
                            &pack,
                            element,
                            descriptor.parameters[parameter],
                            span,
                        )?,
                    ));
                } else {
                    for value in &pack {
                        self.check_discarded_argument(&value.value, element)?;
                    }
                }
                bound[parameter] = true;
                pack.clear();
            }
            if argument.name.is_none() && !named {
                match descriptor.variadic {
                    Variadic::Jai {
                        parameter, ..
                    } if positional >= parameter && !bound[parameter] => {
                        pack.push(argument);
                        continue;
                    }
                    Variadic::C {
                        fixed_parameters,
                    } if positional >= fixed_parameters => {
                        if matches!(runtime.variadic, Variadic::C { .. }) {
                            arguments.push((
                                ParameterId::new(runtime_count + positional - fixed_parameters),
                                self.c_variadic_argument(argument)?,
                            ));
                        } else {
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
                let metadata = metadata.ok_or_else(|| {
                    Diagnostic::new(
                        argument.value.span,
                        "named indirect arguments require a named procedure binding",
                    )
                })?;
                metadata
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
            bound[index] = true;
            if let Some(id) = runtime_ids[index] {
                let value = self.expr_expected(&argument.value, expected)?;
                arguments.push((id, self.coerce_value(value, expected, argument.value.span)?));
            } else {
                self.check_discarded_argument(&argument.value, expected)?;
            }
        }
        if let Variadic::Jai {
            parameter,
            element,
        } = descriptor.variadic
            && !bound[parameter]
        {
            if let Some(id) = runtime_ids[parameter] {
                arguments.push((
                    id,
                    self.jai_variadic_pack(&pack, element, descriptor.parameters[parameter], span)?,
                ));
            } else {
                for value in &pack {
                    self.check_discarded_argument(&value.value, element)?;
                }
            }
            bound[parameter] = true;
        }
        for (index, supplied) in bound.iter().enumerate() {
            if *supplied {
                continue;
            }
            let parameter = metadata.and_then(|metadata| metadata.parameters.get(index));
            let default = parameter
                .and_then(|parameter| parameter.default.as_ref())
                .ok_or_else(|| {
                    let name = parameter
                        .and_then(|parameter| parameter.name)
                        .map(|name| format!(" '{}'", self.symbols.name(name)))
                        .unwrap_or_default();
                    Diagnostic::new(span, format!("missing required argument{name}"))
                })?;
            if let Some(id) = runtime_ids[index] {
                if let Some(ParameterDefault::RuntimeRead(read)) = default.prepared() {
                    omitted_reads.insert(id, read.clone());
                }
                arguments.push((
                    id,
                    self.materialize_parameter_default(
                        default,
                        descriptor.parameters[index],
                        span,
                    )?,
                ));
            }
        }
        Ok((arguments, omitted_reads))
    }
}
