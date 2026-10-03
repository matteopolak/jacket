//! Name filters and mappers execute checked code over VM-owned name values.
use super::*;
use jai_vm::{NoEffects, Outcome, Value, Vm};

impl Resolver<'_> {
    pub(super) fn using_computed_names(
        &mut self,
        expression: &syntax::Expression,
        span: Span,
    ) -> Result<Vec<Vec<u8>>, Diagnostic> {
        let expression = if let syntax::ExpressionKind::CompileTime(run) = &expression.kind {
            if run.flags.stallable {
                return Err(Diagnostic::new(
                    span,
                    "stallable #run requires the live-result continuation consumer",
                ));
            }
            match &run.body {
                syntax::CompileTimeBody::Expression(inner) => inner.as_ref(),
                _ => expression,
            }
        } else {
            expression
        };
        let value = self.expr(expression)?.value(span)?;
        if let Ok(constant) = self.literal_constant(value.clone(), span)
            && let ConstantKind::Array(values) = constant.kind
        {
            return values
                .into_iter()
                .map(|value| match value.kind {
                    ConstantKind::StringBytes(bytes) => Ok(bytes.to_vec()),
                    _ => Err(Diagnostic::new(
                        span,
                        "using selector requires an array of strings",
                    )),
                })
                .collect();
        }
        let ty = value.type_id(self.types);
        match self.types.kind(ty) {
            Ok(TypeKind::Slice(element) | TypeKind::FixedArray { element, .. })
                if *element == self.types.string() => {}
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "using selector requires an array of strings",
                ));
            }
        }
        self.with_using_vm(span, |vm| {
            let values = match vm.evaluate(&value).outcome {
                Outcome::Complete(values) => values,
                Outcome::Pending(dependencies) => {
                    if let Some(context) = self.compile_time {
                        context.record_pending(dependencies);
                    }
                    return Err(Diagnostic::new(
                        span,
                        "using selector is pending checked execution",
                    ));
                }
                Outcome::Failed(error) => {
                    return Err(Diagnostic::new(
                        span,
                        format!("using selector failed: {error}"),
                    ));
                }
            };
            let [value] = values.as_slice() else {
                return Err(Diagnostic::new(
                    span,
                    "using selector must return one name list",
                ));
            };
            let elements = match value {
                Value::Array { elements, .. } => elements.clone(),
                Value::Slice { pointer, count, .. } => {
                    let count = usize::try_from(*count).map_err(|_| {
                        Diagnostic::new(span, "using name list has an invalid count")
                    })?;
                    let limits = self
                        .compile_time
                        .map(|context| context.limits)
                        .unwrap_or_default();
                    if count > limits.value_cells {
                        return Err(Diagnostic::new(
                            span,
                            "using name list exceeds compile-time value budget",
                        ));
                    }
                    vm.memory()
                        .validate_slice(self.types, pointer, count)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    (0..count)
                        .map(|index| {
                            let index = isize::try_from(index).map_err(|_| {
                                Diagnostic::new(span, "using name list exceeds address budget")
                            })?;
                            let pointer = vm
                                .memory()
                                .offset(self.types, pointer, index)
                                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                            vm.memory()
                                .load(self.types, &pointer)
                                .map_err(|error| Diagnostic::new(span, error.to_string()))
                        })
                        .collect::<Result<Vec<_>, _>>()?
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "using selector must return an array of strings",
                    ));
                }
            };
            vm.materialize_values(&elements)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .into_iter()
                .map(|value| match value {
                    Value::String(bytes) => Ok(bytes),
                    _ => Err(Diagnostic::new(
                        span,
                        "using selector requires string names",
                    )),
                })
                .collect()
        })
    }

    pub(super) fn using_map_names(
        &mut self,
        mapper: &syntax::Expression,
        names: &[Vec<u8>],
        span: Span,
    ) -> Result<Vec<Vec<u8>>, Diagnostic> {
        let value = self.expr(mapper)?;
        let ty = self.expression_type(&value, span)?;
        let value = self.coerce_value(value, ty, span)?;
        let constant = self.literal_constant(value, span)?;
        let ConstantKind::Procedure(procedure) = constant.kind else {
            return Err(Diagnostic::new(
                span,
                "using map requires a compile-time procedure",
            ));
        };
        let signature = self
            .types
            .procedure_definition(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let [parameter] = signature.parameters.as_ref() else {
            return Err(Diagnostic::new(
                span,
                "using map requires exactly one []string parameter",
            ));
        };
        if !signature.results.is_empty()
            || !matches!(self.types.kind(*parameter), Ok(TypeKind::Slice(element)) if *element == self.types.string())
        {
            return Err(Diagnostic::new(
                span,
                "using map must take []string and return no values",
            ));
        }
        let slice = *parameter;
        let string = self.types.string();
        let array = self
            .types
            .fixed_array(string, names.len() as u64)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let values = names
            .iter()
            .map(|name| Value::String(name.clone()))
            .collect();
        self.with_using_vm(span, |vm| {
            let allocation = vm
                .allocate_storage(
                    array,
                    Value::Array {
                        ty: array,
                        elements: values,
                    },
                )
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let pointer = vm
                .memory()
                .sequence_data(self.types, &allocation)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let outcome = vm
                .execute(
                    procedure,
                    vec![Value::Slice {
                        ty: slice,
                        pointer,
                        count: names.len() as i64,
                    }],
                )
                .outcome;
            match outcome {
                Outcome::Complete(values) if values.is_empty() => {}
                Outcome::Complete(_) => {
                    return Err(Diagnostic::new(
                        span,
                        "using map returned unexpected values",
                    ));
                }
                Outcome::Pending(dependencies) => {
                    if let Some(context) = self.compile_time {
                        context.record_pending(dependencies);
                    }
                    return Err(Diagnostic::new(
                        span,
                        "using map procedure is pending checked execution",
                    ));
                }
                Outcome::Failed(error) => {
                    return Err(Diagnostic::new(span, format!("using map failed: {error}")));
                }
            }
            let value = vm
                .memory()
                .load(self.types, &allocation)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let Value::Array { elements, .. } = vm
                .materialize_value(&value)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            else {
                return Err(Diagnostic::new(span, "using map lost its name array"));
            };
            elements
                .into_iter()
                .map(|value| match value {
                    Value::String(bytes) => Ok(bytes),
                    _ => Err(Diagnostic::new(
                        span,
                        "using map must preserve string names",
                    )),
                })
                .collect()
        })
    }

    fn with_using_vm<T>(
        &self,
        span: Span,
        operation: impl FnOnce(
            &mut Vm<'_, compile_time::ReadyProcedures<'_>, NoEffects>,
        ) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        let mut procedures = self.meta.local_declarations.ready_snapshot();
        let mut signatures = self.meta.local_declarations.signature_snapshot();
        if let Some(context) = self.compile_time {
            procedures.extend(
                context
                    .procedures
                    .iter()
                    .map(|(&id, value)| (id, value.clone())),
            );
            signatures.extend(context.signatures.iter().map(|(&id, &ty)| (id, ty)));
            signatures.extend(context.generics.borrow().signature_snapshot());
        }
        signatures.extend(
            self.signatures
                .values()
                .map(|signature| (signature.id, signature.ty)),
        );
        let places = self.places.snapshot();
        let globals = self
            .compile_time
            .map(|context| context.globals)
            .unwrap_or(&[]);
        let provider = compile_time::ReadyProcedures::new_with_context(
            self.types,
            &procedures,
            &signatures,
            globals,
            &places,
            self.context.map(|schema| &schema.definition),
        )
        .map_err(|error| Diagnostic::new(span, error.to_string()))?
        .with_local_readiness(&self.meta.local_declarations);
        let provider = if let Some(context) = self.compile_time {
            provider
                .with_runtime(context.runtime)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .with_foreign(context.foreign)
                .with_generic_readiness(context.generics)
        } else {
            provider
        };
        let limits = self
            .compile_time
            .map(|context| context.limits)
            .unwrap_or_default();
        let mut vm = match self.compile_time.and_then(|context| context.target) {
            Some(target) => Vm::new_with_target(&provider, NoEffects, limits, target),
            None => Vm::new(&provider, NoEffects, limits),
        }
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        operation(&mut vm)
    }
}
