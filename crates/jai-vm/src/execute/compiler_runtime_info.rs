//! Admit an owned, target-certified runtime-info checkpoint through metered storage.
use super::*;
use crate::WorkspaceId;
use jai_types::RuntimeInfoSchema;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn compiler_runtime_info(
        &mut self,
        current_workspace: WorkspaceId,
        schema: RuntimeInfoSchema,
        arguments: &[Value],
    ) -> Result<Vec<Value>> {
        let [argument] = arguments else {
            return Err(Error::InvalidIr("runtime info requires one source workspace").into());
        };
        let workspace =
            crate::effects::workspace_argument(argument, current_workspace).map_err(|error| {
                match error {
                    EffectError::Pending(dependency) => Halt::Pending(dependency),
                    EffectError::Failed(error) => Halt::Failed(error),
                }
            })?;
        if workspace != current_workspace {
            return Err(Error::EffectRejected(
                "runtime-info snapshots from another workspace are not implemented".into(),
            )
            .into());
        }
        let snapshot = match self.provider.runtime_info() {
            RuntimeInfoAvailability::Ready {
                workspace: owner,
                snapshot,
            } if owner == workspace && snapshot.schema() == schema => snapshot,
            RuntimeInfoAvailability::Ready {
                ..
            } => {
                return Err(Error::InvalidIr(
                    "runtime-info checkpoint belongs to another workspace or source schema",
                )
                .into());
            }
            RuntimeInfoAvailability::Pending(dependency) => {
                return Err(Halt::Pending(dependency));
            }
            RuntimeInfoAvailability::Missing => {
                return Err(Error::EffectRejected(
                    "a certified source-visible runtime-info checkpoint is unavailable".into(),
                )
                .into());
            }
        };
        // This check is bounded by the fixed ABI. The retained descriptor graph
        // is admitted by static_address, which charges its actual traversal.
        snapshot
            .validate_owner(self.provider.types(), self.memory.target().policy)
            .map_err(|error| Error::IrValidation(error.to_string()))?;
        let pointer = self.static_address(snapshot.data(), snapshot.address(), 0)?;
        self.prepare_pointer_layouts(&pointer, true)?;
        self.charge_work(
            self.memory
                .load_work_cost(self.provider.types(), &pointer)?,
        )?;
        let value = self.memory.load(self.provider.types(), &pointer)?;
        self.validate_values(std::slice::from_ref(&value), &[schema.ty()])?;
        Ok(vec![value])
    }
}

#[cfg(test)]
mod tests;
