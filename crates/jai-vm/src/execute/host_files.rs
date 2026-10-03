//! Explicit stdio capability dispatch shares the compiler effect transaction.
use super::*;
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn file_call(
        &mut self,
        id: ProcedureId,
        procedure: crate::file_abi::FileAbiProcedure,
        arguments: Vec<Value>,
    ) -> Result<Vec<Value>> {
        if procedure.procedure() != id {
            return Err(Error::InvalidIr("FILE capability names another procedure").into());
        }
        self.validate_provided_signature(id, procedure.signature)?;
        let signature = self.signature(procedure.signature)?;
        self.validate_values(&arguments, &signature.parameters)?;
        let work = procedure.work_cost(
            &arguments,
            &self.memory,
            self.provider.types(),
            &self.host_files,
        )?;
        self.charge_work(usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
        let previous_context = self.enter_call_context(signature.context)?;
        let statistics = &mut self.statistics;
        let fuel = self.limits.fuel;
        let mut charge = |work: u64| {
            statistics.steps = statistics
                .steps
                .checked_add(work)
                .filter(|steps| *steps <= fuel)
                .ok_or(Error::Limit(LimitKind::Fuel))?;
            Ok(())
        };
        let result = procedure.invoke(
            &arguments,
            &mut self.memory,
            self.provider.types(),
            &mut self.host_files,
            &mut self.effects,
            &mut charge,
        );
        self.restore_call_context(previous_context);
        let values = result?;
        self.validate_values(&values, &signature.results)?;
        Ok(values)
    }
}
