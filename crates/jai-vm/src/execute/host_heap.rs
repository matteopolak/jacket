//! Actual source allocator capabilities operate only on the VM-owned heap ledger.
use super::*;

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn heap_call(
        &mut self,
        id: ProcedureId,
        procedure: crate::heap_abi::HeapAbiProcedure,
        arguments: Vec<Value>,
    ) -> Result<Vec<Value>> {
        if procedure.procedure() != id {
            return Err(Error::InvalidIr("heap capability names another procedure").into());
        }
        self.validate_provided_signature(id, procedure.signature)?;
        let signature = self.signature(procedure.signature)?;
        self.validate_values(&arguments, &signature.parameters)?;
        let work = procedure.work_cost(
            &arguments,
            &self.memory,
            self.provider.types(),
            &self.host_heap,
        )?;
        self.charge_work(usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
        let previous_context = self.enter_call_context(signature.context)?;
        let result = procedure.invoke(
            &arguments,
            &mut self.memory,
            self.provider.types(),
            &mut self.host_heap,
        );
        self.restore_call_context(previous_context);
        let values = result?;
        self.validate_values(&values, &signature.results)?;
        Ok(values)
    }
}
