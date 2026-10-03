//! Explicit address casts retain integer widths and typed pointer identities.
use super::*;
use jai_types::TypeKind;

impl Resolver<'_> {
    pub(crate) fn pointer_from_operand(
        &mut self,
        value: Expr,
        target: TypeId,
        mode: CastMode,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if !matches!(self.types.kind(target), Ok(TypeKind::Pointer(_))) {
            return Err(Diagnostic::new(
                span,
                "pointer cast target must be a pointer type",
            ));
        }
        let value = self.runtime_type_expression(value, span)?;
        let value = if self.is_runtime_type_expression(&value) {
            let schema = jai_types::RuntimeTypeSchema::from_view(self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            let value = self.coerce_value(value, schema.ty(), span)?;
            Expr::Pointer {
                ty: schema.descriptor_type(),
                value: ValueExpr::TypeDescriptor {
                    value: Box::new(value),
                    ty: schema.descriptor_type(),
                },
            }
        } else {
            value
        };
        let value = match value {
            Expr::Null => ValueExpr::Zero(target),
            Expr::Pointer { value, .. } => ValueExpr::PointerCast {
                value: Box::new(value),
                ty: target,
                mode,
            },
            Expr::Int(value) => ValueExpr::PointerFromInteger {
                value,
                ty: target,
                mode,
            },
            Expr::Literal(number) => {
                let pointer =
                    jai_ir::NativePointerConstant::new_weak(target, number, mode, self.types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                if let Some(layout) = self.target_layout {
                    let bits = layout
                        .pointer()
                        .size
                        .checked_mul(8)
                        .and_then(|bits| u32::try_from(bits).ok())
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "selected target pointer width exceeds normalization limits",
                            )
                        })?;
                    pointer
                        .address(bits)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                }
                jai_ir::ConstantValue {
                    ty: target,
                    kind: jai_ir::ConstantKind::NativePointer(pointer),
                }
                .into_expression()
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "pointer casts require a pointer, integer, or null",
                ));
            }
        };
        Ok(Expr::Pointer { ty: target, value })
    }
}
