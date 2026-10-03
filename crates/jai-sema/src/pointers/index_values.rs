//! Preserve integer index domains until descriptor and target checks run.
use super::*;

impl Resolver<'_> {
    pub(super) fn sequence_index_integer(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<IntExpr, Diagnostic> {
        let value = if matches!(source.kind, syntax::ExpressionKind::InferredCast { .. }) {
            self.expr_expected(source, self.types.scalar(ScalarType::Int(IntegerType::S64)))?
        } else {
            self.expr(source)?
        };
        let domain = match &value {
            Expr::Int(index) => jai_ir::canonical_index_type(index.ty()),
            Expr::Literal(value) if *value > IntegerType::S64.max() => IntegerType::U64,
            Expr::Literal(_) | Expr::WeakConditional(_) => IntegerType::S64,
            _ => {
                return Err(Diagnostic::new(
                    source.span,
                    "sequence index requires an integer value",
                ));
            }
        };
        value.int_as(domain, source.span)
    }

    // Raw pointer offset IR has a signed displacement. The conversion is
    // checked explicitly; sequence descriptors never use this narrowing.
    pub(super) fn pointer_index_offset(&self, index: IntExpr) -> IntExpr {
        if index.ty() == IntegerType::S64 {
            index
        } else {
            IntExpr::new(
                IntegerType::S64,
                IntExprKind::Cast(CastMode::Checked, Box::new(index)),
            )
        }
    }
}
