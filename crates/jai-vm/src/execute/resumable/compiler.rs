//! Compiler control over real VM leaves; the containing Session owns the journal.
use super::*;
use crate::compiler_code_plan::{
    CheckedCompilerCodePlan, CheckedCompilerRuntimeLeaf, CompilerCodePlanId, CompilerControl,
    CompilerControlId, CompilerReturnSiteId, CompilerRuntimeInput, CompilerSlotId,
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FRAME: AtomicU64 = AtomicU64::new(1);

/// A retained compiler frame is distinct from every native LocalId/Procedure frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompilerFrameId {
    plan: CompilerCodePlanId,
    serial: u64,
}
impl CompilerFrameId {
    pub fn plan(self) -> CompilerCodePlanId {
        self.plan
    }
}

/// Only a reached Return control can create this read-only publication view.
pub struct CompilerCodeSelection<'a> {
    site: CompilerReturnSiteId,
    publication_occurrence: u64,
    frame: &'a CompilerFrame,
    captures: &'a [CompilerSlotId],
    capture_flags: &'a [bool],
}
impl CompilerCodeSelection<'_> {
    pub fn site(&self) -> CompilerReturnSiteId {
        self.site
    }
    /// One logical selected-capture event within the canonical source origin.
    /// Resume and publication retry retain this event; arena/frame serials do
    /// not participate in replay or native constant identity.
    pub fn publication_occurrence(&self) -> u64 {
        self.publication_occurrence
    }
    pub fn frame(&self) -> CompilerFrameId {
        self.frame.id
    }
    pub fn captures(&self) -> &[CompilerSlotId] {
        self.captures
    }
    pub fn native_value(
        &self,
        slot: CompilerSlotId,
    ) -> std::result::Result<(TypeId, &Value), Error> {
        if slot.plan() != self.frame.id.plan
            || !self
                .capture_flags
                .get(slot.index())
                .copied()
                .unwrap_or(false)
        {
            return Err(Error::InvalidIr(
                "compiler slot is outside the selected capture",
            ));
        }
        let value = self.frame.slot(slot)?;
        Ok((
            value.ty,
            value
                .value
                .as_ref()
                .ok_or(Error::InvalidIr("uninitialized compiler capture"))?,
        ))
    }
}

struct Slot {
    ty: TypeId,
    value: Option<Value>,
    cells: usize,
}
struct CompilerFrame {
    id: CompilerFrameId,
    slots: Vec<Slot>,
    cells: usize,
}
impl CompilerFrame {
    fn slot(&self, slot: CompilerSlotId) -> std::result::Result<&Slot, Error> {
        if slot.plan() != self.id.plan {
            return Err(Error::InvalidIr(
                "compiler slot belongs to another frame plan",
            ));
        }
        self.slots
            .get(slot.index())
            .ok_or(Error::InvalidIr("unknown compiler slot"))
    }
}
enum Control {
    Evaluate(usize),
    Assign {
        slot: CompilerSlotId,
        leaf: usize,
    },
    Block {
        children: Box<[CompilerControlId]>,
        locals: Box<[CompilerSlotId]>,
    },
    If {
        condition: usize,
        then_control: CompilerControlId,
        else_control: CompilerControlId,
    },
    Return(CompilerReturnSiteId),
}
struct Leaf {
    code: Arc<plan::Plan>,
    inputs: Box<[CompilerRuntimeInput]>,
}
#[derive(Clone, Copy)]
struct SelectedReturn {
    site: CompilerReturnSiteId,
    publication_occurrence: u64,
}
enum Work {
    Enter(CompilerControlId),
    BlockNext {
        block: CompilerControlId,
        index: usize,
    },
}
#[derive(Clone, Copy)]
enum Goal {
    Discard,
    Assign(CompilerSlotId),
    Decide {
        yes: CompilerControlId,
        no: CompilerControlId,
    },
}
struct ActiveLeaf {
    leaf: usize,
    goal: Goal,
    scope: Option<bindings::ScopeToken>,
    next_input: usize,
    input_cells: usize,
    machine: Option<machine::Machine>,
    values: Option<Vec<Value>>,
    value_cells: Option<usize>,
}
impl ActiveLeaf {
    fn retained_cells(&self, code_cells: usize) -> std::result::Result<usize, Error> {
        let mut cells = sum(1, self.value_cells.unwrap_or(0))?;
        cells = sum(cells, self.values.as_ref().map_or(0, Vec::capacity))?;
        if let Some(machine) = &self.machine {
            cells = sum(cells, machine.retained_cells().saturating_sub(code_cells))?;
        }
        Ok(cells)
    }
}

pub(super) struct CompilerController {
    id: CompilerCodePlanId,
    ambient_owner: ProcedureId,
    controls: Vec<Control>,
    leaves: Vec<Leaf>,
    sites: Vec<Option<Box<[CompilerSlotId]>>>,
    capture_flags: Vec<bool>,
    frame: CompilerFrame,
    work: Vec<Work>,
    active: Option<ActiveLeaf>,
    selected: Option<SelectedReturn>,
    fixed_cells: usize,
    planning_work: usize,
    initialized: bool,
}

fn sum(a: usize, b: usize) -> std::result::Result<usize, Error> {
    a.checked_add(b).ok_or(Error::Limit(LimitKind::ValueCells))
}
fn bound(total: usize, limit: usize) -> std::result::Result<(), Error> {
    if total > limit {
        Err(Error::Limit(LimitKind::ValueCells))
    } else {
        Ok(())
    }
}

impl CompilerController {
    /// Consume verified leaves; retain no AST or borrowed provider references.
    pub(super) fn compile(
        checked: CheckedCompilerCodePlan<'_>,
        limits: Limits,
    ) -> std::result::Result<Self, Error> {
        let (source, proofs, ambient_owner) = checked.into_parts();
        let inspection = sum(
            sum(source.controls().len(), source.runtime_leaves().len())?,
            source.return_site_count(),
        )?;
        if u64::try_from(inspection).map_err(|_| Error::Limit(LimitKind::Fuel))? > limits.fuel {
            return Err(Error::Limit(LimitKind::Fuel));
        }
        let mut cells = sum(
            source.metadata_size()?,
            source
                .slots()
                .len()
                .checked_mul(2)
                .ok_or(Error::Limit(LimitKind::ValueCells))?,
        )?;
        bound(cells, limits.value_cells)?;
        // Retained control/frame/leaf/site/capture/work containers and the
        // boxed controller each retain a header even when empty.
        cells = sum(cells, 7)?;
        bound(cells, limits.value_cells)?;
        // An acyclic control path retains at most one cursor per control plus
        // its next entry. Reserve the complete stack before any execution.
        let work_capacity = sum(source.controls().len(), 1)?;
        bound(sum(cells, work_capacity)?, limits.value_cells)?;
        let mut planning_work = sum(sum(cells, work_capacity)?, inspection)?;
        if u64::try_from(planning_work).map_err(|_| Error::Limit(LimitKind::Fuel))? > limits.fuel {
            return Err(Error::Limit(LimitKind::Fuel));
        }
        let mut leaves = Vec::with_capacity(proofs.len());
        for (proof, source_leaf) in proofs.into_iter().zip(source.runtime_leaves()) {
            let remaining = limits
                .value_cells
                .checked_sub(sum(cells, work_capacity)?)
                .ok_or(Error::Limit(LimitKind::ValueCells))?;
            let remaining_fuel = limits
                .fuel
                .checked_sub(
                    u64::try_from(planning_work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                )
                .ok_or(Error::Limit(LimitKind::Fuel))?;
            let leaf_limits = Limits {
                value_cells: remaining,
                fuel: remaining_fuel,
                ..limits
            };
            let code = match proof {
                CheckedCompilerRuntimeLeaf::Expression(proof) => {
                    plan::compile_checked_expression(proof, leaf_limits)?
                }
                CheckedCompilerRuntimeLeaf::Call(proof) => {
                    plan::compile_checked_call(proof, leaf_limits)?
                }
            };
            cells = sum(cells, code.metadata_cells)?;
            bound(sum(cells, work_capacity)?, limits.value_cells)?;
            planning_work = sum(
                planning_work,
                usize::try_from(code.planning_work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
            )?;
            if u64::try_from(planning_work).map_err(|_| Error::Limit(LimitKind::Fuel))?
                > limits.fuel
            {
                return Err(Error::Limit(LimitKind::Fuel));
            }
            leaves.push(Leaf {
                code,
                inputs: source_leaf.inputs().into(),
            });
        }
        let mut sites: Vec<Option<Box<[CompilerSlotId]>>> =
            (0..source.return_site_count()).map(|_| None).collect();
        let mut controls = Vec::with_capacity(source.controls().len());
        for control in source.controls() {
            controls.push(match control {
                CompilerControl::Evaluate(leaf) => Control::Evaluate(*leaf),
                CompilerControl::Assign { slot, leaf } => Control::Assign {
                    slot: *slot,
                    leaf: *leaf,
                },
                CompilerControl::Block { children, locals } => Control::Block {
                    children: children.clone(),
                    locals: locals.clone(),
                },
                CompilerControl::If {
                    condition,
                    then_control,
                    else_control,
                } => Control::If {
                    condition: *condition,
                    then_control: *then_control,
                    else_control: *else_control,
                },
                CompilerControl::Return(site) => {
                    sites[site.index()].get_or_insert_with(|| {
                        source
                            .return_site_captures(*site)
                            .expect("verified return site")
                            .into()
                    });
                    Control::Return(*site)
                }
            });
        }
        let serial = NEXT_FRAME
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |serial| {
                serial.checked_add(1)
            })
            .map_err(|_| Error::InvalidIr("compiler frame identities exhausted"))?;
        let mut work = Vec::with_capacity(work_capacity);
        work.push(Work::Enter(source.root()));
        Ok(Self {
            id: source.id(),
            ambient_owner,
            controls,
            leaves,
            sites,
            capture_flags: vec![false; source.slots().len()],
            frame: CompilerFrame {
                id: CompilerFrameId {
                    plan: source.id(),
                    serial,
                },
                slots: source
                    .slots()
                    .iter()
                    .map(|schema| Slot {
                        ty: schema.ty,
                        value: None,
                        cells: 0,
                    })
                    .collect(),
                cells: 0,
            },
            work,
            active: None,
            selected: None,
            fixed_cells: cells,
            planning_work,
            initialized: false,
        })
    }
    pub(super) fn retained_cells(&self) -> std::result::Result<usize, Error> {
        let mut cells = sum(
            sum(self.fixed_cells, self.frame.cells)?,
            self.work.capacity(),
        )?;
        if let Some(active) = &self.active {
            cells = sum(
                cells,
                active.retained_cells(self.leaves[active.leaf].code.metadata_cells)?,
            )?;
        }
        Ok(cells)
    }
    pub(super) fn retain_metadata(
        &mut self,
        cells: usize,
        limit: usize,
    ) -> std::result::Result<(), Error> {
        let next = sum(self.fixed_cells, cells)?;
        bound(sum(self.retained_cells()?, cells)?, limit)?;
        self.fixed_cells = next;
        Ok(())
    }
    pub(super) fn selection(&self) -> Option<CompilerCodeSelection<'_>> {
        let selected = self.selected?;
        Some(CompilerCodeSelection {
            site: selected.site,
            publication_occurrence: selected.publication_occurrence,
            frame: &self.frame,
            captures: self.sites[selected.site.index()]
                .as_deref()
                .expect("selected compiler site"),
            capture_flags: &self.capture_flags,
        })
    }
    fn control(&self, id: CompilerControlId) -> Result<&Control> {
        if id.plan() != self.id {
            return Err(Error::InvalidIr("compiler control belongs to another plan").into());
        }
        self.controls
            .get(id.index())
            .ok_or_else(|| Error::InvalidIr("unknown compiler control").into())
    }
    fn admit<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &self,
        vm: &Vm<'_, P, E>,
        extra: usize,
    ) -> Result<()> {
        let resident = sum(self.retained_cells()?, extra)?;
        let total = sum(
            sum(resident, vm.memory.value_cells())?,
            vm.expression_bindings.cells(),
        )?;
        bound(
            sum(
                total,
                vm.processes
                    .as_ref()
                    .map_or(0, process::ProcessState::cells),
            )?,
            vm.limits.value_cells,
        )?;
        Ok(())
    }
    fn push<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        work: Work,
    ) -> Result<()> {
        if self.work.len() == self.work.capacity() {
            return Err(
                Error::InvalidIr("compiler control exceeds its checked stack bound").into(),
            );
        }
        self.admit(vm, 0)?;
        vm.charge_work(1)?;
        self.work.push(work);
        Ok(())
    }
    fn start_leaf(&mut self, leaf: usize, goal: Goal) -> Result<()> {
        if self.active.is_some() || leaf >= self.leaves.len() {
            return Err(Error::InvalidIr("invalid compiler leaf transition").into());
        }
        self.active = Some(ActiveLeaf {
            leaf,
            goal,
            scope: None,
            next_input: 0,
            input_cells: 0,
            machine: None,
            values: None,
            value_cells: None,
        });
        Ok(())
    }

    /// Pending retains the exact control/input/native-machine/output state.
    /// The Session forwards outer suspend/resume and performs sole publication.
    /// Reserve the rollback holder; live ancillary ownership is measured anew
    /// for each compiler transition and each native machine step.
    pub(super) fn drive_with_checkpoint<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        rollback_cells: usize,
    ) -> Result<()> {
        let original = vm.limits.value_cells;
        let memory_limit = vm.memory.replace_value_cell_limit(original)?;
        let result = self.drive(vm, original, rollback_cells);
        vm.limits.value_cells = original;
        vm.memory.replace_value_cell_limit(memory_limit)?;
        result
    }

    pub(super) fn drive<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        original: usize,
        rollback_cells: usize,
    ) -> Result<()> {
        admit_live(vm, self.retained_cells()?, None, original, rollback_cells)?;
        if !self.initialized {
            self.admit(vm, 0)?;
            vm.charge_work(self.planning_work)?;
            self.initialized = true;
        }
        loop {
            admit_live(vm, self.retained_cells()?, None, original, rollback_cells)?;
            if self.selected.is_some() {
                return Ok(());
            }
            if let Some(mut active) = self.active.take() {
                let result = self.drive_leaf(vm, &mut active, original, rollback_cells);
                match result {
                    Ok(()) => continue,
                    Err(halt) => {
                        self.active = Some(active);
                        return Err(halt);
                    }
                }
            }
            vm.step(self.work.len())?;
            let work = self
                .work
                .pop()
                .ok_or(Error::InvalidIr("compiler plan completed without Code"))?;
            match work {
                Work::Enter(id) => match self.control(id)? {
                    Control::Evaluate(leaf) => self.start_leaf(*leaf, Goal::Discard)?,
                    Control::Assign { slot, leaf } => {
                        self.start_leaf(*leaf, Goal::Assign(*slot))?
                    }
                    Control::If {
                        condition,
                        then_control,
                        else_control,
                    } => self.start_leaf(
                        *condition,
                        Goal::Decide {
                            yes: *then_control,
                            no: *else_control,
                        },
                    )?,
                    Control::Block { locals, .. } => {
                        vm.charge_work(locals.len())?;
                        for slot in locals {
                            if self.frame.slot(*slot)?.value.is_some() {
                                return Err(Error::InvalidIr(
                                    "compiler block local is already live",
                                )
                                .into());
                            }
                        }
                        self.push(
                            vm,
                            Work::BlockNext {
                                block: id,
                                index: 0,
                            },
                        )?;
                    }
                    Control::Return(site) => {
                        let site = *site;
                        let publication_occurrence = u64::try_from(site.index())
                            .ok()
                            .and_then(|index| index.checked_add(1))
                            .ok_or(Error::InvalidIr(
                                "compiler publication occurrences exhausted",
                            ))?;
                        let captures = self.sites[site.index()]
                            .as_deref()
                            .expect("verified compiler site");
                        vm.charge_work(captures.len())?;
                        for slot in captures {
                            self.frame
                                .slot(*slot)?
                                .value
                                .as_ref()
                                .ok_or(Error::InvalidIr("uninitialized compiler capture"))?;
                        }
                        for slot in captures {
                            self.capture_flags[slot.index()] = true;
                        }
                        self.selected = Some(SelectedReturn {
                            site,
                            publication_occurrence,
                        });
                    }
                },
                Work::BlockNext { block, index } => {
                    let Control::Block { children, locals } = self.control(block)? else {
                        return Err(Error::InvalidIr("compiler block cursor has no block").into());
                    };
                    if let Some(child) = children.get(index).copied() {
                        self.push(
                            vm,
                            Work::BlockNext {
                                block,
                                index: index + 1,
                            },
                        )?;
                        self.push(vm, Work::Enter(child))?;
                    } else {
                        let work = locals.iter().try_fold(locals.len(), |cells, slot| {
                            sum(cells, self.frame.slot(*slot)?.cells)
                        })?;
                        vm.charge_work(work)?;
                        self.admit(vm, locals.len())?;
                        let indices: Vec<_> = locals.iter().map(|slot| slot.index()).collect();
                        for index in indices {
                            let slot = &mut self.frame.slots[index];
                            self.frame.cells =
                                self.frame.cells.checked_sub(slot.cells).ok_or(
                                    Error::InvalidIr("compiler frame accounting underflow"),
                                )?;
                            slot.value = None;
                            slot.cells = 0;
                        }
                    }
                }
            }
        }
    }

    fn drive_leaf<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        active: &mut ActiveLeaf,
        original: usize,
        rollback_cells: usize,
    ) -> Result<()> {
        if active.values.is_none() {
            if active.scope.is_none() {
                active.scope = Some(vm.begin_bindings(sum(self.retained_cells()?, 1)?)?);
            }
            while let Some(input) = self.leaves[active.leaf]
                .inputs
                .get(active.next_input)
                .copied()
            {
                if input.binding.procedure() != self.ambient_owner {
                    return Err(Error::InvalidIr("compiler input owner changed").into());
                }
                let slot = self.frame.slot(input.slot)?;
                if slot.ty != input.ty {
                    return Err(Error::InvalidIr("compiler leaf input type changed").into());
                }
                let value = slot
                    .value
                    .as_ref()
                    .ok_or(Error::InvalidIr("uninitialized compiler leaf input"))?;
                self.admit(vm, sum(slot.cells, 2)?)?;
                vm.charge_work(sum(slot.cells, 1)?)?;
                let value = value.clone();
                vm.capture_binding(input.binding, value, sum(self.retained_cells()?, 1)?)?;
                active.input_cells = sum(active.input_cells, sum(slot.cells, 1)?)?;
                active.next_input += 1;
            }
            if active.machine.is_none() {
                self.admit(vm, 1)?;
                active.machine = Some(machine::Machine::expression(Arc::clone(
                    &self.leaves[active.leaf].code,
                )));
            }
            // Existing VM storage admission sees every compiler-local/other-leaf
            // owner. This leaf's immutable code is counted by Machine itself.
            let external = sum(self.retained_cells()?, 1)?
                .checked_sub(self.leaves[active.leaf].code.metadata_cells)
                .ok_or(Error::InvalidIr("compiler code accounting underflow"))?;
            let memory_limit = vm.memory.replace_value_cell_limit(original)?;
            let progress = active
                .machine
                .as_mut()
                .unwrap()
                .drive_budgeted(vm, |vm, machine| {
                    admit_live(vm, external, Some(machine), original, rollback_cells)
                });
            vm.limits.value_cells = original;
            vm.memory.replace_value_cell_limit(memory_limit)?;
            match progress? {
                machine::DriveStatus::Complete => {}
                machine::DriveStatus::ProcessControl | machine::DriveStatus::Retired => {
                    return Err(Error::UnsupportedPointerOperation(
                        "compiler Code frames require a reviewed process-control transport",
                    )
                    .into());
                }
            }
            active.values = Some(
                active
                    .machine
                    .as_mut()
                    .unwrap()
                    .take_values()
                    .ok_or(Error::InvalidIr("compiler leaf has no yielded values"))?,
            );
            vm.charge_work(active.input_cells)?;
            vm.end_bindings(active.scope.take().expect("compiler leaf input scope"))?;
            vm.charge_work(active.machine.as_ref().unwrap().retained_cells())?;
            active.machine = None;
        }
        if active.value_cells.is_none() {
            let mut cells = 0;
            for value in active.values.as_ref().unwrap() {
                cells = sum(cells, metered_cells(vm, value, 0)?)?;
            }
            active.value_cells = Some(cells);
        }
        let active_cells = active.retained_cells(self.leaves[active.leaf].code.metadata_cells)?;
        admit_live(
            vm,
            sum(self.retained_cells()?, active_cells)?,
            None,
            original,
            rollback_cells,
        )?;
        self.admit(vm, active_cells)?;
        match active.goal {
            Goal::Discard => {
                vm.charge_work(active.value_cells.unwrap())?;
                active.values = None;
            }
            Goal::Assign(id) => {
                let values = active.values.as_ref().unwrap();
                if values.len() != 1 {
                    return Err(
                        Error::InvalidIr("compiler assignment has the wrong result count").into(),
                    );
                }
                let slot = self.frame.slot(id)?;
                vm.charge_work(sum(
                    active
                        .value_cells
                        .unwrap()
                        .checked_mul(2)
                        .ok_or(Error::Limit(LimitKind::Fuel))?,
                    slot.cells,
                )?)?;
                values[0].validate(vm.provider.types(), slot.ty, vm.limits.evaluation_depth)?;
                vm.memory
                    .validate_runtime_type_values(vm.provider.types(), &values[0])?;
                let next = self
                    .frame
                    .cells
                    .checked_sub(slot.cells)
                    .and_then(|n| n.checked_add(active.value_cells.unwrap()))
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                let value = active.values.as_mut().unwrap().pop().unwrap();
                let slot = &mut self.frame.slots[id.index()];
                slot.value = Some(value);
                slot.cells = active.value_cells.unwrap();
                self.frame.cells = next;
            }
            Goal::Decide { yes, no } => {
                let values = active.values.as_ref().unwrap();
                if values.len() != 1 {
                    return Err(
                        Error::InvalidIr("compiler condition has the wrong result count").into(),
                    );
                }
                let selected = if values[0].boolean()? { yes } else { no };
                self.push(vm, Work::Enter(selected))?;
            }
        }
        Ok(())
    }
}

/// The same live storage accounting used by the native root scheduler. Compiler
/// ownership is disjoint from the leaf Machine's complete cached footprint.
fn admit_live<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    compiler_cells: usize,
    machine: Option<&machine::Machine>,
    original: usize,
    rollback_cells: usize,
) -> Result<()> {
    vm.limits.value_cells = original;
    let storage = branches::measured(vm, false)?;
    let residual = checkpoint::residual(vm, storage.cells)?;
    let origin_cells = publication_origin_cells(vm);
    let machine_cells = machine.map_or(0, machine::Machine::retained_cells);
    let machine_extra = match machine {
        Some(machine) => {
            machine_cells
                .checked_sub(machine.accounted_cells())
                .ok_or(Error::InvalidIr(
                    "compiler leaf residual accounting underflow",
                ))?
        }
        None => 0,
    };
    let occupied = sum(
        sum(
            sum(storage.cells, branches::shared_world_cells(vm)?)?,
            compiler_cells,
        )?,
        sum(sum(machine_cells, rollback_cells)?, origin_cells)?,
    )?;
    bound(occupied, original)?;
    cache_drive_publication_retention(
        vm,
        original,
        sum(sum(rollback_cells, compiler_cells)?, machine_cells)?,
        residual,
    )?;
    let mut reserved = sum(sum(rollback_cells, residual)?, origin_cells)?;
    if machine.is_some() {
        reserved = sum(reserved, sum(compiler_cells, machine_extra)?)?;
    }
    let available = original
        .checked_sub(reserved)
        .ok_or(Error::Limit(LimitKind::ValueCells))?;
    vm.memory.replace_value_cell_limit(available)?;
    vm.limits.value_cells = available;
    Ok(())
}

/// Inspect incrementally before any new frame copy/type-validation walk.
fn metered_cells<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    value: &Value,
    depth: usize,
) -> Result<usize> {
    vm.step(depth)?;
    let mut cells = 1;
    match value {
        Value::String(bytes) => {
            vm.charge_work(bytes.len())?;
            cells = sum(cells, bytes.capacity())?;
        }
        Value::Pointer(pointer)
        | Value::Slice { pointer, .. }
        | Value::StringView { pointer, .. } => cells = sum(cells, pointer.metadata_cells())?,
        Value::Type { descriptor } => {
            cells = sum(
                cells,
                descriptor.as_ref().map_or(0, Pointer::metadata_cells),
            )?
        }
        Value::AddressInteger(number) => cells = sum(cells, number.metadata_cells())?,
        Value::Record { fields, .. }
        | Value::Array {
            elements: fields, ..
        } => {
            cells = sum(cells, fields.capacity().saturating_sub(fields.len()))?;
            for value in fields {
                cells = sum(cells, metered_cells(vm, value, depth + 1)?)?;
            }
        }
        Value::Union { value, .. } | Value::Distinct { value, .. } => {
            cells = sum(cells, metered_cells(vm, value, depth + 1)?)?
        }
        Value::DynamicArray {
            pointer, allocator, ..
        } => {
            cells = sum(cells, pointer.metadata_cells())?;
            if let Some(value) = allocator {
                cells = sum(cells, metered_cells(vm, value, depth + 1)?)?;
            }
        }
        Value::StoredAggregate(snapshot) => {
            vm.charge_work(snapshot.image().metadata_inspection_work())?;
            cells = sum(cells, snapshot.storage_cells())?;
            if let Some(value) = snapshot.decoded_semantic() {
                cells = sum(cells, metered_cells(vm, value, depth + 1)?)?;
            }
        }
        Value::Int(_)
        | Value::Bool(_)
        | Value::Float(_)
        | Value::Enum { .. }
        | Value::Procedure { .. } => {}
    }
    bound(cells, vm.limits.value_cells)?;
    Ok(cells)
}

mod entry;

#[cfg(test)]
mod tests;
