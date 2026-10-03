//! A contextual C string is immutable static bytes, including one appended NUL.
use super::*;
use jai_ir::{
    StaticAddress, StaticDataBuilder, StaticDataLimits, StaticProjection, StaticValue,
    StaticValueKind,
};
use jai_types::TypeKind;
use std::sync::Arc;

impl Resolver<'_> {
    pub(crate) fn c_string_literal(
        &mut self,
        bytes: &[u8],
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let byte = self.types.scalar(ScalarType::Int(IntegerType::U8));
        if !matches!(self.types.kind(ty), Ok(TypeKind::Pointer(element)) if *element == byte) {
            return Err(Diagnostic::new(
                span,
                "C string literal requires an unsigned-byte pointer context",
            ));
        }
        let count = bytes
            .len()
            .checked_add(1)
            .and_then(|count| u64::try_from(count).ok())
            .ok_or_else(|| Diagnostic::new(span, "C string literal is too large"))?;
        let limits = StaticDataLimits::default();
        if count >= limits.value_nodes as u64 {
            return Err(Diagnostic::new(
                span,
                "C string literal exceeds static storage limits",
            ));
        }
        let array = self
            .types
            .fixed_array(byte, count)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let mut builder = StaticDataBuilder::new();
        let object = builder
            .reserve(array, self.types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let elements = bytes
            .iter()
            .copied()
            .chain(std::iter::once(0))
            .map(|value| {
                StaticValue::constant(ConstantValue {
                    ty: byte,
                    kind: ConstantKind::Int(IntegerValue::wrapping(
                        IntegerType::U8,
                        i128::from(value),
                    )),
                })
            })
            .collect();
        builder
            .define(
                object,
                StaticValue {
                    ty: array,
                    kind: StaticValueKind::Array(elements),
                },
            )
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let data = builder
            .finish(self.types, limits)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(Expr::Pointer {
            ty,
            value: ValueExpr::StaticAddress {
                data: Arc::new(data),
                address: StaticAddress::new(object).project(StaticProjection::Index(0)),
                ty,
            },
        })
    }
}
