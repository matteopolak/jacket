//! Bind ordered procedure results without constructing a language tuple.
use super::*;
mod mixed;
struct IndirectResultCall<'a> {
    callee: Expr,
    args: &'a [syntax::CallArgument],
    span: Span,
    source: Option<&'a syntax::Expression>,
}

impl Resolver<'_> {
    fn capture_results(
        &mut self,
        source: &[syntax::Expression],
        expected: &[Option<TypeId>],
        used: &[bool],
        statements: &mut Vec<Statement>,
    ) -> Result<Vec<Place>, Diagnostic> {
        let mut values = Vec::new();
        for expression in source {
            if let syntax::ExpressionKind::CallHint {
                hint,
                call,
            } = &expression.kind
            {
                self.capture_hinted_call(
                    *hint,
                    call,
                    expression.span,
                    used,
                    statements,
                    &mut values,
                )?;
                continue;
            }
            if let syntax::ExpressionKind::ContextCall {
                callee,
                args,
                overrides,
            } = &expression.kind
            {
                let (signature, call) =
                    self.context_call_binding(callee, args, overrides, expression.span)?;
                crate::result_obligations::check_result_use(
                    &signature.results,
                    used,
                    values.len(),
                    expression.span,
                )?;
                if signature.results.is_empty() {
                    return Err(Diagnostic::new(
                        expression.span,
                        "void context call supplies no assignment results",
                    ));
                }
                let mut destinations = Vec::new();
                let contracts = self.call_result_contracts(&call, expression.span)?;
                for (index, result) in signature.results.iter().enumerate() {
                    let local = self.allocate_typed(result.ty)?;
                    self.bind_value_contract(
                        local.place(),
                        contracts.get(index).cloned().flatten(),
                        expression.span,
                    )?;
                    destinations.push(Some(local.place()));
                    values.push(local.place());
                }
                statements.push(Statement::CallResults {
                    call,
                    destinations,
                });
                continue;
            }
            let direct = match &expression.kind {
                syntax::ExpressionKind::Call(name, args) => Some((
                    syntax::NamePath {
                        root: *name,
                        members: Vec::new(),
                    },
                    args,
                )),
                syntax::ExpressionKind::QualifiedCall(path, args) => Some((path.clone(), args)),
                _ => None,
            };
            if let Some((path, args)) = direct {
                if !self.call_is_indirect(&path, expression.span) {
                    let (signature, call) =
                        self.resolve_call_binding(&path, args, expression.span)?;
                    crate::result_obligations::check_result_use(
                        &signature.results,
                        used,
                        values.len(),
                        expression.span,
                    )?;
                    if signature.results.is_empty() {
                        return Err(Diagnostic::new(
                            expression.span,
                            "void call supplies no assignment results",
                        ));
                    }
                    let mut destinations = Vec::new();
                    let contracts =
                        self.call_result_contracts_from_source(&call, args, expression.span)?;
                    for (index, result) in signature.results.iter().enumerate() {
                        let local = self.allocate_typed(result.ty)?;
                        self.bind_value_contract(
                            local.place(),
                            contracts.get(index).cloned().flatten(),
                            expression.span,
                        )?;
                        destinations.push(Some(local.place()));
                        values.push(local.place());
                    }
                    statements.push(Statement::CallResults {
                        call,
                        destinations,
                    });
                    continue;
                } else {
                    let source = self.baked_callback_call_source(&path, expression.span)?;
                    let callee = self.path_expression(&path, expression.span)?;
                    self.capture_indirect_results(
                        IndirectResultCall {
                            callee,
                            args,
                            span: expression.span,
                            source: source.as_ref(),
                        },
                        statements,
                        &mut values,
                        used,
                    )?;
                    continue;
                }
            }
            if let syntax::ExpressionKind::IndirectCall {
                callee,
                args,
            } = &expression.kind
            {
                let target = self.expr(callee)?;
                self.capture_indirect_results(
                    IndirectResultCall {
                        callee: target,
                        args,
                        span: expression.span,
                        source: Some(callee),
                    },
                    statements,
                    &mut values,
                    used,
                )?;
                continue;
            }
            let expected = expected.get(values.len()).copied().flatten();
            let value = match expected {
                Some(ty) => self.expr_expected(expression, ty)?,
                None => self.expr(expression)?,
            };
            let ty = match expected {
                Some(ty) => ty,
                None => self.expression_type(&value, expression.span)?,
            };
            let value = self.coerce_value(value, ty, expression.span)?;
            self.check_bound_operator_result_use(expression, &value, used, values.len())?;
            let local = self.allocate_typed(ty)?;
            let contract =
                self.callback_expression_contract(expression, &value, expression.span)?;
            self.bind_value_contract(local.place(), contract, expression.span)?;
            statements.push(Statement::Store(local.place(), value));
            values.push(local.place());
        }
        if source.len() == 1
            && values.len() == 1
            && expected.len() > 1
            && !is_result_call(&source[0])
        {
            values.resize(expected.len(), values[0]);
        }
        Ok(values)
    }

    fn capture_indirect_results(
        &mut self,
        call: IndirectResultCall<'_>,
        statements: &mut Vec<Statement>,
        values: &mut Vec<Place>,
        used: &[bool],
    ) -> Result<(), Diagnostic> {
        let IndirectResultCall {
            callee,
            args,
            span,
            source,
        } = call;
        let (callee, arguments, results) =
            self.resolve_indirect_call_binding_from_source(callee, args, span, source)?;
        let contract = match source {
            Some(source) => self.callback_expression_contract(source, &callee, span)?,
            None => self.callback_value_contract(&callee, span)?,
        };
        let contracts = contract
            .map(|contract| contract.returned())
            .unwrap_or_default();
        crate::result_obligations::check_result_use(&results, used, values.len(), span)?;
        if results.is_empty() {
            return Err(Diagnostic::new(
                span,
                "void call supplies no assignment results",
            ));
        }
        let mut destinations = Vec::new();
        for (index, result) in results.into_iter().enumerate() {
            let local = self.allocate_typed(result.ty)?;
            self.bind_value_contract(local.place(), contracts.get(index).cloned().flatten(), span)?;
            destinations.push(Some(local.place()));
            values.push(local.place());
        }
        statements.push(Statement::IndirectCallResults {
            inline_hint: jai_types::InlineHint::Automatic,
            callee: Box::new(callee),
            arguments,
            destinations,
        });
        Ok(())
    }

    pub(crate) fn declare_results(
        &mut self,
        names: &[Symbol],
        annotation: Option<&syntax::TypeSyntax>,
        source: &[syntax::Expression],
    ) -> Result<Statement, Diagnostic> {
        let ty = match annotation {
            Some(annotation) => Some(self.lexical_annotation(annotation, self.span)?),
            None => None,
        };
        let mut statements = Vec::new();
        if source.len() == 1 && matches!(source[0].kind, syntax::ExpressionKind::Uninitialized) {
            let ty = ty.ok_or_else(|| {
                Diagnostic::new(
                    source[0].span,
                    "--- declaration lists require an explicit type",
                )
            })?;
            for &name in names {
                if self.symbols.name(name) == "_" {
                    continue;
                }
                let local = self.declare_typed(name, ty)?;
                let contract = self.annotation_value_contract(
                    ty,
                    annotation.expect("uninitialized declaration checked its annotation"),
                    self.span,
                )?;
                self.bind_value_contract(local.place(), contract, self.span)?;
            }
            return Ok(Statement::Block(Block {
                statements,
                flow: Flow::FallsThrough,
            }));
        }
        let used = names
            .iter()
            .map(|name| self.symbols.name(*name) != "_")
            .collect::<Vec<_>>();
        let values =
            self.capture_results(source, &vec![ty; names.len()], &used, &mut statements)?;
        if !source.is_empty() && values.len() != names.len() {
            return Err(self.error("assignment result count does not match declaration count"));
        }
        for (index, name) in names.iter().enumerate() {
            if self.symbols.name(*name) == "_" {
                continue;
            }
            let result_ty = ty
                .or_else(|| values.get(index).map(|value| value.ty()))
                .ok_or_else(|| self.error("declaration without values requires a type"))?;
            let value = if let Some(place) = values.get(index) {
                let value = self.typed_value(ValueExpr::Load(*place), place.ty(), self.span)?;
                let value = self.implicit_field_pointer(value, result_ty, self.span)?;
                self.coerce_value(value, result_ty, self.span)?
            } else {
                self.default_value(result_ty, self.span)?.into_expression()
            };
            let local = self.declare_typed(*name, result_ty)?;
            let contract = match annotation {
                Some(annotation) => {
                    self.annotation_value_contract(result_ty, annotation, self.span)?
                }
                None => self.callback_value_contract(&value, self.span)?,
            };
            self.bind_value_contract(local.place(), contract, self.span)?;
            statements.push(Statement::Store(local.place(), value));
        }
        Ok(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        }))
    }

    pub(crate) fn assign_results(
        &mut self,
        targets: &[syntax::PlaceSyntax],
        source: &[syntax::Expression],
        operation: Option<syntax::BinaryOp>,
    ) -> Result<Statement, Diagnostic> {
        let mut statements = Vec::new();
        let mut places = Vec::new();
        let mut left = Vec::new();
        for target in targets {
            if self.is_result_discard_target(target) {
                places.push(None);
                left.push(None);
                continue;
            }
            let place = self.resolve_place(target)?;
            self.reject_iteration_write(place, target.span)?;
            let pointer = self
                .types
                .pointer(place.ty())
                .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
            let address = self.allocate_typed(pointer)?;
            statements.push(Statement::Store(
                address.place(),
                ValueExpr::AddressOf {
                    place,
                    ty: pointer,
                },
            ));
            let place = self
                .places
                .dereference(ValueExpr::Load(address.place()), self.types)
                .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
            let previous = if operation.is_some() {
                let local = self.allocate_typed(place.ty())?;
                statements.push(Statement::Store(local.place(), ValueExpr::Load(place)));
                Some(local.place())
            } else {
                None
            };
            places.push(Some(place));
            left.push(previous);
        }
        let expected: Vec<_> = places
            .iter()
            .map(|place| place.map(|place| place.ty()))
            .collect();
        let used = places.iter().map(Option::is_some).collect::<Vec<_>>();
        let values = self.capture_results(source, &expected, &used, &mut statements)?;
        if values.len() != places.len() {
            return Err(self.error("assignment result count does not match target count"));
        }
        for (index, place) in places.into_iter().enumerate() {
            let Some(place) = place else {
                continue;
            };
            let value = self.typed_value(
                ValueExpr::Load(values[index]),
                values[index].ty(),
                targets[index].span,
            )?;
            let value = if let Some(operation) = operation {
                let previous = left[index].expect("compound assignment captured its left value");
                let previous = self.typed_value(
                    ValueExpr::Load(previous),
                    previous.ty(),
                    targets[index].span,
                )?;
                self.binary(operation, previous, value, targets[index].span)?
            } else {
                value
            };
            let value = self.implicit_field_pointer(value, place.ty(), targets[index].span)?;
            statements.push(Statement::Store(
                place,
                self.coerce_value(value, place.ty(), targets[index].span)?,
            ));
        }
        Ok(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        }))
    }
}

fn is_result_call(expression: &syntax::Expression) -> bool {
    matches!(
        expression.kind,
        syntax::ExpressionKind::Call(..)
            | syntax::ExpressionKind::QualifiedCall(..)
            | syntax::ExpressionKind::IndirectCall { .. }
            | syntax::ExpressionKind::ContextCall { .. }
            | syntax::ExpressionKind::CallHint { .. }
    )
}
