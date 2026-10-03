//! Pure enum_flags masks preserve source nominal identity and representation.
use super::*;
use jai_syntax::{BinaryOp, EnumKind, Expression, ExpressionKind, UnaryOp};
use jai_types::Integer;
impl Builder<'_> {
    pub(super) fn enum_is_flags(&self, declaration: DeclarationId) -> bool {
        matches!(&self.graph.declarations[declaration.index()].syntax.kind,
            FileDeclarationKind::Enum(enumeration) if enumeration.kind == EnumKind::Flags)
    }
    pub(super) fn enum_mask(
        &self,
        file: FileInstanceId,
        expression: &Expression,
        expected: Option<DeclarationId>,
        active: &mut Vec<DeclarationId>,
        depth: usize,
    ) -> Result<Option<EnumParameter>, GraphError> {
        let location = SourceSpan {
            source: self.graph.files[file.index()].source,
            span: expression.span,
        };
        if depth >= 128 {
            return Err(self.located(location, "enum mask expression exceeds the recursion limit"));
        }
        let value = match &expression.kind {
            ExpressionKind::Unary(UnaryOp::Complement, operand) => {
                let Some(mut value) = self.enum_mask(file, operand, expected, active, depth + 1)?
                else {
                    return Ok(None);
                };
                if !self.enum_is_flags(value.declaration) {
                    return Err(self.located(
                        location,
                        "bitwise enum operation requires an enum_flags type",
                    ));
                }
                value.value = Integer::wrapping(value.value.ty(), i128::from(!value.value.bits()));
                value
            }
            ExpressionKind::Binary(
                operator @ (BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor),
                left,
                right,
            ) => {
                let left_value = self.enum_mask(file, left, expected, active, depth + 1)?;
                let right_value = self.enum_mask(file, right, expected, active, depth + 1)?;
                let Some(declaration) =
                    expected.or_else(|| left_value.or(right_value).map(|value| value.declaration))
                else {
                    return Ok(None);
                };
                let left = left_value.or(self.enum_mask(
                    file,
                    left,
                    Some(declaration),
                    active,
                    depth + 1,
                )?);
                let right = right_value.or(self.enum_mask(
                    file,
                    right,
                    Some(declaration),
                    active,
                    depth + 1,
                )?);
                let (Some(left), Some(right)) = (left, right) else {
                    return Err(self.located(
                        location,
                        "enum operation requires values of the same nominal type",
                    ));
                };
                if left.declaration != right.declaration {
                    return Err(self.located(
                        location,
                        "enum operation requires values of the same nominal type",
                    ));
                }
                if !self.enum_is_flags(declaration) {
                    return Err(self.located(
                        location,
                        "bitwise enum operation requires an enum_flags type",
                    ));
                }
                let bits = match operator {
                    BinaryOp::BitAnd => left.value.bits() & right.value.bits(),
                    BinaryOp::BitOr => left.value.bits() | right.value.bits(),
                    BinaryOp::BitXor => left.value.bits() ^ right.value.bits(),
                    _ => unreachable!(),
                };
                EnumParameter {
                    declaration,
                    value: Integer::wrapping(left.value.ty(), i128::from(bits)),
                }
            }
            ExpressionKind::InferredMember(name) => {
                let Some(declaration) = expected else {
                    return Ok(None);
                };
                self.enum_member(declaration, *name, location)?
            }
            _ => {
                let Some(ParameterValue::Enumeration(value)) =
                    self.nominal_argument(file, expression, active)?
                else {
                    return Ok(None);
                };
                value
            }
        };
        if expected.is_some_and(|expected| expected != value.declaration) {
            return Err(self.located(
                location,
                "enum operation requires values of the same nominal type",
            ));
        }
        Ok(Some(value))
    }
}
