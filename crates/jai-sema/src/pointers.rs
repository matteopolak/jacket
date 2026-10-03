//! Pointer expressions retain typed places instead of encoding host addresses.
use super::*;
use jai_types::TypeKind;
mod index_values;
#[cfg(test)]
mod tests;

impl Resolver<'_> {
    pub(crate) fn pointer_conditional(
        &mut self,
        expression: &syntax::ConditionalExpression,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let condition = self.condition_expression(&expression.condition)?;
        let yes = match expected {
            Some(ty) => self.expr_expected(&expression.then_value, ty)?,
            None => self.expr(&expression.then_value)?,
        };
        let no = expression
            .else_value
            .as_ref()
            .map(|source| match expected {
                Some(ty) => self.expr_expected(source, ty),
                None => self.expr(source),
            })
            .transpose()?
            .unwrap_or(Expr::Null);
        self.pointer_conditional_pair(condition, yes, no, expected, span)
    }

    pub(crate) fn pointer_conditional_pair(
        &self,
        condition: BoolExpr,
        yes: Expr,
        no: Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let ty = expected
            .or(match &yes {
                Expr::Pointer {
                    ty, ..
                } => Some(*ty),
                _ => None,
            })
            .or(match &no {
                Expr::Pointer {
                    ty, ..
                } => Some(*ty),
                _ => None,
            })
            .ok_or_else(|| {
                Diagnostic::new(span, "null conditional requires a pointer type context")
            })?;
        let then_value = self.coerce_value(yes, ty, span)?;
        let else_value = self.coerce_value(no, ty, span)?;
        Ok(Expr::Pointer {
            ty,
            value: ValueExpr::Conditional {
                ty,
                expression: Box::new(Conditional {
                    condition,
                    then_value,
                    else_value,
                }),
            },
        })
    }
    pub(crate) fn address_expression(
        &mut self,
        source: &syntax::Expression,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(result) = self.overloaded_index_address(source, span) {
            return result;
        }
        if matches!(
            source.kind,
            syntax::ExpressionKind::Name(_)
                | syntax::ExpressionKind::QualifiedName(_)
                | syntax::ExpressionKind::Type(_)
                | syntax::ExpressionKind::AddressOf(_)
                | syntax::ExpressionKind::TypeQuery { .. }
        ) && let Expr::Type(pointee) = self.expr(source)?
        {
            let ty = self
                .types
                .pointer(pointee)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return Ok(Expr::Type(ty));
        }
        let place = self.expression_place(source)?;
        self.reject_iteration_write(place, span)?;
        let ty = self
            .types
            .pointer(place.ty())
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(Expr::Pointer {
            ty,
            value: ValueExpr::AddressOf {
                place,
                ty,
            },
        })
    }

    pub(crate) fn dereference_place(
        &mut self,
        source: &syntax::Expression,
        span: Span,
    ) -> Result<Place, Diagnostic> {
        let value = self.expr(source)?;
        let Expr::Pointer {
            value, ..
        } = value
        else {
            return Err(Diagnostic::new(
                span,
                "dereference requires a typed pointer",
            ));
        };
        self.places
            .dereference(value, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }

    pub(crate) fn dereference_expression(
        &mut self,
        source: &syntax::Expression,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let place = self.dereference_place(source, span)?;
        self.typed_value(ValueExpr::Load(place), place.ty(), span)
    }

    pub(crate) fn index_place(
        &mut self,
        base: &syntax::Expression,
        index: &syntax::Expression,
        span: Span,
    ) -> Result<Place, Diagnostic> {
        if let Some(result) = self.overloaded_index_place(base, index, span) {
            return result;
        }
        // Resolve addressable bases directly: constructing and discarding a
        // value first could execute a compile-time operand twice.
        let base_place = self.expression_place(base).ok();
        let base_value = match base_place {
            Some(place) => self.typed_value(ValueExpr::Load(place), place.ty(), base.span)?,
            None => self.expr(base)?,
        };
        let base_ty = self.expression_type(&base_value, base.span)?;
        let kind = self
            .types
            .kind(base_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        let index_value = self.sequence_index_integer(index)?;
        self.check_static_index(base_ty, &index_value, index)?;
        match kind {
            TypeKind::Pointer(element) => {
                if element == self.types.void() {
                    return Err(Diagnostic::new(
                        span,
                        "pointer indexing requires a sized pointee type",
                    ));
                }
                let base_value = self.coerce_value(base_value, base_ty, base.span)?;
                let pointer = ValueExpr::PointerOffset {
                    pointer: Box::new(base_value),
                    offset: self.pointer_index_offset(index_value),
                    subtract: false,
                    ty: base_ty,
                };
                self.places
                    .dereference(pointer, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))
            }
            TypeKind::FixedArray {
                ..
            }
            | TypeKind::Slice(_)
            | TypeKind::DynamicArray(_)
            | TypeKind::String => {
                let base = base_place.ok_or_else(|| {
                    Diagnostic::new(
                        base.span,
                        "assignment target does not denote mutable storage",
                    )
                })?;
                self.places
                    .index_with_check(base, index_value, self.checks.array_bounds, self.types)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))
            }
            _ => Err(Diagnostic::new(
                span,
                "indexing requires an array, string, slice, or pointer",
            )),
        }
    }

    pub(crate) fn index_expression(
        &mut self,
        base: &syntax::Expression,
        index: &syntax::Expression,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(result) =
            self.overloaded_operator(syntax::OperatorKind::Index, &[base, index], span)
        {
            return result;
        }
        if let Some(result) = self.overloaded_index_place(base, index, span) {
            let place = result?;
            return self.typed_value(ValueExpr::Load(place), place.ty(), span);
        }
        let base_place = self.expression_place(base).ok();
        if let Some(place) = base_place
            && matches!(
                self.types.kind(place.ty()),
                Ok(TypeKind::FixedArray { .. } | TypeKind::Slice(_) | TypeKind::DynamicArray(_))
            )
        {
            let index_source = index;
            let index = self.sequence_index_integer(index)?;
            self.check_static_index(place.ty(), &index, index_source)?;
            let place = self
                .places
                .index_with_check(place, index, self.checks.array_bounds, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return self.typed_value(ValueExpr::Load(place), place.ty(), span);
        }
        let base = match base_place {
            Some(place) => self.typed_value(ValueExpr::Load(place), place.ty(), base.span)?,
            None => self.expr(base)?,
        };
        let base_ty = self.expression_type(&base, span)?;
        let element = match self
            .types
            .kind(base_ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::FixedArray {
                element, ..
            }
            | TypeKind::Slice(element)
            | TypeKind::DynamicArray(element)
            | TypeKind::Pointer(element) => *element,
            TypeKind::String => self.types.scalar(ScalarType::Int(IntegerType::U8)),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "indexing requires an array, string, slice, or pointer",
                ));
            }
        };
        if element == self.types.void() {
            return Err(Diagnostic::new(
                span,
                "pointer indexing requires a sized pointee type",
            ));
        }
        let base = self.coerce_value(base, base_ty, span)?;
        let index_source = index;
        let index = self.sequence_index_integer(index)?;
        self.check_static_index(base_ty, &index, index_source)?;
        self.typed_value(
            ValueExpr::Index {
                base: Box::new(base),
                index,
                ty: element,
                check: self.checks.array_bounds,
            },
            element,
            span,
        )
    }

    /// Capture an address before the right operand; the final binary expression
    /// reads that captured place once and then evaluates the right operand.
    pub(crate) fn update_place(
        &mut self,
        target: &syntax::PlaceSyntax,
        operation: BinaryOp,
        source: &syntax::Expression,
    ) -> Result<Statement, Diagnostic> {
        if let Some(update) = self.overloaded_index_update(target, operation, source) {
            return update;
        }
        let source_target = target;
        let target_span = target.span;
        let target = self.resolve_place(target)?;
        self.reject_iteration_write(target, target_span)?;
        let pointer_ty = self
            .types
            .pointer(target.ty())
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
        let captured = self.allocate_typed(pointer_ty)?;
        let place = self
            .places
            .dereference(ValueExpr::Load(captured.place()), self.types)
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
        let update = match self.overloaded_compound_update(source_target, place, operation, source)
        {
            Some(update) => update?,
            None => {
                let left = self.typed_value(ValueExpr::Load(place), place.ty(), source.span)?;
                let right = self.compound_operand(source, place.ty(), operation)?;
                let result = self.binary(operation, left, right, source.span)?;
                Statement::Store(place, self.coerce_value(result, place.ty(), source.span)?)
            }
        };
        Ok(Statement::Block(Block {
            flow: Flow::FallsThrough,
            statements: vec![
                Statement::Store(
                    captured.place(),
                    ValueExpr::AddressOf {
                        place: target,
                        ty: pointer_ty,
                    },
                ),
                update,
            ],
        }))
    }

    pub(crate) fn pointer_binary(
        &mut self,
        operator: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if matches!(operator, BinaryOp::Add | BinaryOp::Subtract) {
            let void_type = |value: &Expr| match value {
                Expr::Pointer {
                    ty, ..
                } if matches!(self.types.kind(*ty), Ok(TypeKind::Pointer(element)) if *element == self.types.void()) => {
                    Some(*ty)
                }
                _ => None,
            };
            let left_void = void_type(&left);
            let right_void = void_type(&right);
            let difference =
                operator == BinaryOp::Subtract && left_void.is_some() && right_void == left_void;
            let left_offset =
                left_void.is_some() && !matches!(right, Expr::Pointer { .. } | Expr::Null);
            let right_offset = operator == BinaryOp::Add
                && right_void.is_some()
                && !matches!(left, Expr::Pointer { .. } | Expr::Null);
            if difference || left_offset || right_offset {
                let byte = self.types.scalar(ScalarType::Int(IntegerType::U8));
                let byte_pointer = self
                    .types
                    .pointer(byte)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                let left = if left_void.is_some() {
                    self.pointer_from_operand(left, byte_pointer, CastMode::Checked, span)?
                } else {
                    left
                };
                let right = if right_void.is_some() {
                    self.pointer_from_operand(right, byte_pointer, CastMode::Checked, span)?
                } else {
                    right
                };
                let result = self.pointer_binary(operator, left, right, span)?;
                return match left_void.or(right_void) {
                    Some(target) if !difference => {
                        self.pointer_from_operand(result, target, CastMode::Checked, span)
                    }
                    _ => Ok(result),
                };
            }
        }
        match Operator::from(operator) {
            Operator::And => {
                return Ok(Expr::Bool(BoolExpr::And(
                    Box::new(left.condition(span, self.types)?),
                    Box::new(right.condition(span, self.types)?),
                )));
            }
            Operator::Or => {
                return Ok(Expr::Bool(BoolExpr::Or(
                    Box::new(left.condition(span, self.types)?),
                    Box::new(right.condition(span, self.types)?),
                )));
            }
            Operator::Equality(operation) => {
                let (ty, left, right) = match (left, right) {
                    (Expr::Null, Expr::Null) => {
                        return Ok(Expr::Bool(BoolExpr::Constant(operation == Equality::Equal)));
                    }
                    (
                        Expr::Pointer {
                            ty,
                            value,
                        },
                        Expr::Null,
                    ) => (ty, value, ValueExpr::Zero(ty)),
                    (
                        Expr::Null,
                        Expr::Pointer {
                            ty,
                            value,
                        },
                    ) => (ty, ValueExpr::Zero(ty), value),
                    (
                        Expr::Pointer {
                            ty,
                            value: left,
                        },
                        Expr::Pointer {
                            ty: right_ty,
                            value: right,
                        },
                    ) if ty == right_ty => (ty, left, right),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "pointer equality requires matching pointer types or null",
                        ));
                    }
                };
                debug_assert!(matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_))));
                return Ok(Expr::Bool(BoolExpr::ComparePointers(
                    operation,
                    Box::new(left),
                    Box::new(right),
                )));
            }
            _ => {}
        }
        let (ty, pointer, offset, subtract) = match (operator, left, right) {
            (
                BinaryOp::Subtract,
                Expr::Pointer {
                    ty: left_ty,
                    value: left,
                },
                Expr::Pointer {
                    ty: right_ty,
                    value: right,
                },
            ) => {
                if left_ty != right_ty {
                    return Err(Diagnostic::new(
                        span,
                        "pointer difference requires matching pointee types",
                    ));
                }
                return Ok(Expr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::PointerDifference {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                )));
            }
            (
                BinaryOp::Add,
                Expr::Pointer {
                    ty,
                    value,
                },
                offset,
            ) => (ty, value, offset, false),
            (
                BinaryOp::Subtract,
                Expr::Pointer {
                    ty,
                    value,
                },
                offset,
            ) => (ty, value, offset, true),
            (
                BinaryOp::Add,
                offset,
                Expr::Pointer {
                    ty,
                    value,
                },
            ) => {
                let TypeKind::Pointer(element) = self
                    .types
                    .kind(ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    unreachable!()
                };
                if *element == self.types.void() {
                    return Err(Diagnostic::new(
                        span,
                        "pointer arithmetic requires a sized pointee type",
                    ));
                }
                let offset = offset.int_as(IntegerType::S64, span)?;
                return Ok(Expr::Pointer {
                    ty,
                    value: ValueExpr::PointerOffsetLeft {
                        offset,
                        pointer: Box::new(value),
                        ty,
                    },
                });
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "pointer arithmetic supports pointer plus or minus an integer",
                ));
            }
        };
        let TypeKind::Pointer(element) = self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        else {
            unreachable!()
        };
        if *element == self.types.void() {
            return Err(Diagnostic::new(
                span,
                "pointer arithmetic requires a sized pointee type",
            ));
        }
        let offset = offset.int_as(IntegerType::S64, span)?;
        Ok(Expr::Pointer {
            ty,
            value: ValueExpr::PointerOffset {
                pointer: Box::new(pointer),
                offset,
                subtract,
                ty,
            },
        })
    }
}
