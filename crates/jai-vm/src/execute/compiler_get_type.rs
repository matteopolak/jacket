//! Private staging for inverse lookup of certified workspace-owned descriptors.
//!
//! Registration and exact source *Type_Info signature proof are coordinated
//! separately. Pointer spelling and plausible header bytes are not authority.
use super::*;
use jai_types::RuntimeTypeSchema;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn compiler_get_type(
        &mut self,
        schema: RuntimeTypeSchema,
        arguments: &[Value],
    ) -> Result<Vec<Value>> {
        let [Value::Pointer(pointer)] = arguments else {
            return Err(Error::InvalidIr("get_type requires its checked Type_Info pointer").into());
        };
        if pointer.pointee() != schema.header_type() {
            return Err(
                Error::InvalidIr("get_type pointer has another canonical header type").into(),
            );
        }
        self.charge_work(4)?;
        if pointer.is_null() {
            return Err(Error::NullPointer.into());
        }
        schema.validate(self.provider.types())?;
        self.prepare_pointer_layouts(pointer, true)?;
        // Lookup validates pointer metadata, exact registered descriptor
        // ownership and target policy. Precharge that traversal before cloning.
        let work = pointer
            .metadata_cells()
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        self.charge_work(work)?;
        let result = Value::Type {
            descriptor: Some(pointer.clone()),
        };
        if self.runtime_type_identity(&result)?.schema() != schema {
            return Err(
                Error::InvalidIr("get_type descriptor belongs to another source schema").into(),
            );
        }
        Ok(vec![result])
    }
}

#[cfg(test)]
mod tests;
