//! Materialize a value where it is evaluated, keeping caller-frame storage.
use super::*;
use inkwell::values::BasicValue;

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn address_of_value(
        &mut self,
        expression: &ValueExpr,
        pointer_type: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let TypeKind::Pointer(pointee) = *self.types.kind(pointer_type)? else {
            return Err(Error::Invariant);
        };
        if expression.type_id(self.types) != pointee {
            return Err(Error::Invariant);
        }
        // Alloca and initialization remain in this branch, after its operand.
        // Each evaluation gets storage lasting until the evaluating frame exits.
        let snapshot = self.value(expression)?;
        let layout = self.lowerer.semantic_layout(pointee)?;
        let storage = self
            .builder
            .build_alloca(snapshot.get_type(), "any.payload")?;
        storage
            .as_instruction_value()
            .ok_or(Error::Invariant)?
            .set_alignment(layout.alignment)
            .map_err(|_| Error::Invariant)?;
        memory::store(&self.builder, storage, snapshot, layout.alignment)?;
        Ok(storage.into())
    }
}
