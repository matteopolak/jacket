//! Staged descriptor target adapter; preserves genuine AnySchema fields.
use super::*;
impl Resolver<'_> {
    pub(crate) fn any_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let schema = self.any_schema(ty, span)?;
        let mut slots = Vec::with_capacity(literal.fields.len());
        let mut seen = [false; 2];
        for initializer in &literal.fields {
            let name = crate::modules::aggregates::promoted_literals::descriptor_target(
                &initializer.target,
            )?;
            let field = match self.symbols.name(name) {
                "type" => AnyField::Type,
                "value_pointer" => AnyField::ValuePointer,
                _ => {
                    return Err(Diagnostic::new(
                        initializer.span,
                        "unknown Any descriptor member",
                    ));
                }
            };
            let index = match field {
                AnyField::Type => 0,
                AnyField::ValuePointer => 1,
            };
            if std::mem::replace(&mut seen[index], true) {
                return Err(Diagnostic::new(
                    initializer.span,
                    "duplicate Any descriptor member",
                ));
            }
            let field = schema.field(field);
            slots.push((initializer, field));
        }
        let mut initializers = Vec::with_capacity(2);
        let mut bindings = Vec::with_capacity(slots.len());
        for (source, field) in slots {
            let value = self.expr_expected(&source.value, field.ty)?;
            let mut value = self.coerce_value(value, field.ty, source.span)?;
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
                    ty: field.ty,
                };
            }
            initializers.push((field.id, value));
        }
        for (index, choice) in [AnyField::Type, AnyField::ValuePointer]
            .into_iter()
            .enumerate()
        {
            if !seen[index] {
                let field = schema.field(choice);
                initializers.push((field.id, ValueExpr::Zero(field.ty)));
            }
        }
        let body = ValueExpr::RecordBuild {
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
