//! Descriptor slots use sequence fields rather than nominal record identities.
use super::*;

impl Resolver<'_> {
    pub(crate) fn positional_sequence_literal(
        &mut self,
        literal: &syntax::PositionalStructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let (element, capacity) = match *self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Slice(element) => (element, false),
            TypeKind::DynamicArray(element) => (element, true),
            TypeKind::String => (self.types.scalar(ScalarType::Int(IntegerType::U8)), false),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "positional descriptor requires a sequence type",
                ));
            }
        };
        let count_ty = self.types.scalar(ScalarType::Int(IntegerType::S64));
        let pointer_ty = self
            .types
            .pointer(element)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let mut slots = vec![
            (SequenceField::Count, count_ty),
            (SequenceField::Data, pointer_ty),
        ];
        if capacity {
            slots.push((SequenceField::Allocated, count_ty));
        }
        if literal.values.len() > slots.len() {
            return Err(Diagnostic::new(
                span,
                "positional sequence descriptor has too many values",
            ));
        }
        let mut initializers = Vec::with_capacity(literal.values.len());
        let mut bindings = Vec::with_capacity(literal.values.len());
        for (source, (field, target)) in literal.values.iter().zip(slots) {
            let value = self.expr_expected(source, target)?;
            let mut value = self.coerce_value(value, target, source.span)?;
            if !crate::modules::aggregates::concrete_literal(&value, source.span)? {
                let binding = self.allocate_expression_binding(source.span)?;
                self.capture_expression_value_contract(binding, source, &value, source.span)?;
                bindings.push((binding, value));
                value = ValueExpr::Bound {
                    binding,
                    ty: target,
                };
            }
            initializers.push((field, value));
        }
        let body = ValueExpr::SequenceBuild { ty, initializers };
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
