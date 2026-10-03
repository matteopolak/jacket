//! Exact source process receipts select a lazy, VM-owned virtual process ledger.
use super::*;
use crate::{
    process_abi::{ProcessAbiError, ProcessAbiOperation, ProcessAbiProcedure},
    process_source_machine::{ProcessBranchState, ProcessCallOutcome},
    virtual_process::{ProcessLimits, VirtualProcesses},
};

#[derive(Clone, Debug)]
pub(super) struct ProcessState {
    pub(super) world: VirtualProcesses,
    pub(super) branch: ProcessBranchState,
    cells: usize,
    work: u64,
}
impl ProcessState {
    /// Root creation has a fixed small shape; ordinary VM construction stays lazy.
    pub(super) fn new(limits: Limits) -> std::result::Result<Self, Error> {
        if limits.value_cells < 8 {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        let defaults = ProcessLimits::default();
        let mut world = VirtualProcesses::new(ProcessLimits {
            processes: defaults.processes.min(limits.value_cells),
            descriptors: defaults.descriptors.min(limits.value_cells),
            buffered_bytes: defaults.buffered_bytes.min(limits.value_cells),
            queued_messages: defaults.queued_messages.min(limits.value_cells),
            transferred_descriptors: defaults.transferred_descriptors.min(limits.value_cells),
            transfer_bytes: defaults.transfer_bytes.min(limits.value_cells),
        });
        let root = world.create_root().map_err(process_error)?;
        let mut state = Self {
            world,
            branch: ProcessBranchState::new(root),
            cells: 0,
            work: 0,
        };
        state.refresh(limits)?;
        Ok(state)
    }
    pub(super) fn cells(&self) -> usize {
        self.cells
    }
    pub(super) fn work_cost(&self) -> u64 {
        self.work
    }

    /// Exchange only branch-local errno/current-process state; IPC remains shared.
    pub(super) fn swap_branch(
        &mut self,
        other: &mut ProcessBranchState,
    ) -> std::result::Result<(), Error> {
        let old = self
            .branch
            .retained_metadata_cells()
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let new = other
            .retained_metadata_cells()
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let cells = self
            .cells
            .checked_sub(old)
            .and_then(|cells| cells.checked_add(new))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let work = self
            .work
            .checked_sub(u64::try_from(old).map_err(|_| Error::Limit(LimitKind::Fuel))?)
            .and_then(|work| work.checked_add(u64::try_from(new).ok()?))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        std::mem::swap(&mut self.branch, other);
        self.cells = cells;
        self.work = work;
        Ok(())
    }

    /// The dispatch precharges this walk before the adapter can grow the ledger.
    pub(super) fn refresh(&mut self, limits: Limits) -> std::result::Result<(), Error> {
        let work = self
            .world
            .work_cost()
            .map_err(process_error)?
            .checked_add(
                u64::try_from(self.branch.retained_metadata_cells())
                    .map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )
            .and_then(|work| work.checked_add(1))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        let cells = usize::try_from(work)
            .ok()
            .filter(|cells| *cells <= limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.work = work;
        self.cells = cells;
        Ok(())
    }
}
fn process_error(error: crate::virtual_process::ProcessError) -> Error {
    Error::EffectRejected(error.to_string())
}
fn binding_error(error: ProcessAbiError) -> Error {
    match error {
        ProcessAbiError::Type(error) => Error::Type(error),
        error => Error::EffectRejected(error.to_string()),
    }
}

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn invoke_process_available(
        &mut self,
        requested_id: ProcedureId,
        procedure: &ProcessAbiProcedure,
        arguments: &[Value],
    ) -> Result<ProcessCallOutcome> {
        if procedure.procedure() != requested_id {
            return Err(Error::InvalidIr("process capability names another procedure").into());
        }
        self.validate_provided_signature(requested_id, procedure.signature())?;
        procedure
            .validate(procedure.target(), self.provider.types())
            .map_err(binding_error)?;
        if self.memory.target() != crate::ByteTarget::from(procedure.target()) {
            return Err(
                Error::InvalidIr("process capability differs from VM source target").into(),
            );
        }
        let signature = self.signature(procedure.signature())?;
        // Typed validation and the frozen adapter's scalar carriers traverse
        // these operands; admit their borrowed shapes before either can clone.
        let mut argument_cells = 0;
        for argument in arguments {
            self.admit_value(&mut argument_cells, argument)?;
        }
        let operation = procedure.operation();
        if operation == ProcessAbiOperation::Fcntl {
            let fixed = arguments
                .get(..signature.parameters.len())
                .ok_or(Error::InvalidIr("process ABI argument count"))?;
            self.validate_values(fixed, &signature.parameters)?;
            procedure.validate_process_arguments(
                arguments,
                self.provider.types(),
                procedure.target(),
            )?;
        } else {
            self.validate_values(arguments, &signature.parameters)?;
        }
        if operation == ProcessAbiOperation::ExecVp {
            return Err(Error::UnsupportedForeignProcedure(requested_id).into());
        }
        for (index, argument) in arguments.iter().enumerate() {
            if let Value::Pointer(pointer) = argument {
                // Raw read/write buffers have a void view: prepare their backing
                // allocation, while typed outputs additionally need the pointee.
                if !pointer.is_null() {
                    let raw_buffer = index == 1
                        && matches!(
                            operation,
                            ProcessAbiOperation::Read | ProcessAbiOperation::Write
                        );
                    self.prepare_pointer_layouts(pointer, !raw_buffer)?;
                }
            }
        }
        if matches!(
            operation,
            ProcessAbiOperation::SocketPair
                | ProcessAbiOperation::SendMsg
                | ProcessAbiOperation::RecvMsg
                | ProcessAbiOperation::Shutdown
        ) && let Some(socket) = procedure.nominals().socket
        {
            self.prepare_layout(socket.message_header)?;
            self.prepare_layout(socket.control_header)?;
            self.prepare_layout(socket.io_vector)?;
        }
        if !matches!(
            operation,
            ProcessAbiOperation::GetPid
                | ProcessAbiOperation::GetParentPid
                | ProcessAbiOperation::Fork
                | ProcessAbiOperation::Exit
        ) {
            self.prepare_layout(procedure.nominals().error_code)?;
        }
        if self.processes.is_none() {
            self.charge_work(8)?;
            self.processes = Some(ProcessState::new(self.limits)?);
        }
        // Both initial adapter inspection and cached-shape refresh are admitted
        // before mutable ledger access. Payload growth adds its own scan allowance.
        let previous_work = self
            .processes
            .as_ref()
            .expect("initialized process ledger")
            .work_cost();
        let payload = if matches!(
            operation,
            ProcessAbiOperation::Read | ProcessAbiOperation::Write
        ) {
            let Value::Int(count) = &arguments[2] else {
                return Err(Error::UnsupportedPointerOperation(
                    "process byte count retains a virtual address",
                )
                .into());
            };
            count.bits()
        } else {
            0
        };
        let work = previous_work
            .checked_mul(3)
            .and_then(|work| {
                payload
                    .checked_mul(2)
                    .and_then(|payload| work.checked_add(payload))
            })
            .and_then(|work| work.checked_add(64))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
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
        let state = self.processes.as_mut().expect("initialized process ledger");
        let result = procedure.invoke_process(
            arguments,
            &mut self.memory,
            self.provider.types(),
            procedure.target(),
            &mut state.branch,
            &mut state.world,
            &mut charge,
        );
        let refreshed = state.refresh(self.limits);
        self.restore_call_context(previous_context);
        refreshed?;
        let outcome = result?;
        let mut retained = self
            .processes
            .as_ref()
            .expect("initialized process ledger")
            .cells()
            .checked_add(self.memory.value_cells())
            .and_then(|cells| cells.checked_add(self.expression_bindings.cells()))
            .filter(|cells| *cells <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        if let ProcessCallOutcome::Values(values) = &outcome {
            self.validate_values(values, &signature.results)?;
            for value in values {
                retained = retained
                    .checked_add(value.cells(self.limits.value_cells)?)
                    .filter(|cells| *cells <= self.limits.value_cells)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
            }
        }
        Ok(outcome)
    }
}

#[cfg(test)]
mod fcntl_tests;
#[cfg(test)]
mod tests;
