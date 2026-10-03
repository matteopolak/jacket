//! Immutable VM execution phase; compile-time behavior remains the default.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExecutionPhase {
    #[default]
    CompileTime,
    Runtime,
}
impl ExecutionPhase {
    pub fn is_compile_time(self) -> bool {
        self == Self::CompileTime
    }
}
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub fn execution_phase(&self) -> ExecutionPhase {
        self.execution_phase
    }
    pub(super) fn require_procedure_phase(
        &self,
        id: ProcedureId,
    ) -> std::result::Result<(), Error> {
        if self.execution_phase == ExecutionPhase::Runtime
            && self.provider.procedure_execution(id)
                == jai_types::ProcedureExecution::CompileTimeOnly
        {
            return Err(Error::CompileTimeOnlyProcedure(id));
        }
        Ok(())
    }
}
