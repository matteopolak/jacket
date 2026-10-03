//! Sealed foreign-process stops at an already evaluated result continuation.
use super::*;
use crate::process_abi::{ProcessAbiError, ProcessAbiOperation, ProcessAbiProcedure};
use crate::process_source_machine::{
    ProcessBranchState, ProcessCallOutcome, ProcessExecControl, ProcessExecStage,
    ProcessExitControl, ProcessForkControl,
};
use crate::virtual_process::{ForkPair, ProcessId, VirtualProcesses};

// The frozen target validation excludes heap-bearing Other target strings.
const STOP_PAYLOAD_CELLS: usize = 2;
const STOP_CELLS: usize = STOP_PAYLOAD_CELLS + 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::execute::resumable) enum DriveStatus {
    Complete,
    ProcessControl,
    Retired,
}
#[derive(Clone)]
pub(in crate::execute::resumable) enum ProcessControl {
    Fork(ProcessForkControl),
    Exit(ProcessExitControl),
    Exec(ProcessExecControl),
}
impl ProcessControl {
    fn proof(&self) -> &ProcessAbiProcedure {
        match self {
            Self::Fork(control) => control.procedure(),
            Self::Exit(control) => control.procedure(),
            Self::Exec(control) => control.procedure(),
        }
    }
    fn current(&self) -> ProcessId {
        match self {
            Self::Fork(control) => control.parent(),
            Self::Exit(control) => control.process(),
            Self::Exec(control) => control.process(),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct CallResultContinuation {
    signature: TypeId,
    expected: Option<TypeId>,
}
#[derive(Clone)]
pub(super) struct ProcessStop {
    control: ProcessControl,
    result: CallResultContinuation,
    serial: u64,
}
pub(in crate::execute::resumable) struct PreparedProcessResult {
    operand: Operand,
    cells: usize,
    source_retained: usize,
    next_retained: usize,
    serial: u64,
    result: CallResultContinuation,
    current: ProcessId,
    pair: ForkPair,
}

fn process_failure(error: crate::virtual_process::ProcessError) -> Halt {
    Error::EffectRejected(error.to_string()).into()
}
fn binding_failure(error: ProcessAbiError) -> Halt {
    match error {
        ProcessAbiError::Type(error) => Error::Type(error).into(),
        error => Error::EffectRejected(error.to_string()).into(),
    }
}
impl Machine {
    pub(in crate::execute::resumable) fn process_control(&self) -> Option<&ProcessControl> {
        match &self.tasks.last()?.action {
            Action::ProcessStop(stop) => Some(&stop.control),
            _ => None,
        }
    }
    fn process_stop(&self) -> Result<&ProcessStop> {
        match &self.tasks.last().map(|task| &task.action) {
            Some(Action::ProcessStop(stop)) => Ok(stop),
            _ => Err(Error::InvalidIr("machine has no stopped process call").into()),
        }
    }
    fn process_combined_cells<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &self,
        vm: &Vm<'_, P, E>,
        machine_cells: usize,
    ) -> Result<usize> {
        machine_cells
            .checked_add(self.plan_cells)
            .and_then(|n| n.checked_add(vm.memory.value_cells()))
            .and_then(|n| n.checked_add(vm.expression_bindings.cells()))
            .and_then(|n| {
                n.checked_add(
                    vm.processes
                        .as_ref()
                        .map_or(0, process::ProcessState::cells),
                )
            })
            .filter(|n| *n <= vm.limits.value_cells)
            .ok_or_else(|| Error::Limit(LimitKind::ValueCells).into())
    }
    fn reserve_process_stop<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        arguments: &[Value],
    ) -> Result<()> {
        let transient = arguments.iter().try_fold(0usize, |cells, value| {
            cells
                .checked_add(value.cells(vm.limits.value_cells)?)
                .ok_or(Error::Limit(LimitKind::ValueCells))
        })?;
        let prospective = self
            .retained
            .checked_add(transient)
            .and_then(|n| n.checked_add(STOP_CELLS))
            .and_then(|n| n.checked_add(if vm.processes.is_none() { 8 } else { 0 }))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.process_combined_cells(vm, prospective)?;
        self.next_process_stop
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        // No adapter can mutate the world before the stop's outer slot is reserved.
        if self.tasks.len() == self.tasks.capacity() {
            vm.charge_work(self.tasks.len().saturating_add(1))?;
            self.tasks
                .try_reserve(1)
                .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        }
        vm.charge_work(STOP_CELLS)?;
        Ok(())
    }
    fn install_process_stop<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &Vm<'_, P, E>,
        control: ProcessControl,
        expected: Option<TypeId>,
        depth: usize,
    ) -> Result<()> {
        let serial = self.next_process_stop;
        let next = serial.checked_add(1).ok_or(Error::Limit(LimitKind::Fuel))?;
        let signature = control.proof().signature();
        self.push(
            vm,
            None,
            depth,
            Action::ProcessStop(Box::new(ProcessStop {
                control,
                result: CallResultContinuation {
                    signature,
                    expected,
                },
                serial,
            })),
            STOP_PAYLOAD_CELLS,
        )?;
        self.next_process_stop = next;
        Ok(())
    }
    pub(super) fn invoke_process_leaf<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        id: ProcedureId,
        proof: &ProcessAbiProcedure,
        arguments: &[Value],
        expected: Option<TypeId>,
        depth: usize,
    ) -> Result<()> {
        proof
            .validate(proof.target(), vm.provider.types())
            .map_err(binding_failure)?;
        self.reserve_process_stop(vm, arguments)?;
        // Pure process dispatch deliberately does not start, retry or clear a journal leaf.
        match vm.invoke_process_available(id, proof, arguments)? {
            ProcessCallOutcome::Values(values) => {
                vm.validate_values(&values, &vm.signature(proof.signature())?.results)?;
                self.receive(vm, values, expected)
            }
            ProcessCallOutcome::Pending(event) => Err(Halt::Pending(Dependency::Process(event))),
            ProcessCallOutcome::ForkControl(control) => {
                self.install_process_stop(vm, ProcessControl::Fork(control), expected, depth)
            }
            ProcessCallOutcome::ExitControl(control) => {
                self.install_process_stop(vm, ProcessControl::Exit(control), expected, depth)
            }
            ProcessCallOutcome::ExecControl(control) => {
                self.install_process_stop(vm, ProcessControl::Exec(control), expected, depth)
            }
        }
    }
    pub(in crate::execute::resumable) fn prepare_fork_result<
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    >(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        world: &VirtualProcesses,
        branch: &ProcessBranchState,
        pair: ForkPair,
    ) -> Result<PreparedProcessResult> {
        let stop = self.process_stop()?;
        let ProcessControl::Fork(control) = &stop.control else {
            return Err(Error::InvalidIr("process stop is not a fork").into());
        };
        let proof = control.procedure();
        proof
            .validate(proof.target(), vm.provider.types())
            .map_err(binding_failure)?;
        vm.validate_provided_signature(proof.procedure(), proof.signature())?;
        if vm.memory.target() != crate::ByteTarget::from(proof.target()) {
            return Err(Error::InvalidIr("fork result target differs from stopped call").into());
        }
        if proof.operation() != ProcessAbiOperation::Fork
            || pair.parent != control.parent()
            || world.parent(pair.child).map_err(process_failure)? != Some(pair.parent)
            || ![pair.parent, pair.child].contains(&branch.current())
        {
            return Err(
                Error::InvalidIr("fork result does not name the stopped process pair").into(),
            );
        }
        world
            .validate_running(branch.current())
            .map_err(process_failure)?;
        vm.charge_work(STOP_CELLS + 3)?;
        let bits = if branch.current() == pair.parent {
            i128::from(pair.child.abi_value().map_err(process_failure)?)
        } else {
            0
        };
        let value = Value::Int(
            jai_types::Integer::checked(jai_types::IntegerType::S32, bits)
                .ok_or(Error::CheckedCast)?,
        );
        vm.validate_values(
            std::slice::from_ref(&value),
            &vm.signature(stop.result.signature)?.results,
        )?;
        let result = stop.result;
        let serial = stop.serial;
        let cells = if result.expected.is_some() { 1 } else { 2 };
        let next_retained = self
            .retained
            .checked_sub(STOP_CELLS)
            .and_then(|n| n.checked_add(cells))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.process_combined_cells(vm, next_retained)?;
        if self.operands.len() == self.operands.capacity() {
            vm.charge_work(self.operands.len().saturating_add(1))?;
            self.operands
                .try_reserve(1)
                .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        }
        let operand = if let Some(ty) = result.expected {
            value.validate(vm.provider.types(), ty, vm.limits.evaluation_depth.min(256))?;
            Operand::Value(value)
        } else {
            Operand::Results(vec![value])
        };
        debug_assert_eq!(operand_cells(&operand, vm.limits.value_cells)?, cells);
        Ok(PreparedProcessResult {
            operand,
            cells,
            source_retained: self.retained,
            next_retained,
            serial,
            result,
            current: branch.current(),
            pair,
        })
    }
    pub(in crate::execute::resumable) fn commit_process_result<
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    >(
        &mut self,
        vm: &Vm<'_, P, E>,
        world: &VirtualProcesses,
        branch: &ProcessBranchState,
        prepared: PreparedProcessResult,
    ) -> Result<()> {
        let stop = self.process_stop()?;
        let ProcessControl::Fork(control) = &stop.control else {
            return Err(Error::InvalidIr("prepared fork result has another control").into());
        };
        if stop.serial != prepared.serial
            || stop.result != prepared.result
            || self.retained != prepared.source_retained
            || branch.current() != prepared.current
            || control.parent() != prepared.pair.parent
            || world.parent(prepared.pair.child).map_err(process_failure)?
                != Some(prepared.pair.parent)
        {
            return Err(Error::InvalidIr(
                "prepared process result is stale or belongs to another branch",
            )
            .into());
        }
        world
            .validate_running(branch.current())
            .map_err(process_failure)?;
        self.process_combined_cells(vm, prepared.next_retained)?;
        debug_assert_eq!(
            operand_cells(&prepared.operand, vm.limits.value_cells)?,
            prepared.cells
        );
        if self.operands.len() == self.operands.capacity() {
            return Err(
                Error::InvalidIr("prepared process result lost its reserved operand slot").into(),
            );
        }
        self.tasks.pop();
        self.operands.push(prepared.operand);
        self.retained = prepared.next_retained;
        Ok(())
    }
    pub(in crate::execute::resumable) fn retire_source_control<
        P: ProcedureProvider + ?Sized,
        E: CompilerEffects,
    >(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        branch: &ProcessBranchState,
    ) -> Result<()> {
        let stop = self.process_stop()?;
        let terminal = match &stop.control {
            ProcessControl::Exit(_) => true,
            ProcessControl::Exec(control) => control.stage() == ProcessExecStage::Replaced,
            ProcessControl::Fork(_) => false,
        };
        if stop.control.current() != branch.current() || !terminal {
            return Err(
                Error::InvalidIr("source retirement needs an exact exit or replacement").into(),
            );
        }
        // Storage disposal is not source unwind. The parent separately drops private frames/Memory.
        vm.charge_work(self.fork_work_cost()?)?;
        self.tasks = Vec::new();
        self.operands = Vec::new();
        self.plans = HashMap::new();
        self.initial_plan = None;
        self.retained = 0;
        self.plan_cells = 0;
        self.source_retired = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
