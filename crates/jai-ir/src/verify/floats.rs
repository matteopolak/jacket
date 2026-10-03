use super::*;

impl Context<'_> {
    pub(super) fn float(&self, expression: &FloatExpr) -> Result<(), IrError> {
        let _depth = self.enter()?;
        let expected = self.types.float(expression.ty());
        match expression.kind() {
            FloatExprKind::Constant(value) => {
                same_type(expected, self.types.float(value.ty()))?;
            }
            FloatExprKind::Value(value) => same_type(expected, self.value(value)?)?,
            FloatExprKind::Load(place) => same_type(expected, self.place(*place)?)?,
            FloatExprKind::Call(call) => self.single_call(call, expected)?,
            FloatExprKind::Negate(value) => {
                self.float(value)?;
                same_type(expected, value.type_id(self.types))?;
            }
            FloatExprKind::Binary(_, left, right) => {
                self.float(left)?;
                self.float(right)?;
                same_type(expected, left.type_id(self.types))?;
                same_type(expected, right.type_id(self.types))?;
            }
            FloatExprKind::Cast(value) => self.float(value)?,
            FloatExprKind::FromInt(value) => self.integer(value)?,
            FloatExprKind::Conditional(conditional) => {
                self.boolean(&conditional.condition)?;
                self.float(&conditional.then_value)?;
                self.float(&conditional.else_value)?;
                same_type(expected, conditional.then_value.type_id(self.types))?;
                same_type(expected, conditional.else_value.type_id(self.types))?;
            }
        }
        Ok(())
    }
}
