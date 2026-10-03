//! Runtime Type values name certified immutable descriptors, never integer IDs.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub fn runtime_type_identity(
        &self,
        value: &Value,
    ) -> std::result::Result<jai_ir::RuntimeTypeIdentity, Error> {
        self.memory
            .runtime_type_identity(self.provider.types(), value)
    }
    /// Recover a portable certified descriptor constant while publication can roll back.
    pub fn runtime_type_constant_value(
        &self,
        value: &Value,
    ) -> std::result::Result<RuntimeTypeConstant, Error> {
        let identity = self.runtime_type_identity(value)?;
        let data = self
            .static_data
            .get(&identity.object().arena_identity())
            .ok_or(Error::InvalidIr(
                "runtime Type has no immutable static publication",
            ))?;
        RuntimeTypeConstant::from_identity(
            std::sync::Arc::clone(data),
            identity,
            self.provider.types(),
        )
        .map_err(|error| match error {
            StaticDataError::Type(error) => Error::Type(error),
            error => Error::IrValidation(error.to_string()),
        })
    }
    pub(super) fn runtime_type_constant(
        &mut self,
        value: &RuntimeTypeConstant,
        depth: usize,
    ) -> Result<Value> {
        self.step(depth)?;
        value.identity().validate(self.provider.types())?;
        let descriptor = self.static_address(value.data(), value.address(), depth + 1)?;
        let result = Value::Type {
            descriptor: Some(descriptor),
        };
        if self.runtime_type_identity(&result)? != value.identity() {
            return Err(Error::InvalidIr(
                "runtime Type constant identity differs from static publication",
            )
            .into());
        }
        Ok(result)
    }
}
