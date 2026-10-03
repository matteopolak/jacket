//! Root-only operator result checks consume the already checked expression.
use super::*;

impl Resolver<'_> {
    pub(crate) fn check_bound_operator_result_use(
        &self,
        source: &syntax::Expression,
        value: &ValueExpr,
        used: &[bool],
        offset: usize,
    ) -> Result<(), Diagnostic> {
        if !matches!(
            source.kind,
            syntax::ExpressionKind::Unary(_, _)
                | syntax::ExpressionKind::Binary(_, _, _)
                | syntax::ExpressionKind::Index { .. }
        ) {
            return Ok(());
        }
        if let Some(call) = root_call(value, source.span)? {
            self.check_bound_call_result_use(call, used, offset, source.span)?;
        }
        Ok(())
    }

    pub(crate) fn check_discarded_bound_operator(
        &self,
        source: &syntax::Expression,
        value: &Expr,
    ) -> Result<(), Diagnostic> {
        let value = match value {
            Expr::Int(value) => match value.kind() {
                IntExprKind::Value(value) => Some(value.as_ref()),
                _ => None,
            },
            Expr::Float(value) => match value.kind() {
                FloatExprKind::Value(value) => Some(value.as_ref()),
                _ => None,
            },
            Expr::Bool(BoolExpr::Value(value)) => Some(value.as_ref()),
            Expr::Typed {
                value, ..
            }
            | Expr::Enum {
                value, ..
            }
            | Expr::Pointer {
                value, ..
            } => Some(value),
            _ => None,
        };
        if let Some(value) = value {
            self.check_bound_operator_result_use(source, value, &[], 0)?;
        }
        Ok(())
    }
}

// Only transparent captures are followed. An operand, cast, condition or
// dereference that consumes a call's result does not become a root discard.
fn root_call(value: &ValueExpr, span: Span) -> Result<Option<&Call>, Diagnostic> {
    let mut value = value;
    for _ in 0..crate::constant_limits::MAX_CONSTANT_DEPTH {
        match value {
            ValueExpr::Call {
                call, ..
            } => return Ok(Some(call)),
            ValueExpr::Bind {
                bindings,
                body,
                ..
            } => {
                value = match body.as_ref() {
                    ValueExpr::Bound {
                        binding, ..
                    } => {
                        let Some((_, producer)) =
                            bindings.iter().find(|(producer, _)| producer == binding)
                        else {
                            return Err(Diagnostic::new(
                                span,
                                "operator result capture has no checked producer",
                            ));
                        };
                        producer
                    }
                    other => other,
                };
            }
            _ => return Ok(None),
        }
    }
    Err(Diagnostic::new(
        span,
        "operator result capture exceeds expression depth",
    ))
}
