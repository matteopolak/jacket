//! Value materialization belongs to the frame evaluating the conversion.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn address_of_value(
        &mut self,
        expression: &ValueExpr,
        pointer_type: TypeId,
        depth: usize,
    ) -> Result<Value> {
        let types = self.provider.types();
        let TypeKind::Pointer(pointee) = *types.kind(pointer_type)? else {
            return Err(Error::InvalidIr("materialized address requires a pointer type").into());
        };
        if expression.type_id(types) != pointee {
            return Err(Error::TypeMismatch {
                expected: pointee,
            }
            .into());
        }
        let value = self.value(expression, depth + 1)?;
        let pointer = self.temporary(pointee, value, depth + 1)?;
        Ok(Value::Pointer(pointer))
    }
}
