//! Capture an actual written condition subject before reusing its checked value.
use super::*;
use std::ptr::NonNull;

pub(crate) struct CapturedConditionalSubject {
    node: NonNull<syntax::Expression>,
    value: Expr,
}
impl Resolver<'_> {
    pub(crate) fn captured_conditional_subject(&self, source: &syntax::Expression) -> Option<Expr> {
        let node = NonNull::from(source);
        self.conditional_subjects
            .iter()
            .rev()
            .find(|capture| capture.node == node)
            .map(|capture| capture.value.clone())
    }
    pub(crate) fn source_conditional(
        &mut self,
        source: &syntax::ConditionalExpression,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let mut bindings = Vec::new();
        let (condition, mut yes) = if source.is_implicit() {
            // The selected subject lies on the first evaluation path. Capturing an
            // indirect callee's argument here would move its read behind that producer.
            self.validate_implicit_subject_order(&source.condition, source.then_source())?;
            let subject = source.then_source();
            let value = match expected {
                Some(ty) if crate::inferred_casts::needs_cast_context(subject) => {
                    self.expr_expected(subject, ty)?
                }
                _ => self.expr(subject)?,
            };
            let captured = if matches!(value, Expr::Literal(_) | Expr::WeakFloat(_) | Expr::Null) {
                value.clone()
            } else {
                let ty = self.expression_type(&value, subject.span)?;
                let producer = self.coerce_value(value.clone(), ty, subject.span)?;
                let binding = self.allocate_expression_binding(subject.span)?;
                self.capture_expression_value_contract(binding, subject, &producer, subject.span)?;
                bindings.push((binding, producer));
                self.typed_value(
                    ValueExpr::Bound {
                        binding,
                        ty,
                    },
                    ty,
                    subject.span,
                )?
            };
            if self.conditional_subjects.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
                return Err(Diagnostic::new(span, "implicit ifx capture depth exceeded"));
            }
            let checkpoint = self.conditional_subjects.len();
            self.conditional_subjects.push(CapturedConditionalSubject {
                node: NonNull::from(subject),
                value: captured.clone(),
            });
            // No pointer is dereferenced; exact node equality only selects the bound
            // value during this call. Truncation retires the capture on every Result.
            let condition = self.condition_expression(&source.condition);
            self.conditional_subjects.truncate(checkpoint);
            (condition?, captured)
        } else {
            (
                self.condition_expression(&source.condition)?,
                match expected {
                    Some(ty) => self.expr_expected(source.then_source(), ty)?,
                    None => self.expr(source.then_source())?,
                },
            )
        };
        if source.is_implicit()
            && let Some(ty) = expected
        {
            yes = if matches!(self.types.kind(ty), Ok(jai_types::TypeKind::Any(_))) {
                self.box_any_expression(yes, ty, span)?
            } else {
                let value = self.coerce_value(yes, ty, span)?;
                self.typed_value(value, ty, span)?
            };
        }
        let no = source
            .else_value
            .as_ref()
            .map(|value| match expected {
                Some(ty) => self.expr_expected(value, ty),
                None => self.expr(value),
            })
            .transpose()?;
        let result = match expected {
            Some(ty) if matches!(self.types.kind(ty), Ok(jai_types::TypeKind::Pointer(_))) => self
                .pointer_conditional_pair(
                    condition,
                    yes,
                    no.unwrap_or(Expr::Null),
                    Some(ty),
                    span,
                )?,
            Some(ty) => self.value_conditional_pair(condition, yes, no, ty, span)?,
            None => self.inferred_conditional_pair(condition, yes, no, span)?,
        };
        if bindings.is_empty() {
            return Ok(result);
        }
        let ty = self.expression_type(&result, span)?;
        let body = self.coerce_value(result, ty, span)?;
        self.typed_value(
            ValueExpr::Bind {
                bindings,
                body: Box::new(body),
                ty,
            },
            ty,
            span,
        )
    }
    fn validate_implicit_subject_order(
        &mut self,
        condition: &syntax::Expression,
        subject: &syntax::Expression,
    ) -> Result<(), Diagnostic> {
        let mut current = condition;
        while !std::ptr::eq(current, subject) {
            current = match &current.kind {
                syntax::ExpressionKind::Unary(_, value)
                | syntax::ExpressionKind::CallHint {
                    call: value, ..
                } => value,
                syntax::ExpressionKind::Binary(_, left, _) => left,
                syntax::ExpressionKind::Call(name, arguments) => {
                    let path = syntax::NamePath {
                        root: *name,
                        members: vec![],
                    };
                    self.reject_value_expansion(&path, current.span)?;
                    if self.is_constant_query(&path) {
                        return Err(Diagnostic::new(
                            current.span,
                            "implicit ifx cannot extract a runtime subject from is_constant; write an explicit then arm",
                        ));
                    }
                    if self.call_is_indirect(&path, current.span) {
                        return Err(Diagnostic::new(
                            current.span,
                            "implicit ifx first-argument extraction from an indirect callback requires an explicit then arm",
                        ));
                    }
                    &arguments
                        .first()
                        .ok_or_else(|| {
                            Diagnostic::new(
                                current.span,
                                "implicit ifx call subject is missing its first argument",
                            )
                        })?
                        .value
                }
                syntax::ExpressionKind::QualifiedCall(path, arguments) => {
                    self.reject_value_expansion(path, current.span)?;
                    if self.is_constant_query(path) {
                        return Err(Diagnostic::new(
                            current.span,
                            "implicit ifx cannot extract a runtime subject from is_constant; write an explicit then arm",
                        ));
                    }
                    if self.call_is_indirect(path, current.span) {
                        return Err(Diagnostic::new(
                            current.span,
                            "implicit ifx first-argument extraction from an indirect callback requires an explicit then arm",
                        ));
                    }
                    &arguments
                        .first()
                        .ok_or_else(|| {
                            Diagnostic::new(
                                current.span,
                                "implicit ifx call subject is missing its first argument",
                            )
                        })?
                        .value
                }
                _ => {
                    return Err(Diagnostic::new(
                        current.span,
                        "implicit ifx subject is not on the ordered condition path",
                    ));
                }
            };
        }
        Ok(())
    }
}

impl Resolver<'_> {
    fn inferred_conditional_pair(
        &mut self,
        condition: BoolExpr,
        yes: Expr,
        no: Option<Expr>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if matches!(yes, Expr::Pointer { .. } | Expr::Null)
            || no
                .as_ref()
                .is_some_and(|value| matches!(value, Expr::Pointer { .. } | Expr::Null))
        {
            return self.pointer_conditional_pair(
                condition,
                yes,
                no.unwrap_or(Expr::Null),
                None,
                span,
            );
        }
        if yes.has_float() || no.as_ref().is_some_and(Expr::has_float) {
            return Ok(Expr::WeakConditional(Box::new(Conditional {
                condition,
                then_value: yes,
                else_value: no.unwrap_or(Expr::Literal(0)),
            })));
        }
        if matches!(yes, Expr::Type(_)) {
            let ty = self.types.meta_type();
            let yes = self.runtime_type_expression(yes, span)?;
            let no = no
                .map(|value| self.runtime_type_expression(value, span))
                .transpose()?;
            return self.value_conditional_pair(condition, yes, no, ty, span);
        }
        if matches!(yes, Expr::Typed { .. } | Expr::Enum { .. }) {
            let ty = self.expression_type(&yes, span)?;
            let no = if ty == self.types.meta_type() {
                no.map(|value| self.runtime_type_expression(value, span))
                    .transpose()?
            } else {
                no
            };
            return self.value_conditional_pair(condition, yes, no, ty, span);
        }
        Ok(match yes {
            Expr::Bool(yes) => Expr::Bool(BoolExpr::Conditional(Box::new(Conditional {
                condition,
                then_value: yes,
                else_value: match no {
                    Some(e) => e.bool(span)?,
                    None => BoolExpr::Constant(false),
                },
            }))),
            Expr::Type(_)
            | Expr::Code(_)
            | Expr::Typed {
                ..
            }
            | Expr::Enum {
                ..
            }
            | Expr::Pointer {
                ..
            }
            | Expr::Null => {
                return Err(Diagnostic::new(
                    span,
                    "nominal conditional values are not implemented",
                ));
            }
            Expr::Void(_)
            | Expr::IndirectVoid {
                ..
            } => {
                return Err(Diagnostic::new(
                    span,
                    "void call cannot supply an ifx result",
                ));
            }
            yes => {
                let no = no.unwrap_or(Expr::Literal(0));
                if yes.weak_integer() && no.weak_integer() {
                    return Ok(Expr::WeakConditional(Box::new(Conditional {
                        condition,
                        then_value: yes,
                        else_value: no,
                    })));
                }
                let (yes, no) = Self::integer_pair(yes, no, span)?;
                Expr::Int(IntExpr::new(
                    yes.ty(),
                    IntExprKind::Conditional(Box::new(Conditional {
                        condition,
                        then_value: yes,
                        else_value: no,
                    })),
                ))
            }
        })
    }
}
