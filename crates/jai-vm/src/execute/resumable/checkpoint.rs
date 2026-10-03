//! Admission of the actual rollback owners before their first clone.
use super::*;

pub(super) struct Admission {
    pub(super) cells: usize,
    pub(super) resident_ancillary_cells: usize,
}

/// General rollback accepts live FILE capabilities. Fork's closed-stream rule
/// is enforced separately by BranchSnapshot::admit_fork.
pub(super) fn prepare<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    root_cells: usize,
) -> Result<Admission> {
    let storage = branches::measured(vm, false)?;
    let world = branches::shared_world_cells(vm)?;
    let cells = storage
        .cells
        .checked_add(world)
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    cells
        .checked_mul(2)
        .and_then(|cells| cells.checked_add(root_cells))
        .and_then(|cells| cells.checked_add(publication_origin_cells(vm)))
        .filter(|cells| *cells <= vm.limits.value_cells)
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    let resident_ancillary_cells = residual(vm, storage.cells)?;
    let work = storage
        .work
        .checked_add(world)
        .ok_or(Error::Limit(LimitKind::Fuel))?;
    vm.charge_work(work)?;
    Ok(Admission {
        cells,
        resident_ancillary_cells,
    })
}

pub(super) fn residual<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
    storage_cells: usize,
) -> Result<usize> {
    let branch = vm.processes.as_ref().map_or(0, |process| {
        process.branch.retained_metadata_cells().saturating_add(1)
    });
    storage_cells
        .checked_sub(vm.memory.value_cells())
        .and_then(|cells| cells.checked_sub(vm.expression_bindings.cells()))
        .and_then(|cells| cells.checked_sub(branch))
        .ok_or_else(|| Error::InvalidIr("live storage residual accounting underflow").into())
}
