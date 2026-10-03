//! Getter/setter updates capture real operands rather than inventing an lvalue.
use super::*;

enum CapturedIndex {
    Literal(i128),
    Storage(Local),
}

impl Resolver<'_> {
    pub(crate) fn overloaded_index_update(
        &mut self,
        target: &syntax::PlaceSyntax,
        operation: BinaryOp,
        right: &Expression,
    ) -> Option<Result<Statement, Diagnostic>> {
        optional_result(self.build_index_operator_update(target, operation, right))
    }

    fn build_index_operator_update(
        &mut self,
        target: &syntax::PlaceSyntax,
        operation: BinaryOp,
        right: &Expression,
    ) -> Result<Option<Statement>, Diagnostic> {
        let syntax::PlaceKind::Index {
            base,
            index,
        } = &target.kind
        else {
            return Ok(None);
        };
        let address = self.mutation_operator_operand(base)?;
        let receiver = self.describe_argument(&address)?;
        if !self.nominal_operator_operand(&receiver) {
            return Ok(None);
        }
        let indexed = place_expression(target);
        let updated = Expression {
            span: Span::new(target.span.start, right.span.end),
            kind: ExpressionKind::Binary(
                operation,
                Box::new(indexed.clone()),
                Box::new(right.clone()),
            ),
        };
        let Some(setter) = self.select_operator(
            OperatorKind::IndexAssign,
            &[&address, index, &updated],
            updated.span,
            true,
        ) else {
            return Ok(None);
        };
        let setter = setter?;
        self.check_captured_operator_projection(&setter, updated.span)?;
        let (reader, address_reader) =
            match self.select_operator(OperatorKind::Index, &[base, index], target.span, true) {
                Some(reader) => (reader?, false),
                None => {
                    let reader = self
                        .try_select_operator(
                            OperatorKind::IndexAddress,
                            &[&address, index],
                            target.span,
                        )
                        .ok_or_else(|| {
                            Diagnostic::new(target.span,
                        "indexed compound assignment requires a getter or index address operator")
                        })??;
                    (reader, true)
                }
            };
        let calculation = self
            .try_select_operator(
                OperatorKind::Binary(operation),
                &[&indexed, right],
                updated.span,
            )
            .transpose()?;
        self.check_captured_operator_projection(&reader, target.span)?;
        if let Some(calculation) = &calculation {
            self.check_captured_operator_projection(calculation, updated.span)?;
        }
        let setter_signature = self.operator_selection_signature(&setter, updated.span)?;
        let reader_signature = self.operator_selection_signature(&reader, target.span)?;
        let setter_destinations = operand_destinations(&setter, &setter_signature, updated.span)?;
        let reader_destinations = operand_destinations(&reader, &reader_signature, target.span)?;
        let base_ty = setter_signature.parameters[setter_destinations[0]].ty;
        let base_value = self.expr_expected(&address, base_ty)?;
        let base_local = self.allocate_typed(base_ty)?;
        let mut statements = vec![Statement::Store(
            base_local.place(),
            self.coerce_value(base_value, base_ty, base.span)?,
        )];
        let reader_base_ty = reader_signature.parameters[reader_destinations[0]].ty;
        let reader_base = if matches!(self.types.kind(reader_base_ty), Ok(TypeKind::Pointer(_))) {
            self.typed_value(ValueExpr::Load(base_local.place()), base_ty, base.span)?
        } else {
            let place = self
                .places
                .dereference(ValueExpr::Load(base_local.place()), self.types)
                .map_err(|error| Diagnostic::new(base.span, error.to_string()))?;
            let snapshot = self.allocate_typed(reader_base_ty)?;
            let value = self.typed_value(ValueExpr::Load(place), place.ty(), base.span)?;
            statements.push(Statement::Store(
                snapshot.place(),
                self.coerce_value(value, reader_base_ty, base.span)?,
            ));
            self.typed_value(ValueExpr::Load(snapshot.place()), reader_base_ty, base.span)?
        };
        let index_ty = reader_signature.parameters[reader_destinations[1]].ty;
        let capture_ty = match self.describe_argument(index)?.ty {
            ArgumentType::Known(ty) => ty,
            _ => index_ty,
        };
        let index_value = self.expr_expected(index, capture_ty)?;
        // Weak integer literals have no effects and retain their range for
        // both formal contexts. Runtime indices occupy one captured local.
        let captured_index = match index_value {
            Expr::Literal(value) => CapturedIndex::Literal(value),
            value => {
                let local = self.allocate_typed(capture_ty)?;
                statements.push(Statement::Store(
                    local.place(),
                    self.coerce_value(value, capture_ty, index.span)?,
                ));
                CapturedIndex::Storage(local)
            }
        };
        let reader_index = self.captured_operator_index(&captured_index, index.span)?;
        let (_, read_call) = self.operator_call_with_values(
            reader,
            reader_signature.clone(),
            vec![reader_base, reader_index],
            target.span,
        )?;
        let result_ty = reader_signature.results[0].ty;
        let read_value = self.typed_value(
            ValueExpr::Call {
                ty: result_ty,
                call: read_call,
            },
            result_ty,
            target.span,
        )?;
        let old_value = if address_reader {
            let Expr::Pointer {
                value, ..
            } = read_value
            else {
                return Err(Diagnostic::new(
                    target.span,
                    "index address operator must return a pointer",
                ));
            };
            let place = self
                .places
                .dereference(value, self.types)
                .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
            self.typed_value(ValueExpr::Load(place), place.ty(), target.span)?
        } else {
            read_value
        };
        let old_ty = self.expression_type(&old_value, target.span)?;
        let old = self.allocate_typed(old_ty)?;
        statements.push(Statement::Store(
            old.place(),
            self.coerce_value(old_value, old_ty, target.span)?,
        ));
        let left = self.typed_value(ValueExpr::Load(old.place()), old_ty, target.span)?;
        let result = if let Some(calculation) = calculation {
            let signature = self.operator_selection_signature(&calculation, updated.span)?;
            let destinations = operand_destinations(&calculation, &signature, updated.span)?;
            let right_value =
                self.expr_expected(right, signature.parameters[destinations[1]].ty)?;
            let result_ty = signature.results[0].ty;
            let (_, call) = self.operator_call_with_values(
                calculation,
                signature,
                vec![left, right_value],
                updated.span,
            )?;
            self.typed_value(
                ValueExpr::Call {
                    ty: result_ty,
                    call,
                },
                result_ty,
                updated.span,
            )?
        } else {
            let right_value = self.compound_operand(right, old_ty, operation)?;
            self.binary(operation, left, right_value, updated.span)?
        };
        let base_pointer =
            self.typed_value(ValueExpr::Load(base_local.place()), base_ty, base.span)?;
        let setter_index = self.captured_operator_index(&captured_index, index.span)?;
        let (_, call) = self.operator_call_with_values(
            setter,
            setter_signature,
            vec![base_pointer, setter_index, result],
            updated.span,
        )?;
        statements.push(Statement::CallVoid(call));
        Ok(Some(Statement::Block(Block {
            statements,
            flow: Flow::FallsThrough,
        })))
    }

    fn captured_operator_index(
        &mut self,
        captured: &CapturedIndex,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        match captured {
            CapturedIndex::Literal(value) => Ok(Expr::Literal(*value)),
            CapturedIndex::Storage(local) => {
                self.typed_value(ValueExpr::Load(local.place()), local.place().ty(), span)
            }
        }
    }

    fn operator_selection_signature(
        &mut self,
        selection: &SelectedOperator,
        span: Span,
    ) -> Result<Signature, Diagnostic> {
        let signature = match &selection.origin {
            OperatorOrigin::File(matched) => {
                self.materialize_declaration_match(matched.clone(), span)?
            }
            OperatorOrigin::Local(signature) => signature.clone(),
            OperatorOrigin::LocalGeneric(matched) => self
                .materialize_local_operator_match_with_contracts(
                    matched.clone(),
                    &selection.arguments,
                    span,
                )?,
        };
        self.validate_operator_signature(selection.kind, &signature, span)?;
        Ok(signature)
    }

    fn operator_call_with_values(
        &mut self,
        selection: SelectedOperator,
        signature: Signature,
        values: Vec<Expr>,
        span: Span,
    ) -> Result<(Signature, Call), Diagnostic> {
        let descriptor = self
            .types
            .procedure_definition(signature.ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.check_call_context(descriptor, span)?;
        let destinations = operand_destinations(&selection, &signature, span)?;
        let mut arguments = values
            .into_iter()
            .zip(&destinations)
            .map(|(value, &index)| {
                Ok((
                    ParameterId::new(index),
                    self.coerce_value(value, signature.parameters[index].ty, span)?,
                ))
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        for (index, parameter) in signature.parameters.iter().enumerate() {
            if destinations.contains(&index) {
                continue;
            }
            let default = parameter.default.as_ref().ok_or_else(|| {
                Diagnostic::new(span, "operator trailing parameter requires a default")
            })?;
            arguments.push((
                ParameterId::new(index),
                self.materialize_parameter_default(default, parameter.ty, span)?,
            ));
        }
        self.warn_deprecated_procedure(signature.id, span)?;
        let call = Call::new(signature.id, arguments);
        Ok((signature, call))
    }
}

fn operand_destinations(
    selected: &SelectedOperator,
    signature: &Signature,
    span: Span,
) -> Result<Vec<usize>, Diagnostic> {
    selected
        .arguments
        .iter()
        .map(|argument| {
            signature
                .parameters
                .iter()
                .position(|parameter| Some(parameter.name) == argument.name)
                .ok_or_else(|| Diagnostic::new(span, "operator operand destination is unavailable"))
        })
        .collect()
}
