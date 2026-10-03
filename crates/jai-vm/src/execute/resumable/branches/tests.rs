use super::*;
use crate::NoEffects;
use jai_types::{CallingConvention, ContextMode, IntegerType, ScalarType, TypeRegistry, Variadic};

struct Provider(TypeRegistry);
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.0
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
fn integer(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}
fn vm(provider: &Provider) -> Vm<'_, Provider, NoEffects> {
    let mut vm = Vm::new(provider, NoEffects, Limits::default()).unwrap();
    vm.processes = Some(process::ProcessState::new(vm.limits).unwrap());
    vm
}
fn procedure(types: &mut TypeRegistry) -> Arc<Procedure> {
    let signature = types
        .procedure(ProcedureType {
            parameters: [].into(),
            results: [].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    Arc::new(Procedure {
        id: ProcedureId::new(0),
        signature,
        parameters: vec![],
        locals: vec![],
        body: Block {
            statements: vec![],
            flow: Flow::FallsThrough,
        },
        cleanups: vec![],
    })
}

#[test]
fn branch_swap_restores_memory_frames_captures_context_and_temporary_owners() {
    let mut provider = Provider(TypeRegistry::new());
    let code = procedure(&mut provider.0);
    let word = provider.0.scalar(ScalarType::Int(IntegerType::S64));
    let mut vm = vm(&provider);
    let pointer = vm
        .memory
        .allocate(&provider.0, word, Some(integer(17)))
        .unwrap();
    let binding = ExpressionBindingId::new(code.id, 0);
    let scope = vm.expression_bindings.begin();
    vm.expression_bindings
        .insert(binding, integer(23), 100)
        .unwrap();
    vm.globals.push(Some(pointer.clone()));
    vm.default_context = Some(pointer.clone());
    vm.current_context = Some(pointer.clone());
    vm.root_temporaries.push(pointer.clone());
    vm.root_sequence_temp_bytes = 11;
    vm.literal_backing
        .insert((word, integer(17)), pointer.clone());
    vm.frames.push(Frame {
        procedure: FrameCode::Owned(Arc::clone(&code)),
        slots: vec![pointer.clone()],
        loops: vec![],
        temporaries: vec![pointer.clone()],
        sequence_temp_bytes: 5,
        sequence_temp_roots: vec![pointer.clone()],
        procedure_context: Some(pointer.clone()),
        push_contexts: HashMap::new(),
    });
    let mut child = BranchSnapshot::fork_from_live(&mut vm, 0).unwrap();
    assert!(child.cells() > vm.memory.value_cells());
    assert!(Arc::ptr_eq(&child.frames[0].procedure, &code));
    vm.memory.store(&provider.0, &pointer, integer(99)).unwrap();
    vm.expression_bindings.end(scope).unwrap();
    vm.current_context = None;
    vm.root_temporaries.clear();
    vm.root_sequence_temp_bytes = 0;
    vm.literal_backing.clear();
    vm.frames[0].slots.clear();
    vm.frames[0].sequence_temp_bytes = 0;
    let statistics = vm.statistics;
    child.swap_live(&mut vm, 0).unwrap();
    assert_eq!(vm.memory.load(&provider.0, &pointer).unwrap(), integer(17));
    assert_eq!(vm.expression_bindings.lookup(binding), Ok(&integer(23)));
    assert_eq!(vm.current_context, Some(pointer.clone()));
    assert_eq!(vm.root_temporaries, vec![pointer.clone()]);
    assert_eq!(vm.root_sequence_temp_bytes, 11);
    assert_eq!(vm.frames[0].slots, vec![pointer.clone()]);
    assert_eq!(vm.frames[0].sequence_temp_bytes, 5);
    assert!(vm.literal_backing.contains_key(&(word, integer(17))));
    assert!(vm.statistics.steps > statistics.steps);
    vm.expression_bindings.end(scope).unwrap();
    child.swap_live(&mut vm, 0).unwrap();
    assert_eq!(vm.memory.load(&provider.0, &pointer).unwrap(), integer(99));
    assert!(vm.expression_bindings.lookup(binding).is_err());
    assert!(vm.current_context.is_none());
    assert!(vm.frames[0].slots.is_empty());
}

#[test]
fn actual_child_branch_is_rebound_while_world_mutations_remain_shared() {
    let provider = Provider(TypeRegistry::new());
    let mut vm = vm(&provider);
    let mut child = BranchSnapshot::fork_from_live(&mut vm, 0).unwrap();
    let process = vm.processes.as_mut().unwrap();
    let parent = process.branch.current();
    let pair = process.world.fork(parent).unwrap();
    let child_branch = process
        .branch
        .for_child(pair.child, &process.world)
        .unwrap();
    process.world.pipe(pair.child).unwrap();
    process.refresh(vm.limits).unwrap();
    let world_work = process.world.work_cost().unwrap();
    child.rebind_child(child_branch).unwrap();
    assert_eq!(child.process_branch().current(), pair.child);
    child.swap_live(&mut vm, 0).unwrap();
    let process = vm.processes.as_ref().unwrap();
    assert_eq!(process.branch.current(), pair.child);
    assert_eq!(child.process_branch().current(), parent);
    assert_eq!(process.world.parent(pair.child).unwrap(), Some(parent));
    assert_eq!(process.world.work_cost().unwrap(), world_work);
    child.swap_live(&mut vm, 0).unwrap();
    assert_eq!(vm.processes.as_ref().unwrap().branch.current(), parent);
    assert_eq!(
        vm.processes.as_ref().unwrap().world.work_cost().unwrap(),
        world_work
    );
}

#[test]
fn combined_fork_admission_rejects_parked_retention_without_mutation() {
    let provider = Provider(TypeRegistry::new());
    let word = provider.0.scalar(ScalarType::Int(IntegerType::S64));
    let mut vm = vm(&provider);
    let pointer = vm
        .memory
        .allocate(&provider.0, word, Some(integer(17)))
        .unwrap();
    let allocation_count = vm.memory.allocation_count();
    let current = vm.processes.as_ref().unwrap().branch.current();
    let other = vm.limits.value_cells;
    assert!(matches!(
        BranchSnapshot::fork_from_live(&mut vm, other),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert_eq!(vm.memory.allocation_count(), allocation_count);
    assert_eq!(vm.memory.load(&provider.0, &pointer).unwrap(), integer(17));
    assert_eq!(vm.processes.as_ref().unwrap().branch.current(), current);
}

#[test]
fn fork_inspection_exhausts_fuel_before_cloning_private_state() {
    let provider = Provider(TypeRegistry::new());
    let mut vm = vm(&provider);
    vm.globals = vec![None; 100];
    vm.limits.fuel = vm.statistics.steps + 3;
    assert!(matches!(
        BranchSnapshot::fork_from_live(&mut vm, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.globals.len(), 100);
    assert_eq!(vm.memory.allocation_count(), 0);
}

#[test]
fn borrowed_procedure_frame_cannot_escape_into_a_branch_snapshot() {
    let mut provider = Provider(TypeRegistry::new());
    let code = procedure(&mut provider.0);
    let mut vm = vm(&provider);
    vm.frames.push(Frame {
        procedure: FrameCode::Borrowed(&code),
        slots: vec![],
        loops: vec![],
        temporaries: vec![],
        sequence_temp_bytes: 0,
        sequence_temp_roots: vec![],
        procedure_context: None,
        push_contexts: HashMap::new(),
    });
    assert!(matches!(
        BranchSnapshot::fork_from_live(&mut vm, 0),
        Err(Halt::Failed(Error::InvalidIr(
            "fork requires owned checked frame code"
        )))
    ));
    assert_eq!(vm.frames.len(), 1);
}


#[test]
fn borrowed_string_capacity_inspection_does_not_walk_bytes() {
    for length in [0, 152, 1077, 8192] {
        let mut bytes = Vec::with_capacity(length + 31);
        bytes.resize(length, b'7');
        let capacity = bytes.capacity();
        let value = Value::String(bytes);
        let mut spent = 0;
        let cells = value_cells(&value, 0, 256, &mut |work| {
            spent += work;
            Ok(())
        })
        .unwrap();
        assert_eq!(spent, 1);
        assert_eq!(cells, capacity + 1);
    }
}

#[test]
fn nested_literal_inspection_charges_nodes_and_preserves_spare_capacity() {
    let mut provider = Provider(TypeRegistry::new());
    let ty = provider.0.fixed_array(provider.0.string(), 2).unwrap();
    let mut bytes = Vec::with_capacity(8192);
    bytes.resize(1077, b'7');
    let byte_capacity = bytes.capacity();
    let mut elements = Vec::with_capacity(7);
    elements.push(Value::String(bytes));
    elements.push(Value::String(Vec::new()));
    let outer_capacity = elements.capacity();
    let value = Value::Array {
        ty,
        elements,
    };
    let mut spent = 0;
    let cells = value_cells(&value, 0, 256, &mut |work| {
        spent += work;
        Ok(())
    })
    .unwrap();
    assert_eq!(spent, 3);
    assert_eq!(cells, 1 + outer_capacity + byte_capacity);
}

#[test]
fn scheduler_inspects_large_literal_without_debiting_payload_again() {
    let provider = Provider(TypeRegistry::new());
    let mut vm = vm(&provider);
    vm.immutable_backing(provider.0.string(), Value::String(vec![b'7'; 8192]), 0)
        .unwrap();
    let allocation_count = vm.memory.allocation_count();
    let before = vm.statistics.steps;
    vm.limits.fuel = before + 512;
    let bounds = measured(&mut vm, false).unwrap();
    assert!(bounds.cells >= 8192);
    assert!(bounds.work >= 2 * 8192);
    assert!(vm.statistics.steps > before);
    assert!(vm.statistics.steps < before + 512);
    assert_eq!(vm.memory.allocation_count(), allocation_count);
}

#[test]
fn cheap_string_inspection_keeps_copy_fuel_and_retention_gates() {
    let provider = Provider(TypeRegistry::new());
    let mut vm = vm(&provider);
    vm.immutable_backing(provider.0.string(), Value::String(vec![b'7'; 8192]), 0)
        .unwrap();
    let allocations = vm.memory.allocation_count();
    let literal_count = vm.literal_backing.len();
    vm.limits.fuel = vm.statistics.steps + 512;
    assert!(matches!(
        BranchSnapshot::fork_from_live(&mut vm, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(vm.memory.allocation_count(), allocations);
    assert_eq!(vm.literal_backing.len(), literal_count);
    vm.limits.fuel = vm.statistics.steps + 1_000_000;
    vm.limits.value_cells = 8192;
    assert!(matches!(
        BranchSnapshot::fork_from_live(&mut vm, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert_eq!(vm.memory.allocation_count(), allocations);
    assert_eq!(vm.literal_backing.len(), literal_count);
}
