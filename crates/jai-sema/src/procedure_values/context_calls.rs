//! Context overrides use checked generated helpers, preserving expression evaluation.
use super::*;

impl Resolver<'_> {
    pub(crate) fn context_call_binding(
        &mut self,
        callee: &syntax::Expression,
        args: &[syntax::CallArgument],
        overrides: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(Signature, Call), Diagnostic> {
        self.context_call_binding_with_hint(
            callee,
            args,
            overrides,
            span,
            jai_types::InlineHint::Automatic,
        )
    }
    pub(crate) fn context_call_binding_with_hint(
        &mut self,
        callee: &syntax::Expression,
        args: &[syntax::CallArgument],
        overrides: &[syntax::CallArgument],
        span: Span,
        hint: jai_types::InlineHint,
    ) -> Result<(Signature, Call), Diagnostic> {
        let record = self.require_context(span)?.definition.record_type;
        let path = match &callee.kind {
            syntax::ExpressionKind::Name(name) => Some(syntax::NamePath {
                root: *name,
                members: vec![],
            }),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        let (target, arguments, results, mut deferred) = if let Some(path) = path
            && !self.call_is_indirect(&path, span)
        {
            let (signature, call, reads) =
                self.resolve_call_binding_with_defaults(&path, args, span)?;
            (
                ValueExpr::ProcedureValue {
                    procedure: signature.id,
                    ty: signature.ty,
                },
                call.arguments,
                signature.results,
                reads,
            )
        } else {
            let target = self.expr(callee)?;
            self.resolve_indirect_call_binding_from_source_with_defaults(
                target,
                args,
                span,
                Some(callee),
            )?
        };
        let direct_target = match target {
            ValueExpr::ProcedureValue { procedure, .. } => Some(procedure),
            _ => None,
        };
        if hint == jai_types::InlineHint::Always && direct_target.is_none() {
            return Err(Diagnostic::new(
                span,
                "inline requires a constant procedure target",
            ));
        }
        if let Some(procedure) = direct_target {
            self.validate_call_hint(procedure, hint, span)?;
        }
        let mut context_fields = Vec::with_capacity(overrides.len());
        let mut seen = std::collections::HashSet::new();
        for value in overrides {
            let name = value
                .name
                .or_else(|| self.symbols.find("allocator"))
                .ok_or_else(|| {
                    Diagnostic::new(
                        value.value.span,
                        "unnamed context override requires an allocator field",
                    )
                })?;
            let fields = self
                .field_path(record, name, value.value.span)
                .map_err(|_| {
                    Diagnostic::new(
                        value.value.span,
                        "unknown, ambiguous or constant context override field",
                    )
                })?;
            if !seen.insert(fields.clone()) {
                return Err(Diagnostic::new(
                    value.value.span,
                    "duplicate context override field",
                ));
            }
            let field = *fields
                .last()
                .ok_or_else(|| Diagnostic::new(span, "empty context override path"))?;
            let ty = self
                .types
                .field_type(field)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let expression = self.expr_expected(&value.value, ty)?;
            let expression = self.coerce_value(expression, ty, value.value.span)?;
            context_fields.push((name, fields, expression));
        }
        let name = context_fields
            .first()
            .map(|field| field.0)
            .ok_or_else(|| Diagnostic::new(span, "context call requires at least one override"))?;
        let returned_contracts = if let Some(procedure) = direct_target {
            self.call_result_contracts_for_source(procedure, &arguments, Some(args), span, 0)?
        } else {
            self.callback_expression_contract(callee, &target, span)?
                .map(|contract| contract.returned())
                .unwrap_or_default()
        };
        let owner = self.reserve_generated_procedure(span)?;
        self.meta
            .callbacks
            .returned_contracts
            .insert(owner, returned_contracts);
        let mut incoming = Vec::with_capacity(arguments.len() + context_fields.len() + 2);
        incoming.push(target);
        deferred.retain(|_, read| read.is_context());
        let mut argument_slots = Vec::with_capacity(arguments.len());
        for (parameter, value) in &arguments {
            if deferred.contains_key(parameter) {
                argument_slots.push(None);
            } else {
                argument_slots.push(Some(incoming.len()));
                incoming.push(value.clone());
            }
        }
        let context_index = incoming.len();
        incoming.push(ValueExpr::Context { ty: record });
        incoming.extend(context_fields.iter().map(|(_, _, value)| value.clone()));
        let parameter_types = incoming
            .iter()
            .map(|value| value.type_id(self.types))
            .collect::<Vec<_>>();
        let ty = self
            .types
            .procedure(ProcedureType {
                parameters: parameter_types.clone().into(),
                results: results.iter().map(|result| result.ty).collect(),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let mut locals = parameter_types
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                Local::new_typed(owner, index, *ty, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let parameters = locals.clone();
        let mut body = Vec::new();
        for (index, (_, fields, _)) in context_fields.iter().enumerate() {
            let mut destination = parameters[context_index].place();
            for field in fields {
                destination = self
                    .places
                    .field(destination, *field, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            }
            body.push(Statement::Store(
                destination,
                ValueExpr::Load(parameters[context_index + 1 + index].place()),
            ));
        }
        let mut destinations = Vec::with_capacity(results.len());
        let mut returns = Vec::with_capacity(results.len());
        for result in &results {
            let local = Local::new_typed(owner, locals.len(), result.ty, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            locals.push(local);
            destinations.push(Some(local.place()));
            returns.push(ValueExpr::Load(local.place()));
        }
        let arguments = arguments
            .iter()
            .enumerate()
            .map(|(index, (parameter, _))| {
                let value = match argument_slots[index] {
                    Some(index) => ValueExpr::Load(parameters[index].place()),
                    None => {
                        let read = &deferred[parameter];
                        self.materialize_runtime_default(read, read.ty(), span)?
                    }
                };
                Ok((*parameter, value))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let transfer = if returns.is_empty() {
            Transfer::ReturnVoid
        } else {
            Transfer::ReturnValues(returns)
        };
        let target_call = if hint != jai_types::InlineHint::Automatic
            && let Some(procedure) = direct_target
        {
            let call = Call::new(procedure, arguments).with_inline_hint(hint);
            if results.is_empty() {
                Statement::CallVoid(call)
            } else {
                Statement::CallResults { call, destinations }
            }
        } else {
            Statement::IndirectCallResults {
                inline_hint: hint,
                callee: Box::new(ValueExpr::Load(parameters[0].place())),
                arguments,
                destinations,
            }
        };
        body.push(Statement::PushContext {
            id: PushContextId::new(owner, 0),
            value: ValueExpr::Load(parameters[context_index].place()),
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    target_call,
                    Statement::Exit(Exit {
                        cleanups: vec![],
                        transfer,
                    }),
                ],
            },
        });
        let signature = Signature {
            id: owner,
            ty,
            source_variadic: crate::overloads::CandidateVariadic::None,
            parameters: parameter_types
                .into_iter()
                .map(|ty| ParameterSignature {
                    name,
                    ty,
                    default: None,
                    evaluation: syntax::ParameterEvaluation::Evaluate,
                })
                .collect(),
            results,
        };
        let procedure = Procedure {
            id: owner,
            signature: ty,
            parameters,
            locals,
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: body,
            },
        };
        self.meta
            .local_declarations
            .publish_generated(signature.clone(), procedure)?;
        let call = Call::new(
            owner,
            incoming
                .into_iter()
                .enumerate()
                .map(|(index, value)| (ParameterId::new(index), value))
                .collect(),
        );
        Ok((signature, call))
    }

    pub(crate) fn context_call(
        &mut self,
        callee: &syntax::Expression,
        args: &[syntax::CallArgument],
        overrides: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let (signature, call) = self.context_call_binding(callee, args, overrides, span)?;
        match signature.results.as_slice() {
            [] => Ok(Expr::Void(call)),
            [result] => self.typed_value(
                ValueExpr::Call {
                    ty: result.ty,
                    call,
                },
                result.ty,
                span,
            ),
            _ => Err(Diagnostic::new(
                span,
                "multiple context-call results require result binding",
            )),
        }
    }
}
