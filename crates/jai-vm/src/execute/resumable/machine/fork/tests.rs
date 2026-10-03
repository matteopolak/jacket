use super::*;
use jai_types::{
    CallingConvention, ContextMode, IntegerType, ProcedureType, RecordKind, ScalarType,
    TypeRegistry, Variadic,
};

fn fixture() -> (Library, ProcedureId, TypeId, TypeId, TypeId) {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [word]).unwrap();
    let slice = types.slice(word).unwrap();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(0);
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![Statement::DiscardInt(IntExpr::constant(Integer::wrapping(
                    IntegerType::S64,
                    42,
                )))],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .finish_library()
        .unwrap();
    (library, id, word, record, slice)
}
fn plan(library: &Library, id: ProcedureId) -> Arc<ProcedurePlan> {
    Arc::new(
        compile_checked_procedure(library.checked_procedure(id).unwrap(), Limits::default())
            .unwrap(),
    )
}
fn node(plan: &ProcedurePlan) -> NodeId {
    let StatementCode::Discard(node) = plan.code.blocks[plan.body.index()].statements[0] else {
        panic!("expected checked discard node")
    };
    node
}
fn integer(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}

#[test]
fn insufficient_fork_work_and_finished_result_do_not_clone_shared_plans() {
    let (library, id, _, _, _) = fixture();
    let code = plan(&library, id);
    let mut machine = Machine::expression(code.code.clone());
    machine.plans.insert(id, code.clone());
    let root_count = Arc::strong_count(&code.code);
    let procedure_count = Arc::strong_count(&code);
    let work = machine.fork_work_cost().unwrap();
    assert!(matches!(
        machine.fork_private(work - 1),
        Err(Error::Limit(LimitKind::Fuel))
    ));
    assert_eq!(Arc::strong_count(&code.code), root_count);
    assert_eq!(Arc::strong_count(&code), procedure_count);
    machine.result = Some(vec![Value::String(vec![0; 32])]);
    assert_eq!(
        machine.fork_work_cost(),
        Err(Error::InvalidIr(
            "cannot fork a completed continuation result"
        ))
    );
    assert!(matches!(
        machine.fork_private(usize::MAX),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(Arc::strong_count(&code.code), root_count);
    assert_eq!(Arc::strong_count(&code), procedure_count);
}

#[test]
fn sparse_capacities_and_arithmetic_overflow_are_in_the_cached_cost() {
    let (library, id, _, _, _) = fixture();
    let code = plan(&library, id);
    let mut machine = Machine::expression(code.code.clone());
    machine.tasks.reserve(32);
    machine.operands.reserve(64);
    machine.plans.reserve(16);
    let expected = 3 * (machine.retained + machine.plan_cells)
        + machine.plans.capacity()
        + machine.tasks.capacity()
        + machine.operands.capacity()
        + 1;
    assert_eq!(machine.fork_work_cost(), Ok(expected));
    let before = Arc::strong_count(&code.code);
    machine.retained = usize::MAX;
    assert_eq!(machine.fork_work_cost(), Err(Error::Limit(LimitKind::Fuel)));
    assert!(matches!(
        machine.fork_private(usize::MAX),
        Err(Error::Limit(LimitKind::Fuel))
    ));
    assert_eq!(Arc::strong_count(&code.code), before);
}

#[test]
fn owned_operand_and_task_payloads_copy_progress_and_share_only_frozen_code() {
    let (library, id, _, record, _) = fixture();
    let code = plan(&library, id);
    let mut vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let root = vm.memory.allocate(library.types(), record, None).unwrap();
    let projection = vm.memory.field(library.types(), &root, 0).unwrap();
    let address = vm
        .memory
        .pointer_to_integer(
            library.types(),
            &projection,
            IntegerType::U64,
            jai_types::CastMode::Checked,
        )
        .unwrap();
    let mut machine = Machine::expression(code.code.clone());
    machine.plans.insert(id, code.clone());
    machine.initialized = true;
    machine
        .operand(&vm, Operand::Value(Value::String(vec![1, 2, 3])))
        .unwrap();
    machine
        .operand(&vm, Operand::Value(address.clone().into_value()))
        .unwrap();
    machine
        .operand(&vm, Operand::Place(projection.clone()))
        .unwrap();
    machine
        .push(
            &vm,
            Some(code.code.clone()),
            3,
            Action::Block {
                block: code.body,
                pc: 1,
            },
            0,
        )
        .unwrap();
    let values = vec![Operand::Value(Value::String(vec![4, 5, 6]))];
    let cells = operand_cells(&values[0], vm.limits.value_cells).unwrap();
    machine
        .push(
            &vm,
            Some(code.code.clone()),
            4,
            Action::Collect {
                nodes: vec![node(&code)],
                index: 1,
                values,
                cells,
                goal: Goal::Apply {
                    node: node(&code),
                    place: false,
                },
            },
            1 + cells,
        )
        .unwrap();
    let retained = machine.retained_cells();
    let mut branch = machine
        .fork_private(machine.fork_work_cost().unwrap())
        .unwrap();
    assert!(branch.retained_cells() <= retained);
    assert_eq!(branch.accounted_cells(), machine.accounted_cells());
    assert_eq!(branch.tasks.len(), machine.tasks.len());
    assert_eq!(branch.operands.len(), machine.operands.len());
    assert!(branch.initialized);
    assert!(Arc::ptr_eq(&branch.plans[&id], &machine.plans[&id]));
    for (left, right) in branch.tasks.iter().zip(&machine.tasks) {
        assert_eq!((left.depth, left.cells), (right.depth, right.cells));
        assert!(Arc::ptr_eq(
            left.code.as_ref().unwrap(),
            right.code.as_ref().unwrap()
        ));
    }
    let Action::Block { block, pc } = branch.tasks[0].action else {
        panic!("lost block progress")
    };
    assert_eq!(block, code.body);
    assert_eq!(pc, 1);
    let Action::Collect { index, values, .. } = &mut branch.tasks[1].action else {
        panic!("lost collection progress")
    };
    assert_eq!(*index, 1);
    let Operand::Value(Value::String(bytes)) = &mut values[0] else {
        panic!("lost owned task value")
    };
    bytes[0] = 99;
    let Action::Collect { values, .. } = &machine.tasks[1].action else {
        unreachable!()
    };
    assert!(matches!(&values[0], Operand::Value(Value::String(bytes)) if bytes == &[4, 5, 6]));
    let Operand::Value(Value::String(bytes)) = &mut branch.operands[0] else {
        panic!("lost owned operand")
    };
    bytes[0] = 88;
    assert!(
        matches!(&machine.operands[0], Operand::Value(Value::String(bytes)) if bytes == &[1, 2, 3])
    );
    assert!(
        matches!(&branch.operands[1], Operand::Value(Value::AddressInteger(number)) if number == &address)
    );
    assert!(
        matches!(&branch.operands[2], Operand::Place(pointer) if pointer == &projection && pointer.metadata_cells() == 1)
    );
}

#[test]
fn simd_buffers_are_owned_independently_at_the_same_instruction_index() {
    let (library, id, _, _, _) = fixture();
    let code = plan(&library, id);
    let vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let mut machine = Machine::expression(code.code.clone());
    machine
        .push(
            &vm,
            Some(code.code.clone()),
            2,
            Action::Simd {
                block: code.body,
                pc: 0,
                index: 2,
                registers: vec![Some(vec![7; 16]), None],
                reserved: 18,
            },
            18,
        )
        .unwrap();
    let mut branch = machine
        .fork_private(machine.fork_work_cost().unwrap())
        .unwrap();
    let Action::Simd {
        index,
        registers,
        reserved,
        ..
    } = &mut branch.tasks[0].action
    else {
        panic!("lost SIMD state")
    };
    assert_eq!((*index, *reserved), (2, 18));
    registers[0].as_mut().unwrap()[0] = 99;
    let Action::Simd {
        index, registers, ..
    } = &machine.tasks[0].action
    else {
        unreachable!()
    };
    assert_eq!(*index, 2);
    assert_eq!(registers[0].as_ref().unwrap(), &vec![7; 16]);
}

#[test]
fn float_bits_procedure_identity_and_opaque_storage_survive_the_private_copy() {
    let (library, id, _, record, _) = fixture();
    let code = plan(&library, id);
    let vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let semantic = Value::Record {
        ty: record,
        fields: vec![integer(7)],
    };
    let image = crate::ByteImage::encode(
        library.types(),
        vm.memory.target(),
        record,
        &semantic,
        vm.limits.value_cells,
    )
    .unwrap();
    let mut layouts = jai_types::LayoutEngine::new(library.types(), vm.memory.target().policy);
    let layout = layouts.layout(record).unwrap();
    let carrier = crate::StoredAggregate::opaque(
        library.types(),
        record,
        image,
        layout,
        vm.limits.value_cells,
    )
    .unwrap();
    let float = Value::Float(jai_types::FloatValue::F64(0x7ff8_0000_0000_0123));
    let procedure = Value::Procedure {
        signature: library.signature(id).unwrap(),
        procedure: Some(id),
    };
    let mut machine = Machine::expression(code.code.clone());
    for value in [
        float.clone(),
        procedure.clone(),
        Value::StoredAggregate(carrier),
    ] {
        machine.operand(&vm, Operand::Value(value)).unwrap();
    }
    let mut branch = machine
        .fork_private(machine.fork_work_cost().unwrap())
        .unwrap();
    assert!(matches!(&branch.operands[0], Operand::Value(value) if value == &float));
    assert!(matches!(&branch.operands[1], Operand::Value(value) if value == &procedure));
    let Operand::Value(Value::StoredAggregate(source)) = &machine.operands[2] else {
        panic!("lost source storage carrier")
    };
    let Operand::Value(Value::StoredAggregate(copied)) = &branch.operands[2] else {
        panic!("lost copied storage carrier")
    };
    assert!(copied.decoded_semantic().is_none());
    assert_eq!(copied, source);
    assert!(std::ptr::eq(copied.image(), source.image()));
    let changed = copied
        .with_field(library.types(), 0, &integer(99), vm.limits.value_cells)
        .unwrap();
    branch.operands[2] = Operand::Value(changed);
    assert_eq!(
        source
            .field(library.types(), 0, vm.limits.value_cells)
            .unwrap(),
        integer(7)
    );
    let changed_field = match &branch.operands[2] {
        Operand::Value(Value::StoredAggregate(value)) => value
            .field(library.types(), 0, vm.limits.value_cells)
            .unwrap(),
        Operand::Value(Value::Record { fields, .. }) => fields[0].clone(),
        _ => panic!("lost updated record value"),
    };
    assert_eq!(changed_field, integer(99));
}

#[test]
fn binding_scope_tokens_remain_valid_in_the_matching_private_environment_clone() {
    let (library, id, _, _, _) = fixture();
    let code = plan(&library, id);
    let vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let mut environment = bindings::BindingEnvironment::new();
    let scope = environment.begin();
    let capture = jai_ir::ExpressionBindingId::new(id, 0);
    environment
        .insert(capture, Value::String(vec![1, 2, 3]), 10)
        .unwrap();
    let mut machine = Machine::expression(code.code.clone());
    machine
        .push(
            &vm,
            Some(code.code.clone()),
            1,
            Action::BindNext {
                node: node(&code),
                index: 1,
                scope,
            },
            0,
        )
        .unwrap();
    machine
        .push(&vm, Some(code.code.clone()), 1, Action::BindEnd(scope), 0)
        .unwrap();
    let branch = machine
        .fork_private(machine.fork_work_cost().unwrap())
        .unwrap();
    let mut branch_environment = environment.clone();
    let Action::BindNext {
        index,
        scope: copied,
        ..
    } = branch.tasks[0].action
    else {
        panic!("lost binding progress")
    };
    assert_eq!(index, 1);
    assert_eq!(copied, scope);
    let Action::BindEnd(copied) = branch.tasks[1].action else {
        panic!("lost binding close")
    };
    branch_environment.end(copied).unwrap();
    assert_eq!(branch_environment.cells(), 0);
    assert_eq!(
        environment.lookup(capture),
        Ok(&Value::String(vec![1, 2, 3]))
    );
    environment.end(scope).unwrap();
}

#[test]
fn pack_snapshots_and_source_mutations_stay_in_their_own_machine_branch() {
    let (library, id, word, _, slice) = fixture();
    let code = plan(&library, id);
    let mut vm = Vm::new(&library, crate::NoEffects, Limits::default()).unwrap();
    let source = vm
        .memory
        .allocate(library.types(), word, Some(integer(7)))
        .unwrap();
    let mut pack = crate::execute::resumable::pack::PackState::new(&mut vm, slice).unwrap();
    pack.capture(
        &mut vm,
        PackPartMode::ElementPlace,
        Operand::Place(source.clone()),
        0,
    )
    .unwrap();
    let payload = pack.cells();
    let mut machine = Machine::expression(code.code.clone());
    machine
        .push(
            &vm,
            Some(code.code.clone()),
            2,
            Action::PackNext {
                node: node(&code),
                index: 1,
                pack,
            },
            payload,
        )
        .unwrap();
    let mut branch = machine
        .fork_private(machine.fork_work_cost().unwrap())
        .unwrap();
    let Action::PackNext { index, pack, .. } = &mut machine.tasks[0].action else {
        panic!("lost pack progress")
    };
    assert_eq!(*index, 1);
    vm.memory
        .store(library.types(), &source, integer(99))
        .unwrap();
    pack.capture(
        &mut vm,
        PackPartMode::ElementPlace,
        Operand::Place(source),
        0,
    )
    .unwrap();
    let Action::PackNext { index, pack, .. } = branch.tasks.pop().unwrap().action else {
        panic!("lost forked pack")
    };
    assert_eq!(index, 1);
    let Value::Slice { pointer, count, .. } = pack.finish(&mut vm).unwrap() else {
        panic!("expected snapshot slice")
    };
    assert_eq!(count, 1);
    assert_eq!(
        vm.memory.load(library.types(), &pointer).unwrap(),
        integer(7)
    );
    let Action::PackNext { pack, .. } = machine.tasks.pop().unwrap().action else {
        unreachable!()
    };
    let Value::Slice { pointer, count, .. } = pack.finish(&mut vm).unwrap() else {
        panic!("expected source slice")
    };
    assert_eq!(count, 2);
    assert_eq!(
        vm.memory.load(library.types(), &pointer).unwrap(),
        integer(7)
    );
    let next = vm.memory.offset(library.types(), &pointer, 1).unwrap();
    assert_eq!(vm.memory.load(library.types(), &next).unwrap(), integer(99));
}
