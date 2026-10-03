use super::*;
use jai_types::FloatValue;
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn float(&mut self, expression: &FloatExpr, depth: usize) -> Result<FloatValue> {
        self.step(depth)?;
        let value = match expression.kind() {
            FloatExprKind::Constant(value) => *value,
            FloatExprKind::Value(value) => self.value(value, depth + 1)?.float()?,
            FloatExprKind::Load(place) => self.load(*place, depth + 1)?.float()?,
            FloatExprKind::Call(call) => self.one(call, depth + 1)?.float()?,
            FloatExprKind::Negate(value) => self.float(value, depth + 1)?.negate(),
            FloatExprKind::Binary(op, lhs, rhs) => {
                let lhs = self.float(lhs, depth + 1)?;
                let rhs = self.float(rhs, depth + 1)?;
                crate::floats::binary(expression.ty(), *op, lhs, rhs)?
            }
            FloatExprKind::Cast(value) => self.float(value, depth + 1)?.convert(expression.ty()),
            FloatExprKind::FromInt(value) => {
                let number = self.integer(value, depth + 1)?;
                if number.provenance().is_some() {
                    return Err(Error::UnsupportedPointerOperation(
                        "address-derived integer cannot convert to a float",
                    )
                    .into());
                }
                FloatValue::from_integer(expression.ty(), number.integer())
            }
            FloatExprKind::Conditional(value) => {
                if self.boolean(&value.condition, depth + 1)? {
                    self.float(&value.then_value, depth + 1)?
                } else {
                    self.float(&value.else_value, depth + 1)?
                }
            }
        };
        if value.ty() != expression.ty() {
            return Err(Error::InvalidIr("floating-point expression result type differs").into());
        }
        Ok(value)
    }
}
