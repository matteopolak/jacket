use super::*;
use jai_types::{
    CallingConvention, ContextMode, IntegerType, ProcedureType, RecordLayout, ScalarType,
    TypeRegistry, Variadic,
};
use std::{cell::Cell, collections::HashMap};

struct Provider {
    types: TypeRegistry,
    signatures: HashMap<ProcedureId, TypeId>,
    procedures: Vec<Procedure>,
    globals: Vec<Global>,
    places: Places,
    pending: Cell<bool>,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn globals(&self) -> &[Global] {
        &self.globals
    }
    fn places(&self) -> Option<&Places> {
        Some(&self.places)
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if id.index() == 1 && self.pending.get() {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        let Some(procedure) = self.procedures.iter().find(|p| p.id == id) else {
            return ProcedureAvailability::Missing;
        };
        match verify_procedure(
            &self.types,
            procedure,
            &self.signatures,
            &self.globals,
            &self.places,
        ) {
            Ok(checked) => ProcedureAvailability::Ready(checked),
            Err(error) => ProcedureAvailability::Failed(Error::IrValidation(error.to_string())),
        }
    }
}
fn provider() -> Provider {
    Provider {
        types: TypeRegistry::new(),
        signatures: HashMap::new(),
        procedures: vec![],
        globals: vec![],
        places: Places::default(),
        pending: Cell::new(false),
    }
}
fn integer(number: i128) -> IntExpr {
    IntExpr::constant(Integer::wrapping(IntegerType::S64, number))
}
fn write(field: FieldId, number: i128) -> (Box<[FieldId]>, ValueExpr) {
    (vec![field].into(), ValueExpr::Int(integer(number)))
}
fn result(execution: Execution) -> crate::StoredAggregate {
    let Outcome::Complete(mut values) = execution.outcome else {
        panic!("{execution:?}")
    };
    let Value::StoredAggregate(snapshot) = values.remove(0) else {
        panic!("expected physical storage")
    };
    snapshot
}
fn read(snapshot: &crate::StoredAggregate, types: &dyn TypeView, index: usize) -> i128 {
    snapshot
        .field(types, index, 4096)
        .unwrap()
        .integer()
        .unwrap()
        .value()
}
fn overlapping(p: &mut Provider) -> TypeId {
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types
        .define_record_with_placements(
            owner,
            [word, word],
            RecordLayout::default(),
            [None, Some(0)],
        )
        .unwrap();
    owner
}
#[test]
fn ordinary_and_resumable_preserve_repeated_alias_write_order() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let a = p.types.field(owner, 0).unwrap().id;
    let b = p.types.field(owner, 1).unwrap().id;
    let expression = ValueExpr::OrderedRecord {
        ty: owner,
        backing: OrderedRecordBacking::Zeroed,
        initializers: vec![write(a, 1), write(b, 2), write(a, 42)],
    };
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    let snapshot = result(vm.evaluate(&expression));
    assert_eq!(
        (read(&snapshot, &p.types, 0), read(&snapshot, &p.types, 1)),
        (42, 42)
    );
    assert!(snapshot.image().fully_initialized());
    assert_eq!(
        vm.start_resumable_expression(&expression).outcome,
        ResumableOutcome::AwaitingPublication
    );
    let snapshot = result(vm.finish_resumable_validated(|_, _| Ok(())));
    assert_eq!(
        (read(&snapshot, &p.types, 0), read(&snapshot, &p.types, 1)),
        (42, 42)
    );
}
#[test]
fn nested_write_retains_unknown_sibling_without_parent_read() {
    let mut p = provider();
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let child = p.types.reserve_record(RecordKind::Struct);
    p.types.define_record(child, [word, word]).unwrap();
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types.define_record(owner, [child]).unwrap();
    let root = p.types.field(owner, 0).unwrap().id;
    let leaf = p.types.field(child, 1).unwrap().id;
    let expression = ValueExpr::OrderedRecord {
        ty: owner,
        backing: OrderedRecordBacking::Uninitialized,
        initializers: vec![(vec![root, leaf].into(), ValueExpr::Int(integer(42)))],
    };
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    let outer = result(vm.evaluate(&expression));
    let Value::StoredAggregate(inner) = outer.field(&p.types, 0, 4096).unwrap() else {
        panic!("expected masked child")
    };
    assert_eq!(read(&inner, &p.types, 1), 42);
    assert_eq!(inner.field(&p.types, 0, 4096), Err(Error::Uninitialized));
    assert!(!outer.image().fully_initialized());
}
#[test]
fn foreign_owner_is_rejected_before_an_initializer_can_run() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let foreign = overlapping(&mut p);
    let field = p.types.field(foreign, 0).unwrap().id;
    let expression = ValueExpr::OrderedRecord {
        ty: owner,
        backing: OrderedRecordBacking::Zeroed,
        initializers: vec![write(field, 42)],
    };
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.evaluate(&expression).outcome,
        Outcome::Failed(Error::Type(jai_types::TypeError::FieldOwner { .. }))
    ));
    assert_eq!(vm.statistics.calls, 0);
}
#[test]
fn owner_failure_during_preflight_leaves_real_memory_unchanged() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let foreign = overlapping(&mut p);
    let field = p.types.field(foreign, 0).unwrap().id;
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = vm
        .memory
        .allocate(
            &p.types,
            word,
            Some(Value::Int(Integer::wrapping(IntegerType::S64, 7))),
        )
        .unwrap();
    assert!(vm.prepare_ordered_record(owner, [&[field][..]]).is_err());
    assert_eq!(
        vm.memory
            .load(&p.types, &pointer)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        7
    );
}
#[test]
fn pending_initializer_preserves_completed_effects_and_exact_image() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = p
        .types
        .procedure(ProcedureType {
            parameters: vec![].into(),
            results: vec![word].into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &p.types,
    );
    let place = IntPlace::try_from_place(global.place(), &p.types).unwrap();
    p.globals.push(global);
    for (index, number) in [(0, 1), (1, 2)] {
        let id = ProcedureId::new(index);
        p.signatures.insert(id, signature);
        p.procedures.push(Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                flow: Flow::Terminates,
                statements: vec![
                    Statement::StoreInt(
                        place,
                        IntExpr::new(
                            IntegerType::S64,
                            IntExprKind::Binary(
                                IntOp::Add,
                                Box::new(IntExpr::load(place)),
                                Box::new(integer(1)),
                            ),
                        ),
                    ),
                    Statement::Exit(Exit {
                        cleanups: vec![],
                        transfer: Transfer::ReturnInt(integer(number)),
                    }),
                ],
            },
        });
    }
    p.pending.set(true);
    let a = p.types.field(owner, 0).unwrap().id;
    let b = p.types.field(owner, 1).unwrap().id;
    let call = |id| ValueExpr::Call {
        ty: word,
        call: Call::new(ProcedureId::new(id), vec![]),
    };
    let expression = ValueExpr::OrderedRecord {
        ty: owner,
        backing: OrderedRecordBacking::Zeroed,
        initializers: vec![
            (vec![a].into(), call(0)),
            (vec![b].into(), call(1)),
            write(a, 42),
        ],
    };
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_expression(&expression).outcome,
        ResumableOutcome::Suspended(vec![Dependency::Procedure(ProcedureId::new(1))])
    );
    let pointer = vm.globals[0].as_ref().unwrap();
    assert_eq!(
        vm.memory
            .load(&p.types, pointer)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        1
    );
    assert!(matches!(
        vm.resume_resumable().outcome,
        ResumableOutcome::Suspended(_)
    ));
    assert_eq!(
        vm.memory
            .load(&p.types, vm.globals[0].as_ref().unwrap())
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        1
    );
    let state = vm.into_continuation().unwrap();
    p.pending.set(false);
    let mut vm = Vm::with_continuation(&p, crate::NoEffects, state).unwrap();
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(
        vm.memory
            .load(&p.types, vm.globals[0].as_ref().unwrap())
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        2
    );
    let snapshot = result(vm.finish_resumable_validated(|_, _| Ok(())));
    assert_eq!(
        (read(&snapshot, &p.types, 0), read(&snapshot, &p.types, 1)),
        (42, 42)
    );
}
#[test]
fn backing_admission_counts_mask_and_pending_write_metadata() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let a = p.types.field(owner, 0).unwrap().id;
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    vm.prepare_ordered_record(owner, [&[a][..]]).unwrap();
    let state = vm
        .start_ordered_record(owner, OrderedRecordBacking::Uninitialized, [&[a][..]], 0)
        .unwrap();
    assert_eq!(state.cells(4096).unwrap(), 23);
    assert!(matches!(
        state.cells(22),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
}

#[test]
fn finalizer_normalizes_real_pointer_receipts_and_whole_copy_keeps_holes() {
    let mut p = provider();
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer_ty = p.types.pointer(word).unwrap();
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types
        .define_record(owner, [pointer_ty, pointer_ty, word])
        .unwrap();
    let a = p.types.field(owner, 0).unwrap().id;
    let b = p.types.field(owner, 1).unwrap().id;
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    let first = vm
        .memory
        .allocate(
            &p.types,
            word,
            Some(Value::Int(Integer::wrapping(IntegerType::S64, 1))),
        )
        .unwrap();
    let second = vm
        .memory
        .allocate(
            &p.types,
            word,
            Some(Value::Int(Integer::wrapping(IntegerType::S64, 42))),
        )
        .unwrap();
    vm.prepare_ordered_record(owner, [&[a][..], &[b][..], &[a][..]])
        .unwrap();
    let mut state = vm
        .start_ordered_record(
            owner,
            OrderedRecordBacking::Uninitialized,
            [&[a][..], &[b][..], &[a][..]],
            0,
        )
        .unwrap();
    vm.write_ordered_record(&mut state, 0, &Value::Pointer(first), 0)
        .unwrap();
    vm.write_ordered_record(&mut state, 1, &Value::Pointer(second.clone()), 0)
        .unwrap();
    vm.write_ordered_record(&mut state, 2, &Value::Pointer(second.clone()), 0)
        .unwrap();
    let value = vm.finish_ordered_record(state, 0).unwrap();
    let Value::StoredAggregate(snapshot) = &value else {
        panic!("expected storage")
    };
    let token = vm
        .memory
        .pointer_to_integer(
            &p.types,
            &second,
            IntegerType::U64,
            jai_types::CastMode::Checked,
        )
        .unwrap();
    let bits = token.integer().value() as u64;
    assert_eq!(&snapshot.image().bytes()[0..8], &bits.to_le_bytes());
    assert_eq!(&snapshot.image().bytes()[8..16], &bits.to_le_bytes());
    let destination = vm.memory.allocate(&p.types, owner, Some(value)).unwrap();
    let Value::StoredAggregate(copied) = vm.memory.load(&p.types, &destination).unwrap() else {
        panic!("expected copied storage")
    };
    assert_eq!(
        copied.field(&p.types, 0, 4096).unwrap(),
        Value::Pointer(second.clone())
    );
    assert_eq!(
        copied.field(&p.types, 1, 4096).unwrap(),
        Value::Pointer(second)
    );
    assert_eq!(copied.field(&p.types, 2, 4096), Err(Error::Uninitialized));
}
#[test]
fn rejected_transient_patch_does_not_replace_completed_image() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let a = p.types.field(owner, 0).unwrap().id;
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    vm.prepare_ordered_record(owner, [&[a][..], &[a][..]])
        .unwrap();
    let mut state = vm
        .start_ordered_record(
            owner,
            OrderedRecordBacking::Uninitialized,
            [&[a][..], &[a][..]],
            0,
        )
        .unwrap();
    vm.write_ordered_record(
        &mut state,
        0,
        &Value::Int(Integer::wrapping(IntegerType::S64, 7)),
        0,
    )
    .unwrap();
    let previous = state.image.clone();
    vm.limits.value_cells = state.cells(4096).unwrap() + vm.memory.value_cells();
    assert!(matches!(
        vm.write_ordered_record(
            &mut state,
            1,
            &Value::Int(Integer::wrapping(IntegerType::S64, 42)),
            0
        ),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert_eq!(state.image, previous);
}

#[test]
fn empty_journal_distinguishes_zeroed_from_genuinely_skipped_backing() {
    let mut p = provider();
    let owner = overlapping(&mut p);
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    for backing in [
        OrderedRecordBacking::Zeroed,
        OrderedRecordBacking::Uninitialized,
    ] {
        let expression = ValueExpr::OrderedRecord {
            ty: owner,
            backing,
            initializers: vec![],
        };
        let snapshot = result(vm.evaluate(&expression));
        if backing == OrderedRecordBacking::Zeroed {
            assert_eq!(read(&snapshot, &p.types, 0), 0);
            assert!(snapshot.image().fully_initialized());
        } else {
            assert_eq!(snapshot.field(&p.types, 0, 4096), Err(Error::Uninitialized));
            assert!(!snapshot.image().fully_initialized());
        }
    }
}
#[test]
fn complete_carrier_whole_copies_retain_actual_initialized_padding() {
    let mut p = provider();
    let byte = p.types.scalar(ScalarType::Int(IntegerType::U8));
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types.define_record(owner, [byte, word]).unwrap();
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    vm.prepare_layout(owner).unwrap();
    let layout = vm.memory.prepared_layout(&p.types, owner).unwrap();
    assert_eq!(layout.field_offsets.as_ref(), [0, 8]);
    // This is actual fully known byte storage, including seven nonzero padding
    // bytes. No padding is synthesized from a semantic aggregate value.
    let mut bytes = vec![0xaa; 16];
    bytes[0] = 7;
    bytes[8..16].copy_from_slice(&42u64.to_le_bytes());
    let image = ByteImage::from_bytes(vm.memory.target(), bytes.clone(), 4096).unwrap();
    let value = vm
        .memory
        .finish_ordered_record(&p.types, owner, layout, image)
        .unwrap();
    let first = vm.memory.allocate(&p.types, owner, Some(value)).unwrap();
    let copied = vm.memory.load(&p.types, &first).unwrap();
    let second = vm.memory.allocate(&p.types, owner, None).unwrap();
    vm.memory.store(&p.types, &second, copied).unwrap();
    let Value::StoredAggregate(snapshot) = vm.memory.load(&p.types, &second).unwrap() else {
        panic!("expected exact carrier")
    };
    assert_eq!(snapshot.image().bytes(), bytes);
    assert_eq!(read(&snapshot, &p.types, 1), 42);
}

#[test]
fn scalar_overlap_precharges_cloned_existing_pointer_provenance() {
    let mut p = provider();
    let byte = p.types.scalar(ScalarType::Int(IntegerType::U8));
    let address = p.types.pointer(byte).unwrap();
    let prefix = p.types.fixed_array(byte, 3).unwrap();
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types
        .define_record_with_placements(
            owner,
            [address, prefix, byte],
            RecordLayout::default(),
            [None, Some(0), None],
        )
        .unwrap();
    let mut nested = byte;
    let mut containers = vec![];
    for _ in 0..32 {
        let record = p.types.reserve_record(RecordKind::Struct);
        p.types.define_record(record, [nested]).unwrap();
        containers.push(record);
        nested = record;
    }
    let pointer_field = p.types.field(owner, 0).unwrap().id;
    let byte_field = p.types.field(owner, 2).unwrap().id;
    let paths = [&[pointer_field][..], &[byte_field][..]];
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    vm.prepare_layout(nested).unwrap();
    let mut pointer = vm.memory.allocate(&p.types, nested, None).unwrap();
    for _ in containers.iter().rev() {
        pointer = vm.memory.field(&p.types, &pointer, 0).unwrap();
    }
    vm.prepare_ordered_record(owner, paths).unwrap();
    assert_eq!(
        vm.memory
            .prepared_layout(&p.types, owner)
            .unwrap()
            .field_offsets
            .as_ref(),
        [0, 0, 3]
    );
    let mut state = vm
        .start_ordered_record(owner, OrderedRecordBacking::Uninitialized, paths, 0)
        .unwrap();
    vm.write_ordered_record(&mut state, 0, &Value::Pointer(pointer), 0)
        .unwrap();
    let old_metadata = state.image.metadata_cells();
    assert!(old_metadata > 32);
    let old_image = state.image.clone();
    let rhs = Value::Int(Integer::wrapping(IntegerType::U8, 7));
    // The old formula admits the scalar RHS and two-byte patch, but omits
    // cloning an existing projected Pointer when the address span splits.
    vm.limits.value_cells =
        state.cells(4096).unwrap() + vm.memory.value_cells() + rhs.cells(4096).unwrap() * 3 + 2;
    assert!(matches!(
        vm.write_ordered_record(&mut state, 1, &rhs, 0),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert_eq!(state.image, old_image);
}

#[test]
fn partial_pointer_overwrite_keeps_actual_owning_memory_bytes() {
    let mut p = provider();
    let byte = p.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_ty = p.types.pointer(byte).unwrap();
    let prefix = p.types.fixed_array(byte, 3).unwrap();
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types
        .define_record_with_placements(
            owner,
            [pointer_ty, prefix, byte],
            RecordLayout::default(),
            [None, Some(0), None],
        )
        .unwrap();
    let pointer_field = p.types.field(owner, 0).unwrap().id;
    let byte_field = p.types.field(owner, 2).unwrap().id;
    let paths = [&[pointer_field][..], &[byte_field][..]];
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    let target = vm.memory.allocate(&p.types, byte, None).unwrap();
    let oracle = vm.memory.allocate(&p.types, owner, None).unwrap();
    let oracle_pointer = vm.memory.field(&p.types, &oracle, 0).unwrap();
    let oracle_byte = vm.memory.field(&p.types, &oracle, 2).unwrap();
    let pointer_value = Value::Pointer(target);
    let byte_value = Value::Int(Integer::wrapping(IntegerType::U8, 7));
    vm.memory
        .store(&p.types, &oracle_pointer, pointer_value.clone())
        .unwrap();
    vm.memory
        .store(&p.types, &oracle_byte, byte_value.clone())
        .unwrap();
    let Value::StoredAggregate(expected) = vm.memory.load(&p.types, &oracle).unwrap() else {
        panic!("expected actual oracle storage")
    };
    vm.prepare_ordered_record(owner, paths).unwrap();
    let mut state = vm
        .start_ordered_record(owner, OrderedRecordBacking::Uninitialized, paths, 0)
        .unwrap();
    vm.write_ordered_record(&mut state, 0, &pointer_value, 0)
        .unwrap();
    vm.write_ordered_record(&mut state, 1, &byte_value, 0)
        .unwrap();
    let Value::StoredAggregate(actual) = vm.finish_ordered_record(state, 0).unwrap() else {
        panic!("expected physical recipe storage")
    };
    assert_eq!(actual.image().bytes(), expected.image().bytes());
    assert_eq!(actual.image(), expected.image());
    assert_eq!(
        actual
            .field(&p.types, 2, 4096)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        7
    );
    assert!(actual.field(&p.types, 0, 4096).is_err());
}
#[test]
fn foreign_memory_pointer_patch_fails_before_completed_image_mutation() {
    let mut p = provider();
    let word = p.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer_ty = p.types.pointer(word).unwrap();
    let owner = p.types.reserve_record(RecordKind::Struct);
    p.types.define_record(owner, [pointer_ty]).unwrap();
    let field = p.types.field(owner, 0).unwrap().id;
    let mut vm = Vm::new(&p, crate::NoEffects, Limits::default()).unwrap();
    let mut foreign = Memory::new(Limits::default());
    let pointer = foreign.allocate(&p.types, word, None).unwrap();
    vm.prepare_ordered_record(owner, [&[field][..]]).unwrap();
    let mut state = vm
        .start_ordered_record(owner, OrderedRecordBacking::Zeroed, [&[field][..]], 0)
        .unwrap();
    let old_image = state.image.clone();
    assert!(matches!(
        vm.write_ordered_record(&mut state, 0, &Value::Pointer(pointer), 0),
        Err(Halt::Failed(Error::ForeignPointer))
    ));
    assert_eq!(state.image, old_image);
}
