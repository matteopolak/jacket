//! Force casts retain actual source bytes instead of performing numeric conversion.
use super::*;
use jai_ir::StorageBitcastSource;
use jai_types::{StorageBitcast, StorageBitcastStrength};

impl Resolver<'_> {
    pub(crate) fn storage_cast_operand(
        &mut self,
        operand: Expr,
        target: TypeId,
        strength: StorageBitcastStrength,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let source_type = self.expression_type(&operand, span)?;
        let policy = self.target_layout.ok_or_else(|| {
            Diagnostic::new(
                span,
                "storage cast is waiting for the compilation target layout",
            )
        })?;
        let cast = StorageBitcast::prove(self.types, policy, source_type, target, strength)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let operand = self.runtime_type_expression(operand, span)?;
        let value = self.coerce_value(operand, source_type, span)?;
        // Recover the already-resolved place, including evaluated address/index IR.
        // Re-resolving source syntax would duplicate compiler effects and calls.
        let source = match self.boxed_value_place(&value, span)? {
            Some(place) if place.ty() == source_type => StorageBitcastSource::Place(place),
            _ => StorageBitcastSource::Value(Box::new(value)),
        };
        self.typed_value(ValueExpr::StorageBitcast { source, cast }, target, span)
    }
}
