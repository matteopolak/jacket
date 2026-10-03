//! Serial private branches under one source transaction and one effect cursor.
use super::*;
use crate::process_source_machine::ProcessBranchState;
use crate::virtual_process::{ProcessEvent, ProcessId};
use machine::process_control::ProcessControl;
use std::collections::BTreeMap;
#[cfg(test)]
mod tests;

struct ParkedBranch {
    storage: branches::BranchSnapshot,
    machine: machine::Machine,
    pending: Option<ProcessEvent>,
    cells: usize,
}

#[derive(Default)]
pub(super) struct ProcessScheduler {
    root: Option<ProcessId>,
    parked: BTreeMap<usize, ParkedBranch>,
    next_ordinal: usize,
    parked_cells: usize,
    rollback_cells: usize,
    resident_ancillary_cells: usize,
}

fn process_error(error: crate::virtual_process::ProcessError) -> Halt {
    Error::EffectRejected(error.to_string()).into()
}

impl ProcessScheduler {
    pub(super) fn new(rollback_cells: usize, resident_ancillary_cells: usize) -> Self {
        Self {
            rollback_cells,
            resident_ancillary_cells,
            ..Self::default()
        }
    }
    pub(super) fn publication_cells(&self) -> std::result::Result<usize, Error> {
        self.rollback_cells
            .checked_add(self.parked_cells)
            .and_then(|cells| cells.checked_add(self.resident_ancillary_cells))
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
    pub(super) fn refresh_publication_ancillary(&mut self, cells: usize) {
        self.resident_ancillary_cells = cells;
    }
    fn current<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        vm: &Vm<'_, P, E>,
    ) -> Result<ProcessId> {
        Ok(vm
            .processes
            .as_ref()
            .ok_or(Error::InvalidIr("process continuation has no shared world"))?
            .branch
            .current())
    }

    fn branch_cells(
        storage: &branches::BranchSnapshot,
        machine: &machine::Machine,
    ) -> Result<usize> {
        storage
            .cells()
            .checked_add(machine.retained_cells())
            .and_then(|cells| cells.checked_add(1))
            .ok_or_else(|| Error::Limit(LimitKind::ValueCells).into())
    }

    pub(super) fn drive<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        machine: &mut machine::Machine,
    ) -> Result<machine::DriveStatus> {
        loop {
            let original = vm.limits.value_cells;
            let memory_limit = vm.memory.replace_value_cell_limit(original)?;
            let previous_branch_mode = vm.process_branch_execution;
            vm.process_branch_execution = self.root.is_some();
            let progress =
                machine.drive_budgeted(vm, |vm, active| self.admit_live(vm, active, original));
            vm.process_branch_execution = previous_branch_mode;
            vm.limits.value_cells = original;
            vm.memory.replace_value_cell_limit(memory_limit)?;

            match progress {
                Ok(machine::DriveStatus::Complete) => {
                    if self
                        .root
                        .is_some_and(|root| Self::current(vm).ok() != Some(root))
                    {
                        return Err(Error::UnsupportedPointerOperation(
                            "a child continuation must exit or replace its process",
                        )
                        .into());
                    }
                    return Ok(machine::DriveStatus::Complete);
                }
                Ok(machine::DriveStatus::ProcessControl) => {
                    let control = machine
                        .process_control()
                        .ok_or(Error::InvalidIr("process stop has no sealed control"))?;
                    match control {
                        ProcessControl::Fork(control) => {
                            let parent = control.parent();
                            self.fork(vm, machine, parent)?;
                        }
                        ProcessControl::Exit(control) => {
                            let process = control.process();
                            let status = control.status();
                            self.exit(vm, machine, process, status)?;
                        }
                        ProcessControl::Exec(_) => {
                            return Err(Error::UnsupportedPointerOperation(
                                "process replacement requires a reviewed host completion route",
                            )
                            .into());
                        }
                    }
                }
                Ok(machine::DriveStatus::Retired) => {
                    return Err(
                        Error::InvalidIr("retired source has no scheduled successor").into(),
                    );
                }
                Err(Halt::Pending(Dependency::Process(event))) if self.root.is_some() => {
                    if !self.switch(vm, machine, Some(event), false)? {
                        return Err(Error::EffectRejected(
                            "virtual process deadlock: all source branches await internal events"
                                .into(),
                        )
                        .into());
                    }
                }
                // External readiness parks the entire source job, including this
                // queue. It never starts a sibling compiler/host request stream.
                Err(halt) => return Err(halt),
            }
        }
    }

    /// Restore the job's full quota, then reserve actual rollback, parked and
    /// live ancillary backing before each source action. Machine payload growth
    /// contributes its current cached shape, including sparse outer capacities.
    fn admit_live<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        machine: &machine::Machine,
        original: usize,
    ) -> Result<()> {
        vm.limits.value_cells = original;
        let storage = branches::measured(vm, false)?;
        let residual = checkpoint::residual(vm, storage.cells)?;
        let machine_extra = machine
            .retained_cells()
            .checked_sub(machine.accounted_cells())
            .ok_or(Error::InvalidIr("machine residual accounting underflow"))?;
        let reserved = self
            .parked_cells
            .checked_add(self.rollback_cells)
            .and_then(|cells| cells.checked_add(residual))
            .and_then(|cells| cells.checked_add(machine_extra))
            .and_then(|cells| cells.checked_add(publication_origin_cells(vm)))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let occupied = storage
            .cells
            .checked_add(branches::shared_world_cells(vm)?)
            .and_then(|cells| cells.checked_add(machine.retained_cells()))
            .and_then(|cells| cells.checked_add(self.parked_cells))
            .and_then(|cells| cells.checked_add(self.rollback_cells))
            .and_then(|cells| cells.checked_add(publication_origin_cells(vm)))
            .filter(|cells| *cells <= original)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let _ = occupied;
        let available = original
            .checked_sub(reserved)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        vm.memory.replace_value_cell_limit(available)?;
        vm.limits.value_cells = available;
        self.resident_ancillary_cells = residual;
        cache_drive_publication_retention(
            vm,
            original,
            self.rollback_cells
                .checked_add(self.parked_cells)
                .and_then(|cells| cells.checked_add(machine.retained_cells()))
                .ok_or(Error::Limit(LimitKind::ValueCells))?,
            residual,
        )?;
        Ok(())
    }

    fn fork<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        machine: &mut machine::Machine,
        parent: ProcessId,
    ) -> Result<()> {
        // Checked before inspection, copies, table mutation or PID allocation.
        let ordinal = self.next_ordinal;
        let next_ordinal = ordinal
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        if Self::current(vm)? != parent {
            return Err(Error::InvalidIr("fork control belongs to another branch").into());
        }
        let cached_work = vm.processes.as_ref().unwrap().work_cost();
        vm.charge_work(usize::try_from(cached_work).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
        let descriptors = vm
            .processes
            .as_ref()
            .unwrap()
            .world
            .fork_descriptor_cells(parent)
            .map_err(process_error)?;
        let world_cells = branches::shared_world_cells(vm)?;
        let candidate_world_cells = world_cells
            .checked_add(descriptors)
            .and_then(|cells| cells.checked_add(1))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let machine_work = machine.fork_work_cost()?;
        // Both saved destinations may grow their operand tables during result
        // preparation. Reserve those capacities and temporary descriptor slots.
        let injection = machine.fork_result_scratch_cells()?;
        let other = self
            .parked_cells
            .checked_add(self.rollback_cells)
            .and_then(|cells| cells.checked_add(machine.retained_cells().checked_mul(2)?))
            .and_then(|cells| cells.checked_add(candidate_world_cells))
            .and_then(|cells| cells.checked_add(descriptors))
            .and_then(|cells| cells.checked_add(injection))
            .and_then(|cells| cells.checked_add(1))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let admitted = branches::BranchSnapshot::admit_fork(vm, other)?;
        let world_work = candidate_world_cells
            .checked_mul(3)
            .and_then(|work| work.checked_add(descriptors))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        let clone_work = world_work
            .checked_add(machine_work)
            .and_then(|work| work.checked_add(admitted.work))
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        // Result preparation has a bounded stop/operand reservation cost. Check
        // every remaining fuel charge before the candidate can mint a child PID.
        let remaining_work = clone_work
            .checked_add(injection)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        vm.statistics
            .steps
            .checked_add(u64::try_from(remaining_work).map_err(|_| Error::Limit(LimitKind::Fuel))?)
            .filter(|steps| *steps <= vm.limits.fuel)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        vm.charge_work(clone_work)?;
        // Every deep owner and the mutator's temporary slot copy were admitted.
        let mut world = vm.processes.as_ref().unwrap().world.clone();
        let pair = world.fork(parent).map_err(process_error)?;
        let actual_world_cells = usize::try_from(world.work_cost().map_err(process_error)?)
            .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        if actual_world_cells > candidate_world_cells {
            return Err(Error::InvalidIr("fork exceeded admitted world shape").into());
        }
        let parent_branch = vm.processes.as_ref().unwrap().branch.clone();
        let child_branch = parent_branch.for_child(pair.child, &world)?;
        let mut storage = branches::BranchSnapshot::clone_admitted(vm, admitted)?;
        storage.rebind_child(child_branch)?;
        let mut child_machine = machine.fork_private(machine_work)?;
        let parent_result = machine.prepare_fork_result(vm, &world, &parent_branch, pair)?;
        let child_result =
            child_machine.prepare_fork_result(vm, &world, storage.process_branch(), pair)?;
        machine.commit_process_result(vm, &world, &parent_branch, parent_result)?;
        child_machine.commit_process_result(vm, &world, storage.process_branch(), child_result)?;
        let cells = Self::branch_cells(&storage, &child_machine)?;
        let queued = self
            .parked_cells
            .checked_add(cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let state = vm.processes.as_mut().unwrap();
        state.world = world;
        state.refresh(vm.limits)?;
        self.root.get_or_insert(parent);
        self.parked.insert(
            ordinal,
            ParkedBranch {
                storage,
                machine: child_machine,
                pending: None,
                cells,
            },
        );
        self.next_ordinal = next_ordinal;
        self.parked_cells = queued;
        Ok(())
    }

    fn ready<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &self,
        vm: &mut Vm<'_, P, E>,
    ) -> Result<Option<usize>> {
        vm.charge_work(self.parked.len())?;
        let world = &vm
            .processes
            .as_ref()
            .ok_or(Error::InvalidIr(
                "scheduled branch has no shared process world",
            ))?
            .world;
        for (ordinal, branch) in &self.parked {
            if branch.pending.is_none()
                || world
                    .event_ready(branch.pending.unwrap())
                    .map_err(process_error)?
            {
                return Ok(Some(*ordinal));
            }
        }
        Ok(None)
    }

    fn switch<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        machine: &mut machine::Machine,
        pending: Option<ProcessEvent>,
        retiring: bool,
    ) -> Result<bool> {
        let ordinal = self.next_ordinal;
        let next_ordinal = if retiring {
            ordinal
        } else {
            ordinal
                .checked_add(1)
                .ok_or(Error::Limit(LimitKind::Fuel))?
        };
        let Some(index) = self.ready(vm)? else {
            return Ok(false);
        };
        let selected = self
            .parked
            .get(&index)
            .expect("ready branch remains parked");
        let other_parked = self
            .parked_cells
            .checked_sub(selected.cells)
            .ok_or(Error::InvalidIr("parked branch accounting underflow"))?;
        let other = other_parked
            .checked_add(self.rollback_cells)
            .and_then(|cells| cells.checked_add(machine.retained_cells()))
            .and_then(|cells| cells.checked_add(selected.machine.retained_cells()))
            .and_then(|cells| cells.checked_add(1))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        // A failed swap admission leaves the queue and active owners untouched.
        self.parked
            .get_mut(&index)
            .unwrap()
            .storage
            .swap_live(vm, other)?;
        let mut next = self.parked.remove(&index).unwrap();
        std::mem::swap(machine, &mut next.machine);
        self.parked_cells = other_parked;
        if !retiring {
            next.pending = pending;
            next.cells = Self::branch_cells(&next.storage, &next.machine)?;
            self.parked_cells = self
                .parked_cells
                .checked_add(next.cells)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            self.parked.insert(ordinal, next);
            self.next_ordinal = next_ordinal;
        }

        Ok(true)
    }

    fn exit<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        machine: &mut machine::Machine,
        process: ProcessId,
        status: i32,
    ) -> Result<()> {
        if self.root == Some(process) {
            return Err(Error::UnsupportedPointerOperation(
                "the source root exited without a publishable result",
            )
            .into());
        }
        if Self::current(vm)? != process {
            return Err(Error::InvalidIr("exit control belongs to another branch").into());
        }
        let work = vm.processes.as_ref().unwrap().work_cost();
        vm.charge_work(usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
        let branch: ProcessBranchState = vm.processes.as_ref().unwrap().branch.clone();
        // Disposal is admitted before publishing the termination. It deliberately
        // skips source cleanup, function returns and result publication.
        machine.retire_source_control(vm, &branch)?;
        vm.processes
            .as_mut()
            .unwrap()
            .world
            .exit(process, status)
            .map_err(process_error)?;
        vm.processes.as_mut().unwrap().refresh(vm.limits)?;
        if !self.switch(vm, machine, None, true)? {
            return Err(Error::EffectRejected(
                "terminated child has no runnable source parent".into(),
            )
            .into());
        }
        Ok(())
    }
}
