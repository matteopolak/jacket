//! Positional values capture in written order before physical field grouping.
use super::PreparedLiteral;
use crate::{Diagnostic, Expr, Resolver, Span};
use jai_syntax as syntax;
use jai_types::TypeId;

impl Resolver<'_> {
    pub(crate) fn prepared_positional_record_literal(
        &mut self,
        literal: &syntax::PositionalStructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let mut leaves = Vec::with_capacity(literal.values.len());
        let mut producers = Vec::with_capacity(literal.values.len());
        for (index, source) in literal.values.iter().enumerate() {
            let field = self
                .types
                .field(ty, index)
                .map_err(|error| Diagnostic::new(source.span, error.to_string()))?
                .id;
            let target = self
                .types
                .validate_field(ty, field)
                .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
            leaves.push((vec![field], target));
            producers.push((source, target, source.span));
        }
        let plan = PreparedLiteral::new(self.types, ty, leaves).map_err(|error| {
            Diagnostic::new(span, format!("invalid positional record path: {error:?}"))
        })?;
        self.lower_prepared_record_literal(plan, &producers, ty, span)
    }
}
