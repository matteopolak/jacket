//! Stage with typed literal targets; validate all descriptor roles before RHSs.
use super::*;

impl Resolver<'_> {
    pub(crate) fn sequence_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let mut slots = Vec::with_capacity(literal.fields.len());
        let mut seen = std::collections::HashSet::new();
        for source in &literal.fields {
            let name =
                crate::modules::aggregates::promoted_literals::descriptor_target(&source.target)?;
            let slot = self.sequence_field_type(ty, name, source.span)?;
            if !seen.insert(slot.0) {
                return Err(Diagnostic::new(
                    source.span,
                    "duplicate sequence descriptor field",
                ));
            }
            slots.push(slot);
        }
        let mut initializers = Vec::with_capacity(slots.len());
        let mut bindings = Vec::with_capacity(slots.len());
        for (source, (field, target)) in literal.fields.iter().zip(slots) {
            let value = self.expr_expected(&source.value, target)?;
            let mut value = self.coerce_value(value, target, source.span)?;
            if !crate::modules::aggregates::concrete_literal(&value, source.span)? {
                let binding = self.allocate_expression_binding(source.span)?;
                self.capture_expression_value_contract(
                    binding,
                    &source.value,
                    &value,
                    source.span,
                )?;
                bindings.push((binding, value));
                value = ValueExpr::Bound {
                    binding,
                    ty: target,
                };
            }
            initializers.push((field, value));
        }
        let body = ValueExpr::SequenceBuild {
            ty,
            initializers,
        };
        self.typed_value(
            if bindings.is_empty() {
                body
            } else {
                ValueExpr::Bind {
                    bindings,
                    body: Box::new(body),
                    ty,
                }
            },
            ty,
            span,
        )
    }
}
