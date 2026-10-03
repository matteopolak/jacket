//! Generic result contracts follow source variables bound by checked runtime arguments.
use super::*;

impl Resolver<'_> {
    pub(crate) fn register_generic_callback_arguments(
        &mut self,
        call: &Call,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.call_result_contracts_from_source(call, arguments, span)?;
        Ok(())
    }
    pub(crate) fn call_result_contracts(
        &mut self,
        call: &Call,
        span: Span,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        self.call_result_contracts_for(call.procedure, &call.arguments, span)
    }
    pub(crate) fn call_result_contracts_from_source(
        &mut self,
        call: &Call,
        source: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        self.call_result_contracts_for_source(
            call.procedure,
            &call.arguments,
            Some(source),
            span,
            0,
        )
    }
    pub(crate) fn call_result_contracts_for(
        &mut self,
        procedure: ProcedureId,
        arguments: &[(ParameterId, ValueExpr)],
        span: Span,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        self.call_result_contracts_for_depth(procedure, arguments, span, 0)
    }
    pub(super) fn call_result_contracts_for_depth(
        &mut self,
        procedure: ProcedureId,
        arguments: &[(ParameterId, ValueExpr)],
        span: Span,
        depth: usize,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        self.call_result_contracts_for_source(procedure, arguments, None, span, depth)
    }
    pub(crate) fn call_result_contracts_for_source(
        &mut self,
        procedure: ProcedureId,
        arguments: &[(ParameterId, ValueExpr)],
        source_arguments: Option<&[syntax::CallArgument]>,
        span: Span,
        depth: usize,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        self.call_result_contracts_for_source_overrides(
            procedure,
            arguments,
            source_arguments,
            None,
            span,
            depth,
        )
    }

    /// Overrides are checked policies of genuine supplied original formals. An
    /// explicit `None` is authoritative; an erased constant cannot restore it.
    pub(crate) fn call_result_contracts_for_source_overrides(
        &mut self,
        procedure: ProcedureId,
        arguments: &[(ParameterId, ValueExpr)],
        source_arguments: Option<&[syntax::CallArgument]>,
        overrides: Option<&HashMap<Symbol, Option<ValueContract>>>,
        span: Span,
        depth: usize,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "generic callback contract exceeds call depth",
            ));
        }
        if overrides.is_none()
            && let Some(contracts) = self.local_operator_call_result_contracts(
                procedure,
                arguments,
                source_arguments,
                span,
                depth,
            )?
        {
            return Ok(contracts);
        }
        let mut contracts = self.procedure_result_contracts(procedure, span)?;
        let Some(scope) = self.graph_scope else {
            return Ok(contracts);
        };
        let Some((file, parameters, results)) = scope.callback_contract_header(procedure) else {
            return Ok(contracts);
        };
        let Some(signature) = self.contract_procedure_signature(procedure) else {
            return Ok(contracts);
        };
        if let Some(overrides) = overrides {
            for (name, contract) in overrides {
                if !parameters.iter().any(|source| source.name == *name) {
                    return Err(Diagnostic::new(
                        span,
                        "partial policy has no original source formal",
                    ));
                }
                let parameter = signature
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == *name)
                    .ok_or_else(|| Diagnostic::new(span, "partial policy has no checked formal"))?;
                if contract
                    .as_ref()
                    .is_some_and(|contract| contract.ty != parameter.ty)
                {
                    return Err(Diagnostic::new(
                        span,
                        "partial policy differs from its checked formal type",
                    ));
                }
            }
        }
        let mut bindings = HashMap::new();
        let mut inferred_baked = HashMap::new();
        let mut has_baked_values = false;
        // Result-introduced Types have no runtime formal; retain their actual
        // named source annotations for this invocation's callback obligations.
        for name in scope.callback_result_type_parameters(procedure) {
            let argument = source_arguments.and_then(|arguments| {
                arguments
                    .iter()
                    .find(|argument| argument.name == Some(name))
            });
            let contract = match (
                scope.callback_specialized_type(procedure, name),
                argument.and_then(|argument| source_type(&argument.value)),
            ) {
                (Some(ty), Some(annotation)) => {
                    self.annotation_value_contract(ty, &annotation, span)?
                }
                _ => None,
            };
            bindings.insert(name, contract);
        }
        for (ordinal, source) in parameters.iter().enumerate() {
            let source_argument = source_arguments
                .and_then(|arguments| source_argument(arguments, source.name, ordinal));
            if let Some(contract) = overrides.and_then(|overrides| overrides.get(&source.name)) {
                let annotation = match &source.binding {
                    syntax::ParameterBinding::RequiredType(ty)
                    | syntax::ParameterBinding::DefaultedType {
                        ty: Some(ty), ..
                    } => Some(ty),
                    _ => None,
                };
                if let Some(annotation) = annotation {
                    capture_variables(
                        annotation,
                        contract.as_ref(),
                        &mut bindings,
                        source.span,
                        0,
                    )?;
                }
                if source.baking != syntax::ParameterBaking::None {
                    bindings.insert(source.name, contract.clone());
                    has_baked_values = true;
                }
                continue;
            }
            if source.baking != syntax::ParameterBaking::None
                && matches!(
                    source.binding,
                    syntax::ParameterBinding::RequiredType(syntax::TypeSyntax::Builtin(
                        syntax::BuiltinType::Type
                    ))
                )
            {
                let contract = match (
                    scope.callback_specialized_type(procedure, source.name),
                    source_argument.and_then(source_type),
                ) {
                    (Some(ty), Some(source)) => {
                        self.annotation_value_contract(ty, &source, span)?
                    }
                    _ => None,
                };
                bindings.insert(source.name, contract);
                continue;
            }
            if self.capture_baked_callback_variables(
                procedure,
                source,
                source_argument,
                &mut bindings,
                &mut inferred_baked,
                span,
            )? {
                has_baked_values = true;
                continue;
            }
            let syntax = match &source.binding {
                syntax::ParameterBinding::RequiredType(ty)
                | syntax::ParameterBinding::DefaultedType {
                    ty: Some(ty), ..
                } => ty,
                _ => continue,
            };
            let Some(index) = signature
                .parameters
                .iter()
                .position(|parameter| parameter.name == source.name)
            else {
                continue;
            };
            let parameter = &signature.parameters[index];
            let contract = if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                match source_argument {
                    Some(source) => {
                        self.preview_expected_callback_contract(source, parameter.ty, source.span)?
                    }
                    None => self
                        .meta
                        .callbacks
                        .discarded_defaults
                        .get(&(procedure, source.name))
                        .cloned(),
                }
            } else {
                let runtime = signature.parameters[..index]
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .count();
                let value = arguments
                    .iter()
                    .find(|(id, _)| id.index() == runtime)
                    .map(|(_, value)| value);
                match value {
                    Some(value) => match source_argument {
                        Some(source) => {
                            self.callback_expression_contract_inner(source, value, span, depth + 1)?
                        }
                        None => {
                            let default = match &source.binding {
                                syntax::ParameterBinding::Defaulted {
                                    expression, ..
                                }
                                | syntax::ParameterBinding::DefaultedType {
                                    expression, ..
                                } => Some(expression),
                                _ => None,
                            };
                            match default.map(|expression| &expression.kind) {
                                Some(syntax::ExpressionKind::TypeCast {
                                    ty, ..
                                }) => {
                                    // The runtime default has erased its cast. Its
                                    // source annotation belongs to the defining
                                    // specialization, never the caller's aliases.
                                    let substitution =
                                        scope.callback_contract_substitution(procedure);
                                    let retained = scope
                                        .callback_contract_syntax_in_specialization(
                                            file,
                                            substitution.as_ref(),
                                            ty,
                                            source.span,
                                        )?;
                                    self.bound_syntax_contract(
                                        value.type_id(self.types),
                                        &retained,
                                        &bindings,
                                        source.span,
                                        depth + 1,
                                    )?
                                }
                                _ => self.callback_value_contract_inner(value, span, depth + 1)?,
                            }
                        }
                    },
                    None => None,
                }
            };
            capture_variables(syntax, contract.as_ref(), &mut bindings, span, 0)?;
        }
        if bindings.is_empty() && !has_baked_values {
            return Ok(contracts);
        }
        let mut changed = self.merge_generic_callback_bindings(procedure, &bindings);
        for source in &parameters {
            if source.baking != syntax::ParameterBaking::None {
                changed |= self.bind_baked_callback_parameter(
                    procedure,
                    source,
                    &bindings,
                    &inferred_baked,
                    span,
                )?;
                continue;
            }
            let ty = match &source.binding {
                syntax::ParameterBinding::RequiredType(ty)
                | syntax::ParameterBinding::DefaultedType {
                    ty: Some(ty), ..
                } => ty,
                _ => continue,
            };
            let Some(parameter) = signature
                .parameters
                .iter()
                .find(|parameter| parameter.name == source.name)
            else {
                continue;
            };
            let substitution = scope.callback_contract_substitution(procedure);
            let retained = scope.callback_contract_syntax_in_specialization(
                file,
                substitution.as_ref(),
                ty,
                span,
            )?;
            let Some(contract) =
                self.bound_syntax_contract(parameter.ty, &retained, &bindings, span, 0)?
            else {
                continue;
            };
            changed |= self.merge_generic_callback_parameter(procedure, source.name, contract);
        }
        if changed {
            scope.recheck_callback_body(procedure)?;
        }
        contracts.resize(signature.results.len(), None);
        for ((source, result), contract) in
            results.iter().zip(&signature.results).zip(&mut contracts)
        {
            if let syntax::ResultBinding::Typed {
                ty, ..
            } = &source.binding
            {
                let substitution = scope.callback_contract_substitution(procedure);
                let source = scope.callback_contract_syntax_in_specialization(
                    file,
                    substitution.as_ref(),
                    ty,
                    span,
                )?;
                *contract = self.bound_syntax_contract(result.ty, &source, &bindings, span, 0)?;
            }
        }
        Ok(contracts)
    }
}
pub(super) fn capture_variables(
    source: &syntax::TypeSyntax,
    contract: Option<&ValueContract>,
    bindings: &mut HashMap<Symbol, Option<ValueContract>>,
    span: Span,
    depth: usize,
) -> Result<(), Diagnostic> {
    if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
        return Err(Diagnostic::new(
            span,
            "generic callback parameter contract exceeds source type depth",
        ));
    }
    match source {
        syntax::TypeSyntax::Variable(name)
        | syntax::TypeSyntax::Restricted {
            variable: name, ..
        } => {
            let entry = bindings.entry(*name).or_insert(None);
            if let Some(contract) = contract {
                *entry = Some(match entry.take() {
                    Some(old) => old.merge(contract.clone()),
                    None => contract.clone(),
                });
            }
        }
        syntax::TypeSyntax::Pointer(inner)
        | syntax::TypeSyntax::Slice(inner)
        | syntax::TypeSyntax::DynamicArray(inner)
        | syntax::TypeSyntax::FixedArray {
            element: inner, ..
        } => {
            let inner_contract = contract.and_then(ValueContract::element);
            capture_variables(inner, inner_contract.as_ref(), bindings, span, depth + 1)?;
        }
        syntax::TypeSyntax::Procedure(source) => {
            for parameter in &source.parameters {
                capture_variables(&parameter.ty, None, bindings, span, depth + 1)?;
            }
            let results = contract.map(ValueContract::returned).unwrap_or_default();
            for (index, result) in source.results.iter().enumerate() {
                capture_variables(
                    &result.ty,
                    results.get(index).and_then(Option::as_ref),
                    bindings,
                    span,
                    depth + 1,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn source_argument(
    arguments: &[syntax::CallArgument],
    name: Symbol,
    ordinal: usize,
) -> Option<&syntax::Expression> {
    arguments
        .iter()
        .find(|argument| argument.name == Some(name))
        .or_else(|| {
            arguments
                .iter()
                .filter(|argument| argument.name.is_none())
                .nth(ordinal)
        })
        .filter(|argument| !argument.spread)
        .map(|argument| &argument.value)
}
pub(crate) fn source_type(source: &syntax::Expression) -> Option<syntax::TypeSyntax> {
    match &source.kind {
        syntax::ExpressionKind::Type(ty) => Some(ty.clone()),
        syntax::ExpressionKind::Name(name) => Some(syntax::TypeSyntax::Named(syntax::NamePath {
            root: *name,
            members: vec![],
        })),
        syntax::ExpressionKind::QualifiedName(path) => {
            Some(syntax::TypeSyntax::Named(path.clone()))
        }
        syntax::ExpressionKind::AddressOf(inner) => {
            Some(syntax::TypeSyntax::Pointer(Box::new(source_type(inner)?)))
        }
        syntax::ExpressionKind::Call(name, arguments) => Some(syntax::TypeSyntax::Application(
            syntax::TypeApplicationSyntax {
                base: Box::new(syntax::TypeSyntax::Named(syntax::NamePath {
                    root: *name,
                    members: vec![],
                })),
                arguments: arguments.clone(),
                span: source.span,
            },
        )),
        syntax::ExpressionKind::QualifiedCall(path, arguments) => Some(
            syntax::TypeSyntax::Application(syntax::TypeApplicationSyntax {
                base: Box::new(syntax::TypeSyntax::Named(path.clone())),
                arguments: arguments.clone(),
                span: source.span,
            }),
        ),
        _ => None,
    }
}
