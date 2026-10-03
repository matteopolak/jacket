//! Resolve declaration allocation policy without altering canonical value types.
use super::{Diagnostic, Resolver, ScalarConstant, Span, syntax};

pub(crate) fn integer_alignment(value: ScalarConstant) -> Option<u32> {
    let integer = match value {
        ScalarConstant::Literal(value) => Some(value),
        ScalarConstant::Int(value) => Some(value.value()),
        ScalarConstant::Bool(_) | ScalarConstant::Float(_) | ScalarConstant::WeakFloat(_) => None,
    };
    integer
        .and_then(|integer| u32::try_from(integer).ok())
        .filter(|alignment| alignment.is_power_of_two())
}

pub(crate) fn constant(value: ScalarConstant, span: Span) -> Result<u32, Diagnostic> {
    integer_alignment(value).ok_or_else(|| Diagnostic::new(
            span,
            "storage alignment requires a nonzero power-of-two integer constant representable as u32",
        ))
}

pub(crate) fn target_bound(
    alignment: u32,
    target: Option<jai_types::LayoutPolicy>,
    span: Span,
) -> Result<u32, Diagnostic> {
    if let Some(target) = target {
        let bits = target.pointer().size.saturating_mul(8);
        if bits == 0 || (bits <= 64 && u64::from(alignment) > (1u64 << (bits - 1)) - 1) {
            return Err(Diagnostic::new(
                span,
                "storage alignment exceeds the target signed address space",
            ));
        }
    }
    Ok(alignment)
}

impl Resolver<'_> {
    fn alignment_value(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<ScalarConstant, Diagnostic> {
        match self.local_scalar_expression(expression) {
            Ok(value) => Ok(value),
            Err(scalar_error) => {
                let typed = (|| {
                    let value = self.expr(expression)?;
                    let ty = self.expression_type(&value, expression.span)?;
                    let value = self.coerce_value(value, ty, expression.span)?;
                    self.evaluate_pure_constant(value, expression.span)
                })();
                match typed {
                    Ok(value) => match value.kind {
                        super::ConstantKind::Int(value) | super::ConstantKind::Enum(value) => {
                            Ok(ScalarConstant::Int(value))
                        }
                        super::ConstantKind::Bool(value) => Ok(ScalarConstant::Bool(value)),
                        super::ConstantKind::Float(value) => Ok(ScalarConstant::Float(value)),
                        _ => Err(Diagnostic::new(
                            expression.span,
                            "storage alignment requires an integer constant",
                        )),
                    },
                    Err(error) if super::reflection::is_semantic_constant(expression) => Err(error),
                    Err(_) => Err(scalar_error),
                }
            }
        }
    }
    pub(crate) fn declaration_alignment(
        &mut self,
        declaration: &syntax::Declaration,
    ) -> Result<Option<u32>, Diagnostic> {
        self.declaration_attributes_alignment(declaration.attributes())
    }
    pub(crate) fn declaration_attributes_alignment(
        &mut self,
        attributes: &[syntax::DeclarationAttribute],
    ) -> Result<Option<u32>, Diagnostic> {
        let mut alignment = None;
        for attribute in attributes {
            match attribute {
                syntax::DeclarationAttribute::Alignment(expression) => {
                    let value = self.alignment_value(expression)?;
                    alignment = Some(target_bound(
                        constant(value, expression.span)?,
                        self.target_layout,
                        expression.span,
                    )?);
                }
            }
        }
        Ok(alignment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{LayoutPolicy, ScalarLayout};

    #[test]
    fn storage_alignment_rejects_zero_negative_fractional_and_oversized_constants() {
        for invalid in [0, -1, 3, 1i128 << 32] {
            assert!(constant(ScalarConstant::Literal(invalid), Span::default()).is_err());
        }
        assert!(constant(ScalarConstant::Bool(true), Span::default()).is_err());
        assert_eq!(
            constant(ScalarConstant::Literal(64), Span::default()).unwrap(),
            64
        );
    }

    #[test]
    fn storage_alignment_obeys_the_selected_target() {
        let narrow = LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap();
        assert!(target_bound(1 << 31, Some(narrow), Span::default()).is_err());
        assert!(target_bound(1 << 31, Some(LayoutPolicy::lp64()), Span::default()).is_ok());
    }
}
