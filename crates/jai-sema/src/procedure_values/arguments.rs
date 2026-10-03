//! Variadic argument construction retains evaluation order and target type.
use super::*;
use jai_types::{FloatType, IntegerType, TypeKind};

impl Resolver<'_> {
    pub(crate) fn jai_variadic_pack(
        &mut self,
        arguments: &[&syntax::CallArgument],
        element: TypeId,
        slice: TypeId,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        let mut elements = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let value = self.expr_expected(&argument.value, element)?;
            elements.push(self.coerce_value(value, element, argument.value.span)?);
        }
        let count = u64::try_from(elements.len())
            .map_err(|_| Diagnostic::new(span, "variadic pack exceeds the supported array size"))?;
        let array = self
            .types
            .fixed_array(element, count)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(ValueExpr::ArrayView {
            array: Box::new(ValueExpr::Array {
                ty: array,
                elements,
            }),
            ty: slice,
        })
    }

    pub(crate) fn c_variadic_argument(
        &mut self,
        argument: &syntax::CallArgument,
    ) -> Result<ValueExpr, Diagnostic> {
        let span = argument.value.span;
        if argument.name.is_some() {
            return Err(Diagnostic::new(
                span,
                "C variadic arguments cannot be named",
            ));
        }
        let value = self.expr(&argument.value)?;
        match value {
            value if value.has_float() => {
                value.float_as(FloatType::F64, span).map(ValueExpr::Float)
            }
            Expr::Bool(value) => Ok(ValueExpr::Int(IntExpr::new(
                IntegerType::S32,
                IntExprKind::FromBool(Box::new(value)),
            ))),
            Expr::Enum {
                representation,
                value,
                ..
            } => {
                let value = IntExpr::new(representation, IntExprKind::EnumValue(Box::new(value)));
                Ok(ValueExpr::Int(if representation.bits() < 32 {
                    IntExpr::new(
                        IntegerType::S32,
                        IntExprKind::Cast(CastMode::Unchecked, Box::new(value)),
                    )
                } else {
                    value
                }))
            }
            Expr::Int(value) if value.ty().bits() < 32 => Ok(ValueExpr::Int(IntExpr::new(
                IntegerType::S32,
                IntExprKind::Cast(CastMode::Unchecked, Box::new(value)),
            ))),
            Expr::Int(value) => Ok(ValueExpr::Int(value)),
            Expr::Literal(_) | Expr::WeakConditional(_) => {
                value.int_as(IntegerType::S64, span).map(ValueExpr::Int)
            }
            Expr::Pointer {
                value, ..
            } => Ok(value),
            Expr::Typed {
                ty,
                value,
            } if matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_))) => Ok(value),
            _ => Err(Diagnostic::new(
                span,
                "C variadic argument requires an ABI scalar or pointer",
            )),
        }
    }
}
