//! Instance namespace access preserves canonical members and receiver effects.
use super::*;

impl Resolver<'_> {
    pub(crate) fn instance_namespace_member(
        &mut self,
        owner: TypeId,
        member: Symbol,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        if let Some(binding) = self.specialized_instance_record_member(owner, member, span)? {
            return Ok(Some(binding));
        }
        // The specialization's evaluation overlay also contains defining outer
        // names. They are not additional members of this runtime instance.
        if self.meta.record_specializations.record(owner).is_some() {
            return Ok(None);
        }
        self.type_namespace_member(owner, member, span)
    }

    pub(super) fn instance_namespace_value(
        &mut self,
        receiver: ValueExpr,
        binding: Binding,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let expression = self.binding_expression(binding, span)?;
        if receiver_is_simple(&receiver) {
            return Ok(expression);
        }
        if matches!(expression, Expr::Type(_) | Expr::Code(_)) {
            return Err(Diagnostic::new(
                span,
                "effectful receiver of a compile-time record namespace member requires source type-expression evaluation",
            ));
        }
        let ty = self.expression_type(&expression, span)?;
        let value = self.coerce_value(expression, ty, span)?;
        let binding = self.allocate_expression_binding(span)?;
        self.typed_value(
            ValueExpr::Bind {
                bindings: vec![(binding, receiver)],
                body: Box::new(value),
                ty,
            },
            ty,
            span,
        )
    }
}

fn receiver_is_simple(receiver: &ValueExpr) -> bool {
    jai_ir::is_static_value(receiver)
        || matches!(receiver, ValueExpr::Load(place)
            if matches!(place.kind(), jai_ir::PlaceKind::Local(_) | jai_ir::PlaceKind::Global(_)))
}
