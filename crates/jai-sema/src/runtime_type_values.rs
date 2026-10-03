//! First-class Type values use the existing canonical immutable reflection objects.
use super::*;
use jai_types::{RuntimeTypeSchema, TypeKind};

impl Resolver<'_> {
    pub(crate) fn ensure_runtime_type_storage(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut pending = vec![ty];
        let mut seen = std::collections::HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match self
                .types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .clone()
            {
                TypeKind::Type => {
                    self.schema_header_type(span)?;
                }
                TypeKind::Record(_) | TypeKind::Any(_) => {
                    pending.extend(
                        self.types
                            .record_storage_definition(ty)
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?
                            .fields
                            .iter()
                            .copied(),
                    );
                }
                TypeKind::Distinct(_) => pending.push(
                    self.types
                        .distinct_definition(ty)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?
                        .representation,
                ),
                TypeKind::FixedArray {
                    element, ..
                } => pending.push(element),
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn runtime_type_expression(
        &mut self,
        value: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Type(represented) = value else {
            return Ok(value);
        };
        let Expr::Pointer {
            value:
                ValueExpr::StaticAddress {
                    data,
                    address,
                    ..
                },
            ..
        } = self.type_info_header_expression(represented, span)?
        else {
            unreachable!("reflection returns immutable descriptor storage")
        };
        let constant = RuntimeTypeConstant::new(data, address.object(), self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if constant.address() != &address || constant.identity().ty() != represented {
            return Err(Diagnostic::new(
                span,
                "runtime Type value has a different descriptor identity",
            ));
        }
        Ok(Expr::Typed {
            ty: constant.ty(),
            value: ValueExpr::RuntimeType(constant),
        })
    }

    pub(crate) fn is_runtime_type_expression(&self, value: &Expr) -> bool {
        matches!(value, Expr::Typed { ty, .. } if *ty == self.types.meta_type())
    }

    pub(crate) fn runtime_type_binary_operands(
        &mut self,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<(Expr, Expr), Diagnostic> {
        if self.is_runtime_type_expression(&left) || self.is_runtime_type_expression(&right) {
            Ok((
                self.runtime_type_expression(left, span)?,
                self.runtime_type_expression(right, span)?,
            ))
        } else {
            Ok((left, right))
        }
    }

    pub(crate) fn runtime_type_binary(
        &self,
        operator: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Operator::Equality(equality) = Operator::from(operator) else {
            return Err(Diagnostic::new(
                span,
                "Type values only support nominal equality comparisons",
            ));
        };
        let schema = RuntimeTypeSchema::from_view(self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let left = self.coerce_value(left, schema.ty(), span)?;
        let right = self.coerce_value(right, schema.ty(), span)?;
        Ok(Expr::Bool(BoolExpr::ComparePointers(
            equality,
            Box::new(ValueExpr::TypeDescriptor {
                value: Box::new(left),
                ty: schema.descriptor_type(),
            }),
            Box::new(ValueExpr::TypeDescriptor {
                value: Box::new(right),
                ty: schema.descriptor_type(),
            }),
        )))
    }
}
