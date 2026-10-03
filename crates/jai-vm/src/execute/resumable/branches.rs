//! Scheduler-private branch data; IPC, effects, fuel and immutable code stay shared.
use super::*;
use crate::process_source_machine::ProcessBranchState;

pub(super) struct BranchSnapshot {
    memory: Memory,
    host_files: crate::host_files::HostFileMachine,
    host_heap: crate::virtual_heap::VirtualHeap,
    branch: ProcessBranchState,
    frames: Vec<OwnedFrame>,
    expression_bindings: bindings::BindingEnvironment,
    globals: Vec<Option<Pointer>>,
    static_objects: HashMap<StaticObjectId, Pointer>,
    static_publications: HashMap<u64, usize>,
    static_data: HashMap<u64, Arc<StaticData>>,
    literal_backing: HashMap<(TypeId, Value), Pointer>,
    current_context: Option<Pointer>,
    default_context: Option<Pointer>,
    root_temporaries: Vec<Pointer>,
    root_sequence_temp_bytes: u64,
    cells: usize,
}
pub(super) struct Bounds {
    pub(super) cells: usize,
    pub(super) work: usize,
    pub(super) memory_work: usize,
    pub(super) binding_work: usize,
}

fn add(left: usize, right: usize) -> std::result::Result<usize, Error> {
    left.checked_add(right)
        .ok_or(Error::Limit(LimitKind::ValueCells))
}
fn fuel(count: usize) -> std::result::Result<u64, Error> {
    u64::try_from(count).map_err(|_| Error::Limit(LimitKind::Fuel))
}
fn pointer_cells(pointer: &Pointer) -> std::result::Result<usize, Error> {
    add(1, pointer.metadata_cells())
}
fn option_pointer_cells(pointer: Option<&Pointer>) -> std::result::Result<usize, Error> {
    pointer.map_or(Ok(0), pointer_cells)
}

/// Count borrowed literal-key payloads incrementally, charging before descending
/// or inspecting image metadata. Nothing is cloned or realized by this walk.
fn value_cells(
    value: &Value,
    depth: usize,
    maximum_depth: usize,
    charge: &mut impl FnMut(u64) -> std::result::Result<(), Error>,
) -> std::result::Result<usize, Error> {
    if depth > maximum_depth.min(256) {
        return Err(Error::Limit(LimitKind::EvaluationDepth));
    }
    charge(1)?;
    let mut cells = 1;
    match value {
        Value::String(bytes) => {
            // Reading Vec capacity does not inspect its byte payload. Retain the
            // full capacity here; Bounds::work separately charges actual copies.
            cells = add(cells, bytes.capacity())?;
        }
        Value::Pointer(pointer)
        | Value::Slice {
            pointer, ..
        }
        | Value::StringView {
            pointer, ..
        } => cells = add(cells, pointer.metadata_cells())?,
        Value::Type {
            descriptor,
        } => cells = add(cells, option_pointer_cells(descriptor.as_ref())?)?,
        Value::AddressInteger(number) => {
            cells = add(cells, number.origin_count())?;
            if let Some(crate::AddressProvenance::Pointer(pointer)) = number.provenance() {
                cells = add(cells, pointer.metadata_cells())?;
            }
        }
        Value::Array {
            elements, ..
        }
        | Value::Record {
            fields: elements, ..
        } => {
            cells = add(cells, elements.capacity().saturating_sub(elements.len()))?;
            for element in elements {
                cells = add(
                    cells,
                    value_cells(element, depth + 1, maximum_depth, charge)?,
                )?;
            }
        }
        Value::Distinct {
            value, ..
        }
        | Value::Union {
            value, ..
        } => {
            cells = add(cells, value_cells(value, depth + 1, maximum_depth, charge)?)?;
        }
        Value::DynamicArray {
            pointer,
            allocator,
            ..
        } => {
            cells = add(cells, pointer.metadata_cells())?;
            if let Some(allocator) = allocator {
                cells = add(
                    cells,
                    value_cells(allocator, depth + 1, maximum_depth, charge)?,
                )?;
            }
        }
        Value::StoredAggregate(snapshot) => {
            charge(fuel(snapshot.image().metadata_inspection_work())?)?;
            cells = add(
                cells,
                add(snapshot.image().len(), snapshot.image().metadata_cells())?,
            )?;
            if let Some(semantic) = snapshot.decoded_semantic() {
                cells = add(
                    cells,
                    value_cells(semantic, depth + 1, maximum_depth, charge)?,
                )?;
            }
        }
        Value::Int(_)
        | Value::Bool(_)
        | Value::Float(_)
        | Value::Enum {
            ..
        }
        | Value::Procedure {
            ..
        } => {}
    }
    Ok(cells)
}

fn frame_cells(
    frame: &Frame<'_>,
    charge: &mut impl FnMut(u64) -> std::result::Result<(), Error>,
) -> std::result::Result<usize, Error> {
    if !matches!(frame.procedure, FrameCode::Owned(_)) {
        return Err(Error::InvalidIr("fork requires owned checked frame code"));
    }
    let mut headers = add(frame.slots.capacity(), frame.loops.capacity())?;
    headers = add(headers, frame.temporaries.capacity())?;
    headers = add(headers, frame.sequence_temp_roots.capacity())?;
    headers = add(headers, frame.push_contexts.capacity())?;
    charge(fuel(add(headers, 1)?)?)?;
    let mut cells = add(headers, 1)?;
    for pointer in frame
        .slots
        .iter()
        .chain(&frame.temporaries)
        .chain(&frame.sequence_temp_roots)
        .chain(frame.push_contexts.values())
    {
        cells = add(cells, pointer.metadata_cells())?;
    }
    add(
        cells,
        option_pointer_cells(frame.procedure_context.as_ref())?,
    )
}

fn bounds<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
    fork: bool,
    charge: &mut impl FnMut(u64) -> std::result::Result<(), Error>,
) -> std::result::Result<Bounds, Error> {
    let files = if fork {
        vm.host_files.closed_fork_bounds(charge)?
    } else {
        vm.host_files.snapshot_bounds(charge)?
    };
    let heap = vm.host_heap.fork_bounds(charge)?;
    let (bindings, binding_work) = vm.expression_bindings.fork_bounds(charge)?;
    // This existing bound intentionally counts the maximum cached allocation
    // shape three times: semantic values, mutable byte images and masks can
    // coexist. It also covers every sparse Memory table and pool/layout ledgers.
    let memory_work = vm.memory.snapshot_work_cost()?;
    let mut cells = add(add(add(memory_work, files)?, heap)?, bindings)?;
    if let Some(process) = &vm.processes {
        cells = add(cells, add(process.branch.retained_metadata_cells(), 1)?)?;
    } else if fork {
        return Err(Error::InvalidIr("fork without a process ledger"));
    }
    let mut outer = add(vm.globals.capacity(), vm.frames.capacity())?;
    outer = add(outer, vm.root_temporaries.capacity())?;
    outer = add(outer, vm.static_objects.capacity())?;
    outer = add(outer, vm.static_publications.capacity())?;
    outer = add(outer, vm.static_data.capacity())?;
    outer = add(outer, vm.literal_backing.capacity())?;
    charge(fuel(add(outer, 1)?)?)?;
    cells = add(cells, add(outer, 1)?)?;
    for pointer in vm
        .globals
        .iter()
        .flatten()
        .chain(&vm.root_temporaries)
        .chain(vm.static_objects.values())
    {
        cells = add(cells, pointer.metadata_cells())?;
    }
    for frame in &vm.frames {
        cells = add(cells, frame_cells(frame, charge)?)?;
    }
    for ((_, value), pointer) in &vm.literal_backing {
        cells = add(
            cells,
            value_cells(value, 0, vm.limits.evaluation_depth, charge)?,
        )?;
        cells = add(cells, pointer.metadata_cells())?;
    }
    cells = add(cells, option_pointer_cells(vm.current_context.as_ref())?)?;
    cells = add(cells, option_pointer_cells(vm.default_context.as_ref())?)?;
    // Bounds count concrete copied records and byte/value shapes, never configured
    // maxima. The second factor covers nested clone bookkeeping conservatively.
    let work = add(
        cells.checked_mul(2).ok_or(Error::Limit(LimitKind::Fuel))?,
        binding_work,
    )?;
    Ok(Bounds {
        cells,
        work,
        memory_work,
        binding_work,
    })
}

pub(super) fn measured<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &mut Vm<'_, P, E>,
    fork: bool,
) -> Result<Bounds> {
    let available = vm.publication_remaining_fuel();
    let mut spent = 0_u64;
    let result = bounds(vm, fork, &mut |work| {
        spent = spent
            .checked_add(work)
            .filter(|work| *work <= available)
            .ok_or(Error::Limit(LimitKind::Fuel))?;
        Ok(())
    });
    vm.charge_work(usize::try_from(spent).map_err(|_| Error::Limit(LimitKind::Fuel))?)?;
    Ok(result?)
}
pub(super) fn shared_world_cells<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
) -> Result<usize> {
    let Some(process) = vm.processes.as_ref() else {
        return Ok(0);
    };
    Ok(process
        .cells()
        .checked_sub(add(process.branch.retained_metadata_cells(), 1)?)
        .ok_or(Error::InvalidIr("process shared-world accounting mismatch"))?)
}
fn admit<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
    vm: &Vm<'_, P, E>,
    live: usize,
    parked: usize,
    other_retained: usize,
    scratch: usize,
) -> Result<()> {
    let total = add(
        add(
            add(add(live, parked)?, other_retained)?,
            shared_world_cells(vm)?,
        )?,
        add(scratch, publication_origin_cells(vm))?,
    )?;
    if total > vm.limits.value_cells {
        Err(Error::Limit(LimitKind::ValueCells).into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;

impl BranchSnapshot {
    /// A single gate combines live storage, its prospective copy, both machine
    /// owners, queued branches and the candidate world before any deep clone.
    pub(super) fn admit_fork<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        vm: &mut Vm<'_, P, E>,
        other_retained: usize,
    ) -> Result<Bounds> {
        if vm.effects.has_journal() {
            return Err(Error::InvalidIr("finish the process leaf before private fork").into());
        }
        let bounds = measured(vm, true)?;
        admit(
            vm,
            bounds.cells,
            bounds.cells,
            other_retained,
            vm.frames.len().saturating_mul(2),
        )?;
        Ok(bounds)
    }
    /// `other_retained` includes both machine footprints, queued branches and
    /// the one shared rollback/effect/source metadata owner. IPC is counted here once.
    #[cfg(test)]
    pub(super) fn fork_from_live<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        vm: &mut Vm<'_, P, E>,
        other_retained: usize,
    ) -> Result<Self> {
        let bounds = Self::admit_fork(vm, other_retained)?;
        vm.charge_work(bounds.work)?;
        Self::clone_admitted(vm, bounds)
    }

    pub(super) fn clone_admitted<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        vm: &mut Vm<'_, P, E>,
        bounds: Bounds,
    ) -> Result<Self> {
        let frames = vm
            .frames
            .iter()
            .map(|frame| {
                let FrameCode::Owned(procedure) = &frame.procedure else {
                    unreachable!("owned frames preflighted")
                };
                OwnedFrame {
                    procedure: Arc::clone(procedure),
                    slots: frame.slots.clone(),
                    loops: frame.loops.clone(),
                    temporaries: frame.temporaries.clone(),
                    sequence_temp_bytes: frame.sequence_temp_bytes,
                    sequence_temp_roots: frame.sequence_temp_roots.clone(),
                    procedure_context: frame.procedure_context.clone(),
                    push_contexts: frame.push_contexts.clone(),
                }
            })
            .collect();
        Ok(Self {
            memory: vm.memory.fork_private_branch(bounds.memory_work)?,
            host_files: vm.host_files.clone(),
            host_heap: vm.host_heap.clone(),
            branch: vm
                .processes
                .as_ref()
                .expect("process preflighted")
                .branch
                .clone(),
            frames,
            expression_bindings: vm.expression_bindings.fork_private(bounds.binding_work)?,
            globals: vm.globals.clone(),
            static_objects: vm.static_objects.clone(),
            static_publications: vm.static_publications.clone(),
            static_data: vm.static_data.clone(),
            literal_backing: vm.literal_backing.clone(),
            current_context: vm.current_context.clone(),
            default_context: vm.default_context.clone(),
            root_temporaries: vm.root_temporaries.clone(),
            root_sequence_temp_bytes: vm.root_sequence_temp_bytes,
            cells: bounds.cells,
        })
    }
    pub(super) fn cells(&self) -> usize {
        self.cells
    }
    pub(super) fn process_branch(&self) -> &ProcessBranchState {
        &self.branch
    }

    /// Scheduler supplies the already-metered branch minted from the actual
    /// candidate-world ForkPair. This setter grants no source process authority.
    pub(super) fn rebind_child(
        &mut self,
        branch: ProcessBranchState,
    ) -> std::result::Result<(), Error> {
        let cells = self
            .cells
            .checked_sub(self.branch.retained_metadata_cells())
            .and_then(|cells| cells.checked_add(branch.retained_metadata_cells()))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.branch = branch;
        self.cells = cells;
        Ok(())
    }

    /// Move two isolated owners. Shared effects, world, statistics and provider
    /// remain resident. Only outer frame wrappers need fresh, admitted vectors.
    pub(super) fn swap_live<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &mut self,
        vm: &mut Vm<'_, P, E>,
        other_retained: usize,
    ) -> Result<()> {
        let live = measured(vm, false)?;
        let scratch = add(vm.frames.len(), self.frames.len())?;
        admit(vm, live.cells, self.cells, other_retained, scratch)?;
        vm.charge_work(scratch)?;
        let mut parked_frames = Vec::with_capacity(vm.frames.len());
        let mut live_frames = Vec::with_capacity(self.frames.len());
        // This is the only fallible operation after allocation and before moving
        // any owner. Its cache update is checked before the branch is exchanged.
        vm.processes
            .as_mut()
            .expect("process preflighted")
            .swap_branch(&mut self.branch)?;
        for frame in std::mem::take(&mut vm.frames) {
            let FrameCode::Owned(procedure) = frame.procedure else {
                unreachable!("owned frames preflighted")
            };
            parked_frames.push(OwnedFrame {
                procedure,
                slots: frame.slots,
                loops: frame.loops,
                temporaries: frame.temporaries,
                sequence_temp_bytes: frame.sequence_temp_bytes,
                sequence_temp_roots: frame.sequence_temp_roots,
                procedure_context: frame.procedure_context,
                push_contexts: frame.push_contexts,
            });
        }
        for frame in std::mem::take(&mut self.frames) {
            live_frames.push(Frame {
                procedure: FrameCode::Owned(frame.procedure),
                slots: frame.slots,
                loops: frame.loops,
                temporaries: frame.temporaries,
                sequence_temp_bytes: frame.sequence_temp_bytes,
                sequence_temp_roots: frame.sequence_temp_roots,
                procedure_context: frame.procedure_context,
                push_contexts: frame.push_contexts,
            });
        }
        self.frames = parked_frames;
        vm.frames = live_frames;
        std::mem::swap(&mut self.memory, &mut vm.memory);
        std::mem::swap(&mut self.host_files, &mut vm.host_files);
        std::mem::swap(&mut self.host_heap, &mut vm.host_heap);
        std::mem::swap(&mut self.expression_bindings, &mut vm.expression_bindings);
        std::mem::swap(&mut self.globals, &mut vm.globals);
        std::mem::swap(&mut self.static_objects, &mut vm.static_objects);
        std::mem::swap(&mut self.static_publications, &mut vm.static_publications);
        std::mem::swap(&mut self.static_data, &mut vm.static_data);
        std::mem::swap(&mut self.literal_backing, &mut vm.literal_backing);
        std::mem::swap(&mut self.current_context, &mut vm.current_context);
        std::mem::swap(&mut self.default_context, &mut vm.default_context);
        std::mem::swap(&mut self.root_temporaries, &mut vm.root_temporaries);
        std::mem::swap(
            &mut self.root_sequence_temp_bytes,
            &mut vm.root_sequence_temp_bytes,
        );
        self.cells = live.cells;
        Ok(())
    }
}
