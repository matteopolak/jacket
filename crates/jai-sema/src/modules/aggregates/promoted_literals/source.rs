//! Source initializer order is captured before pure canonical field grouping.
use super::{
    FieldDefaults, PreparedLiteral, build_diagnostic, path_diagnostic, tree::PreparedLeaf,
};
use crate::{BoolExpr, Diagnostic, Expr, FloatExprKind, IntExprKind, Resolver, Span, ValueExpr};
use jai_syntax as syntax;
use jai_types::{FieldId, TypeId};

impl Resolver<'_> {
    pub(crate) fn promoted_record_literal(
        &mut self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let (plan, targets) = self.prepare_promoted_literal(literal, ty, span)?;
        let producers = literal
            .fields
            .iter()
            .zip(targets)
            .map(|(source, target)| (&source.value, target, source.span))
            .collect::<Vec<_>>();
        self.lower_prepared_record_literal(plan, &producers, ty, span)
    }

    pub(super) fn lower_prepared_record_literal(
        &mut self,
        plan: PreparedLiteral,
        producers: &[(&syntax::Expression, TypeId, Span)],
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let mut bindings = Vec::with_capacity(producers.len());
        let mut values = Vec::with_capacity(producers.len());
        for &(source, target, source_span) in producers {
            let value = self.expr_expected(source, target)?;
            let value = self.coerce_value(value, target, source_span)?;
            if concrete_literal(&value, source_span)? {
                values.push(PreparedLeaf::constant(
                    self.literal_constant(value, source_span)?,
                ));
                continue;
            }
            let binding = self.allocate_expression_binding(source_span)?;
            self.capture_expression_value_contract(binding, source, &value, source_span)?;
            bindings.push((binding, value));
            values.push(PreparedLeaf::bound(binding, target));
        }
        let body = plan
            .compose(
                self.types,
                values,
                &mut SourceDefaults {
                    resolver: self,
                    span,
                },
            )
            .map_err(|error| match build_diagnostic(error, span) {
                Ok(error) | Err(error) => error,
            })?;
        Ok(Expr::Typed {
            ty,
            value: if bindings.is_empty() {
                body
            } else {
                ValueExpr::Bind {
                    bindings,
                    body: Box::new(body),
                    ty,
                }
            },
        })
    }

    fn prepare_promoted_literal(
        &self,
        literal: &syntax::StructLiteral,
        ty: TypeId,
        span: Span,
    ) -> Result<(PreparedLiteral, Vec<TypeId>), Diagnostic> {
        let mut leaves = Vec::with_capacity(literal.fields.len());
        let mut targets = Vec::with_capacity(literal.fields.len());
        for source in &literal.fields {
            let path = crate::record_default_overrides::find_field_path(
                ty,
                source.name,
                source.span,
                self.types,
                |owner| self.record_metadata(owner, source.span),
            )?;
            let mut target = ty;
            for &field in &path {
                target = self
                    .types
                    .validate_field(target, field)
                    .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
            }
            targets.push(target);
            leaves.push((path, target));
        }
        let plan = PreparedLiteral::new(self.types, ty, leaves)
            .map_err(|error| path_diagnostic(error, literal, span))?;
        Ok((plan, targets))
    }
}

// This admits only actual literal storage. Arithmetic, casts, calls and loads
// keep their evaluation position even when another visitor calls them static.
pub(crate) fn concrete_literal(value: &ValueExpr, span: Span) -> Result<bool, Diagnostic> {
    let mut pending = vec![(value, 0)];
    let mut remaining = crate::constant_limits::MAX_CONSTANT_CELLS;
    while let Some((value, depth)) = pending.pop() {
        if remaining == 0 || depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                span,
                "literal constant exceeds compiler budget",
            ));
        }
        remaining -= 1;
        match value {
            ValueExpr::Int(value) if matches!(value.kind(), IntExprKind::Constant(_)) => {}
            ValueExpr::Float(value) if matches!(value.kind(), FloatExprKind::Constant(_)) => {}
            ValueExpr::Bool(BoolExpr::Constant(_))
            | ValueExpr::Enum { .. }
            | ValueExpr::ProcedureValue { .. }
            | ValueExpr::NativePointer(_)
            | ValueExpr::RuntimeType(_)
            | ValueExpr::Zero(_)
            | ValueExpr::StringBytes { .. } => {}
            ValueExpr::Record { fields, .. }
            | ValueExpr::Array {
                elements: fields, ..
            } => {
                pending.extend(fields.iter().map(|field| (field, depth + 1)));
            }
            ValueExpr::RecordBuild { initializers, .. } => {
                pending.extend(initializers.iter().map(|(_, value)| (value, depth + 1)));
            }
            ValueExpr::Union { value, .. } | ValueExpr::Distinct { value, .. } => {
                pending.push((value, depth + 1));
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

struct SourceDefaults<'r, 's> {
    resolver: &'r Resolver<'s>,
    span: Span,
}
impl FieldDefaults for SourceDefaults<'_, '_> {
    type Error = Diagnostic;
    fn complete(&mut self, field: FieldId) -> Result<jai_ir::ConstantValue, Diagnostic> {
        self.resolver.field_default_value(field, self.span)
    }
    fn partial(&mut self, field: FieldId) -> Result<Option<jai_ir::ConstantValue>, Diagnostic> {
        self.resolver.field_construction_overlay(field, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_ir::{CheckMode, IntExpr, IntOp};
    use jai_types::{Integer, IntegerType};

    #[test]
    fn static_arithmetic_is_not_admitted_as_an_evaluated_literal() {
        let maximum = IntExpr::constant(Integer::checked(IntegerType::S8, 127).unwrap());
        let one = IntExpr::constant(Integer::checked(IntegerType::S8, 1).unwrap());
        let expression = ValueExpr::Int(
            IntExpr::new(
                IntegerType::S8,
                IntExprKind::Binary(IntOp::Add, Box::new(maximum), Box::new(one)),
            )
            .with_overflow_check(CheckMode::Enabled),
        );
        assert!(jai_ir::is_static_value(&expression));
        assert!(!concrete_literal(&expression, Span::default()).unwrap());
    }

    #[test]
    fn checked_literal_leaf_is_admitted_without_capture_authority() {
        let expression = ValueExpr::Int(IntExpr::constant(
            Integer::checked(IntegerType::S64, 42).unwrap(),
        ));
        assert!(concrete_literal(&expression, Span::default()).unwrap());
    }
}
