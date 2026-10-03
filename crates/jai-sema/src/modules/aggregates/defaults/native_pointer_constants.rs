//! Numeric native-address defaults retain their cast until target normalization.
use super::*;

impl Defaults<'_, '_> {
    pub(super) fn native_pointer_constant(
        &mut self,
        file: FileInstanceId,
        value: &Expression,
        target: TypeId,
        mode: jai_types::CastMode,
        span: Span,
    ) -> Result<TypedConstant, LocatedDiagnostic> {
        if matches!(mode, jai_types::CastMode::Force(_)) {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    span,
                    "storage force casts require a storage constant recipe, not a numeric native address",
                ),
            ));
        }
        let (pointer, zero) = match self.scalar(file, value)? {
            ConstantValue::Literal(value) => (
                jai_ir::NativePointerConstant::new_weak(target, value, mode, self.types),
                value == 0,
            ),
            ConstantValue::Int(value) => (
                jai_ir::NativePointerConstant::new(target, value, mode, self.types),
                value.bits() == 0,
            ),
            _ => {
                return Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "numeric native pointer cast requires an integer operand",
                    ),
                ));
            }
        };
        let pointer = pointer
            .map_err(|error| located(self.graph, file, Diagnostic::new(span, error.to_string())))?;
        if let Some(layout) = self.nominals.annotation_target() {
            let bits = layout
                .pointer()
                .size
                .checked_mul(8)
                .and_then(|bits| u32::try_from(bits).ok())
                .ok_or_else(|| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            span,
                            "selected target pointer width exceeds normalization limits",
                        ),
                    )
                })?;
            pointer.address(bits).map_err(|error| {
                located(self.graph, file, Diagnostic::new(span, error.to_string()))
            })?;
        }
        // Only a source zero is null independently of the future execution target.
        // A nonzero value truncated to zero for one width must retain its payload.
        let kind = if zero {
            ConstantKind::Zero
        } else {
            ConstantKind::NativePointer(pointer)
        };
        Ok(TypedConstant {
            ty: target,
            kind,
        })
    }
}
