//! Flags retain nominal identity while their operators use the declared representation.
use super::*;

impl Resolver<'_> {
    pub(crate) fn enum_binary(
        &self,
        operator: BinaryOp,
        a: Expr,
        b: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if matches!(operator, BinaryOp::Equal | BinaryOp::NotEqual) {
            let flags_zero = match (&a, &b) {
                (
                    Expr::Enum {
                        ty, ..
                    },
                    Expr::Literal(0),
                )
                | (
                    Expr::Literal(0),
                    Expr::Enum {
                        ty, ..
                    },
                ) => self.enum_is_flags(*ty),
                _ => false,
            };
            if flags_zero {
                return Self::flags_zero_comparison(operator, a, b, span);
            }
        }
        let (
            Expr::Enum {
                ty: a_ty,
                representation,
                value: a,
                ..
            },
            Expr::Enum {
                ty: b_ty,
                value: b,
                ..
            },
        ) = (a, b)
        else {
            return Err(Diagnostic::new(
                span,
                "enum operation requires values of the same nominal type",
            ));
        };
        if a_ty != b_ty {
            return Err(Diagnostic::new(
                span,
                "enum operation requires values of the same nominal type",
            ));
        }
        let a = IntExpr::new(representation, IntExprKind::EnumValue(Box::new(a)));
        let b = IntExpr::new(representation, IntExprKind::EnumValue(Box::new(b)));
        match Operator::from(operator) {
            Operator::Equality(operator) => Ok(Expr::Bool(BoolExpr::CompareInts(
                match operator {
                    Equality::Equal => Relation::Equal,
                    Equality::NotEqual => Relation::NotEqual,
                },
                Box::new(a),
                Box::new(b),
            ))),
            Operator::Relation(operator) => Ok(Expr::Bool(BoolExpr::CompareInts(
                operator,
                Box::new(a),
                Box::new(b),
            ))),
            Operator::Integer(operator)
                if matches!(operator, IntOp::BitAnd | IntOp::BitOr | IntOp::BitXor) =>
            {
                if !self.enum_is_flags(a_ty) {
                    return Err(Diagnostic::new(
                        span,
                        "bitwise enum operation requires an enum_flags type",
                    ));
                }
                Ok(Expr::Enum {
                    ty: a_ty,
                    flags: true,
                    representation,
                    value: ValueExpr::EnumFromInt {
                        ty: a_ty,
                        value: IntExpr::new(
                            representation,
                            IntExprKind::Binary(operator, Box::new(a), Box::new(b)),
                        ),
                    },
                })
            }
            _ => Err(Diagnostic::new(
                span,
                "enum arithmetic requires an explicit integer conversion",
            )),
        }
    }

    fn flags_zero_comparison(
        operator: BinaryOp,
        a: Expr,
        b: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let representation = match (&a, &b) {
            (
                Expr::Enum {
                    representation, ..
                },
                _,
            )
            | (
                _,
                Expr::Enum {
                    representation, ..
                },
            ) => *representation,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "flags comparison requires an enum_flags value",
                ));
            }
        };
        let operand = |value| match value {
            Expr::Enum {
                value, ..
            } => IntExpr::new(representation, IntExprKind::EnumValue(Box::new(value))),
            Expr::Literal(0) => IntExpr::constant(IntegerValue::wrapping(representation, 0)),
            _ => unreachable!("flags-zero operands were checked before lowering"),
        };
        Ok(Expr::Bool(BoolExpr::CompareInts(
            if operator == BinaryOp::Equal {
                Relation::Equal
            } else {
                Relation::NotEqual
            },
            Box::new(operand(a)),
            Box::new(operand(b)),
        )))
    }

    pub(crate) fn enum_unary(
        &self,
        operator: UnaryOp,
        value: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Enum {
            ty,
            representation,
            value,
            ..
        } = value
        else {
            return Err(Diagnostic::new(
                span,
                "enum unary operation requires an enum value",
            ));
        };
        if operator != UnaryOp::Complement || !self.enum_is_flags(ty) {
            return Err(Diagnostic::new(
                span,
                "enum unary operation requires complement on an enum_flags type",
            ));
        }
        Ok(Expr::Enum {
            ty,
            flags: true,
            representation,
            value: ValueExpr::EnumFromInt {
                ty,
                value: IntExpr::new(
                    representation,
                    IntExprKind::Complement(Box::new(IntExpr::new(
                        representation,
                        IntExprKind::EnumValue(Box::new(value)),
                    ))),
                ),
            },
        })
    }

    pub(crate) fn contextual_enum_operand(expression: &syntax::Expression) -> bool {
        let mut expression = expression;
        for _ in 0..128 {
            match &expression.kind {
                syntax::ExpressionKind::InferredMember(_) => return true,
                syntax::ExpressionKind::Unary(UnaryOp::Complement, inner) => expression = inner,
                _ => return false,
            }
        }
        false
    }

    pub(crate) fn resolve_contextual_enum_operand(
        &self,
        expression: &syntax::Expression,
        ty: TypeId,
    ) -> Result<Expr, Diagnostic> {
        let mut inner = expression;
        let mut complements = Vec::new();
        for _ in 0..128 {
            match &inner.kind {
                syntax::ExpressionKind::InferredMember(name) => {
                    let mut value = self.inferred_enum_member(ty, *name, inner.span)?;
                    for span in complements.into_iter().rev() {
                        value = self.enum_unary(UnaryOp::Complement, value, span)?;
                    }
                    return Ok(value);
                }
                syntax::ExpressionKind::Unary(UnaryOp::Complement, value) => {
                    complements.push(inner.span);
                    inner = value;
                }
                _ => {
                    return Err(Diagnostic::new(
                        expression.span,
                        "expression does not have a contextual enum member",
                    ));
                }
            }
        }
        Err(Diagnostic::new(
            expression.span,
            "contextual enum mask exceeds the supported nesting depth",
        ))
    }
}
