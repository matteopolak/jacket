use super::*;

fn field(base: ValueExpr, field: SequenceField, ty: TypeId) -> ValueExpr {
    ValueExpr::SequenceField {
        base: Box::new(base),
        field,
        ty,
    }
}

fn integer_expression(expression: ValueExpr, ty: IntegerType) -> IntExpr {
    IntExpr::new(ty, IntExprKind::Value(Box::new(expression)))
}

fn string(ty: TypeId, bytes: &[u8]) -> ValueExpr {
    ValueExpr::StringBytes {
        ty,
        bytes: bytes.to_vec(),
    }
}

fn typed_local(f: &Fixture, procedure: usize, index: usize, ty: TypeId) -> Local {
    Local::new_typed(ProcedureId::new(procedure), index, ty, &f.types).unwrap()
}

fn value_procedure(
    f: &mut Fixture,
    index: usize,
    result: TypeId,
    locals: Vec<Local>,
    statements: Vec<Statement>,
) {
    let signature = signature(&mut f.types, vec![], vec![result]);
    f.signatures.insert(ProcedureId::new(index), signature);
    f.procedures.push(Procedure {
        id: ProcedureId::new(index),
        signature,
        parameters: vec![],
        locals,
        body: block(statements),
        cleanups: vec![],
    });
}

fn return_value(expression: ValueExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![expression]),
    })
}

#[test]
fn string_data_keeps_literal_bytes_after_descriptor_reassignment() {
    let mut f = fixture();
    let mut places = PlaceRegistry::new();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let byte_pointer = f.types.pointer(byte).unwrap();
    let s = typed_local(&f, 0, 0, text);
    let p = typed_local(&f, 0, 1, byte_pointer);
    let original_byte = places
        .dereference(ValueExpr::Load(p.place()), &f.types)
        .unwrap();
    let s_count = places
        .sequence_field(s.place(), SequenceField::Count, &f.types)
        .unwrap();
    let s_count = IntPlace::try_from_place(s_count, &f.types).unwrap();
    let first = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Cast(
            jai_types::CastMode::Unchecked,
            Box::new(integer_expression(
                ValueExpr::Load(original_byte),
                IntegerType::U8,
            )),
        ),
    );
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![s, p],
        vec![
            Statement::Store(s.place(), string(text, b"abc")),
            Statement::Store(
                p.place(),
                field(
                    ValueExpr::Load(s.place()),
                    SequenceField::Data,
                    byte_pointer,
                ),
            ),
            Statement::Store(s.place(), string(text, b"xyz")),
            Statement::StoreInt(s_count, int(1)),
            ret(binary(IntOp::Add, first, IntExpr::load(s_count))),
        ],
    );
    f.procedures.push(root);
    f.places = places.freeze();
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(98)]));
}

#[test]
fn writing_through_literal_string_data_is_rejected() {
    let mut f = fixture();
    let mut places = PlaceRegistry::new();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let byte_pointer = f.types.pointer(byte).unwrap();
    let byte_place = places
        .dereference(
            field(string(text, b"abc"), SequenceField::Data, byte_pointer),
            &f.types,
        )
        .unwrap();
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::Store(byte_place, ValueExpr::Int(typed(IntegerType::U8, 0))),
            ret(int(1)),
        ],
    );
    f.procedures.push(root);
    f.places = places.freeze();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Failed(Error::ReadOnlyStorage)
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    let data = vm.evaluate(&field(
        string(text, b"abc"),
        SequenceField::Data,
        byte_pointer,
    ));
    let Outcome::Complete(values) = data.outcome else {
        panic!("{data:?}");
    };
    assert_eq!(
        vm.memory()
            .load(&f.types, values[0].pointer().unwrap())
            .unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 97))
    );
    assert_eq!(vm.memory().allocation_count(), 1);
}

#[test]
fn static_array_views_survive_callee_frames_and_reuse_backing() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 2).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let integer_pointer = f.types.pointer(integer).unwrap();
    let a = typed_local(&f, 0, 0, slice);
    let b = typed_local(&f, 0, 1, slice);
    value_procedure(
        &mut f,
        1,
        slice,
        vec![],
        vec![return_value(ValueExpr::ArrayView {
            array: Box::new(ValueExpr::Array {
                ty: array,
                elements: vec![ValueExpr::Int(int(10)), ValueExpr::Int(int(20))],
            }),
            ty: slice,
        })],
    );
    let call = || ValueExpr::Call {
        call: super::call(1, vec![]),
        ty: slice,
    };
    let same = BoolExpr::ComparePointers(
        jai_types::Equality::Equal,
        Box::new(field(
            ValueExpr::Load(a.place()),
            SequenceField::Data,
            integer_pointer,
        )),
        Box::new(field(
            ValueExpr::Load(b.place()),
            SequenceField::Data,
            integer_pointer,
        )),
    );
    let index = integer_expression(
        ValueExpr::Index {
            check: CheckMode::Enabled,
            base: Box::new(ValueExpr::Load(a.place())),
            index: int(1),
            ty: integer,
        },
        IntegerType::S64,
    );
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![a, b],
        vec![
            Statement::Store(a.place(), call()),
            Statement::Store(b.place(), call()),
            ret(binary(
                IntOp::Add,
                index,
                IntExpr::new(IntegerType::S64, IntExprKind::FromBool(Box::new(same))),
            )),
        ],
    );
    f.procedures.push(root);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(21)]));
}

#[test]
fn empty_static_array_and_string_data_are_null_without_allocations() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 0).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let integer_pointer = f.types.pointer(integer).unwrap();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let byte_pointer = f.types.pointer(byte).unwrap();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let empty_view = ValueExpr::ArrayView {
        array: Box::new(ValueExpr::Array {
            ty: array,
            elements: vec![],
        }),
        ty: slice,
    };
    let view = vm.evaluate(&empty_view);
    let Outcome::Complete(values) = view.outcome else {
        panic!("{view:?}");
    };
    let Value::Slice {
        pointer,
        count,
        ..
    } = &values[0]
    else {
        panic!("{values:?}");
    };
    assert!(pointer.is_null());
    assert_eq!(*count, 0);
    let empty_data = vm.evaluate(&field(string(text, b""), SequenceField::Data, byte_pointer));
    assert_eq!(
        empty_data.outcome,
        Outcome::Complete(vec![Value::Pointer(Pointer::null(byte))])
    );
    let array_data = vm.evaluate(&field(
        ValueExpr::Array {
            ty: array,
            elements: vec![],
        },
        SequenceField::Data,
        integer_pointer,
    ));
    assert_eq!(
        array_data.outcome,
        Outcome::Complete(vec![Value::Pointer(Pointer::null(integer))])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn array_variable_views_alias_mutable_array_storage() {
    let mut f = fixture();
    let mut places = PlaceRegistry::new();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 2).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let a = typed_local(&f, 0, 0, array);
    let s = typed_local(&f, 0, 1, slice);
    let element = places.index(a.place(), int(1), &f.types).unwrap();
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![a, s],
        vec![
            Statement::Store(
                a.place(),
                ValueExpr::Array {
                    ty: array,
                    elements: vec![ValueExpr::Int(int(10)), ValueExpr::Int(int(20))],
                },
            ),
            Statement::Store(
                s.place(),
                ValueExpr::ArrayToSlice {
                    array: a.place(),
                    ty: slice,
                },
            ),
            Statement::Store(element, ValueExpr::Int(int(99))),
            ret(integer_expression(
                ValueExpr::Index {
                    check: CheckMode::Enabled,
                    base: Box::new(ValueExpr::Load(s.place())),
                    index: int(1),
                    ty: integer,
                },
                IntegerType::S64,
            )),
        ],
    );
    f.procedures.push(root);
    f.places = places.freeze();
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(99)]));
}

#[test]
fn uninitialized_arrays_allow_metadata_and_aliases_before_element_writes() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 2).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let pointer = f.types.pointer(integer).unwrap();
    let a = typed_local(&f, 0, 0, array);
    let s = typed_local(&f, 0, 1, slice);
    let p = typed_local(&f, 0, 2, pointer);
    let count = typed_local(&f, 0, 3, integer);
    let mut places = PlaceRegistry::new();
    let first = places
        .dereference(ValueExpr::Load(p.place()), &f.types)
        .unwrap();
    let second = places.index(s.place(), int(1), &f.types).unwrap();
    let original_second = places.index(a.place(), int(1), &f.types).unwrap();
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![a, s, p, count],
        vec![
            Statement::Store(
                count.place(),
                field(ValueExpr::Load(a.place()), SequenceField::Count, integer),
            ),
            Statement::Store(
                p.place(),
                field(ValueExpr::Load(a.place()), SequenceField::Data, pointer),
            ),
            Statement::Store(
                s.place(),
                ValueExpr::ArrayToSlice {
                    array: a.place(),
                    ty: slice,
                },
            ),
            Statement::Store(first, ValueExpr::Int(int(10))),
            Statement::Store(
                s.place(),
                ValueExpr::ArrayView {
                    array: Box::new(ValueExpr::Load(a.place())),
                    ty: slice,
                },
            ),
            Statement::Store(second, ValueExpr::Int(int(30))),
            ret(binary(
                IntOp::Add,
                integer_expression(ValueExpr::Load(count.place()), IntegerType::S64),
                binary(
                    IntOp::Add,
                    integer_expression(ValueExpr::Load(first), IntegerType::S64),
                    integer_expression(ValueExpr::Load(original_second), IntegerType::S64),
                ),
            )),
        ],
    );
    f.procedures.push(root);
    f.places = places.freeze();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(42)])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn fixed_array_metadata_keeps_uninitialized_backing_unmaterialized() {
    for count in [0, 100_000] {
        let mut f = fixture();
        let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
        let array = f.types.fixed_array(integer, count).unwrap();
        let array_pointer = f.types.pointer(array).unwrap();
        let pointer = f.types.pointer(integer).unwrap();
        let slice = f.types.slice(integer).unwrap();
        let argument = typed_local(&f, 0, 0, array_pointer);
        let mut places = PlaceRegistry::new();
        let a = places
            .dereference(ValueExpr::Load(argument.place()), &f.types)
            .unwrap();
        let root = procedure(
            &mut f,
            0,
            vec![argument],
            vec![argument],
            vec![
                Statement::DiscardValue(field(ValueExpr::Load(a), SequenceField::Data, pointer)),
                Statement::DiscardValue(ValueExpr::ArrayView {
                    array: Box::new(ValueExpr::Load(a)),
                    ty: slice,
                }),
                ret(integer_expression(
                    field(ValueExpr::Load(a), SequenceField::Count, integer),
                    IntegerType::S64,
                )),
            ],
        );
        f.procedures.push(root);
        f.places = places.freeze();
        let mut vm = Vm::new(
            &f,
            NoEffects,
            Limits {
                fuel: 128,
                value_cells: 64,
                ..Limits::default()
            },
        )
        .unwrap();
        for ty in [array, array_pointer, integer] {
            vm.memory()
                .prepare_layout(&f.types, ty, usize::MAX)
                .1
                .unwrap();
        }
        let layout_cells = vm.memory().value_cells();
        let storage = vm.memory_mut().allocate(&f.types, array, None).unwrap();
        let baseline = vm.memory().value_cells();
        assert_eq!(baseline, layout_cells + 1);
        assert_eq!(
            vm.execute(ProcedureId::new(0), vec![Value::Pointer(storage.clone())])
                .outcome,
            Outcome::Complete(vec![value(count.into())])
        );
        assert_eq!(vm.memory().allocation_count(), 1);
        assert_eq!(vm.memory().value_cells(), baseline);
        assert_eq!(
            vm.memory().load(&f.types, &storage),
            Err(Error::Uninitialized)
        );
    }
}

#[test]
fn fixed_array_metadata_and_views_evaluate_projected_places_once() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 2).unwrap();
    let array_pointer = f.types.pointer(array).unwrap();
    let pointer = f.types.pointer(integer).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let tick = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let ticks = IntPlace::try_from_place(tick.storage().place(), &f.types).unwrap();
    let storage = Global::new_typed(
        1,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Zero,
        },
        &f.types,
    )
    .unwrap();
    let array_place = storage.storage().place();
    f.globals.extend([tick, storage]);
    value_procedure(
        &mut f,
        1,
        array_pointer,
        vec![],
        vec![
            Statement::StoreInt(ticks, binary(IntOp::Add, IntExpr::load(ticks), int(1))),
            return_value(ValueExpr::AddressOf {
                place: array_place,
                ty: array_pointer,
            }),
        ],
    );
    let mut places = PlaceRegistry::new();
    let a = places
        .dereference(
            ValueExpr::Call {
                call: call(1, vec![]),
                ty: array_pointer,
            },
            &f.types,
        )
        .unwrap();
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::DiscardValue(field(ValueExpr::Load(a), SequenceField::Count, integer)),
            Statement::DiscardValue(field(ValueExpr::Load(a), SequenceField::Data, pointer)),
            Statement::DiscardValue(ValueExpr::ArrayView {
                array: Box::new(ValueExpr::Load(a)),
                ty: slice,
            }),
            ret(IntExpr::load(ticks)),
        ],
    );
    f.procedures.push(root);
    f.places = places.freeze();
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(3)]));
}

#[test]
fn sequence_load_projection_is_evaluated_once() {
    let mut f = fixture();
    let mut places = PlaceRegistry::new();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let text = f.types.string();
    let pointer = f.types.pointer(text).unwrap();
    let tick = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let ticks = IntPlace::try_from_place(tick.storage().place(), &f.types).unwrap();
    let string_global = Global::new_typed(
        1,
        ConstantValue {
            ty: text,
            kind: ConstantKind::StringBytes(b"abc".to_vec()),
        },
        &f.types,
    )
    .unwrap();
    let string_place = string_global.storage().place();
    f.globals.extend([tick, string_global]);
    value_procedure(
        &mut f,
        1,
        pointer,
        vec![],
        vec![
            Statement::StoreInt(ticks, binary(IntOp::Add, IntExpr::load(ticks), int(1))),
            return_value(ValueExpr::AddressOf {
                place: string_place,
                ty: pointer,
            }),
        ],
    );
    let projected = places
        .dereference(
            ValueExpr::Call {
                call: call(1, vec![]),
                ty: pointer,
            },
            &f.types,
        )
        .unwrap();
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![ret(binary(
            IntOp::Add,
            integer_expression(
                field(ValueExpr::Load(projected), SequenceField::Count, integer),
                IntegerType::S64,
            ),
            binary(IntOp::Multiply, IntExpr::load(ticks), int(10)),
        ))],
    );
    f.procedures.push(root);
    f.places = places.freeze();
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(13)]));
}

#[test]
fn materialization_preserves_non_utf8_bytes_and_bounds_total_output() {
    let f = fixture();
    let text = f.types.string();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let storage = vm
        .memory_mut()
        .allocate(&f.types, text, Some(Value::String(vec![0xff, 0, 7])))
        .unwrap();
    let pointer = vm.memory().sequence_data(&f.types, &storage).unwrap();
    let descriptor = Value::StringView {
        pointer: pointer.clone(),
        count: 3,
    };
    assert_eq!(
        vm.materialize_value(&descriptor).unwrap(),
        Value::String(vec![0xff, 0, 7])
    );
    assert!(matches!(
        vm.materialize_value(&Value::StringView {
            pointer,
            count: 4
        }),
        Err(Error::OutOfBounds { .. })
    ));
    let strict = Vm::new(
        &f,
        NoEffects,
        Limits {
            value_cells: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        strict.materialize_value(&descriptor),
        Err(Error::Limit(LimitKind::ValueCells))
    );
}

#[test]
fn materialization_batch_shares_cell_and_work_budgets() {
    let f = fixture();
    let arguments = [Value::String(vec![1, 2]), Value::String(vec![3, 4])];
    let cells = Vm::new(
        &f,
        NoEffects,
        Limits {
            value_cells: 5,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        cells.materialize_values(&arguments),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    let work = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 5,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        work.materialize_values(&arguments),
        Err(Error::Limit(LimitKind::Fuel))
    );
}

#[test]
fn materialization_rejects_inactive_snapshot_bytes_and_precharges_the_proof() {
    let mut f = fixture();
    let word = f.types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let union = f.types.reserve_record(RecordKind::Union);
    f.types.define_record(union, [word, byte]).unwrap();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let storage = vm
        .memory_mut()
        .allocate(
            &f.types,
            union,
            Some(Value::Union {
                ty: union,
                field: 0,
                value: Box::new(Value::Int(Integer::wrapping(
                    IntegerType::U64,
                    0x0102_0304_0506_0708,
                ))),
            }),
        )
        .unwrap();
    let low = vm.memory().field(&f.types, &storage, 1).unwrap();
    vm.memory_mut()
        .store(
            &f.types,
            &low,
            Value::Int(Integer::wrapping(IntegerType::U8, 42)),
        )
        .unwrap();
    let snapshot = vm.memory().load(&f.types, &storage).unwrap();
    assert!(matches!(snapshot, Value::StoredAggregate(_)));
    assert!(matches!(
        vm.materialize_value(&snapshot),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let strict = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        strict.materialize_value(&snapshot),
        Err(Error::Limit(LimitKind::Fuel))
    );

    let plain = Value::Union {
        ty: union,
        field: 1,
        value: Box::new(Value::Int(Integer::wrapping(IntegerType::U8, 42))),
    };
    let canonical = vm
        .memory_mut()
        .allocate(&f.types, union, Some(plain.clone()))
        .unwrap();
    vm.memory()
        .sequence_snapshot(&f.types, &canonical, 8)
        .unwrap();
    let snapshot = vm.memory().load(&f.types, &canonical).unwrap();
    assert_eq!(vm.materialize_value(&snapshot), Ok(plain));
}

#[test]
fn rvalue_array_carriers_keep_inactive_union_bytes_through_sequence_operations() {
    let mut f = fixture();
    let word = f.types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let count_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let union = f.types.reserve_record(RecordKind::Union);
    f.types.define_record(union, [word, byte]).unwrap();
    let word_field = f.types.field(union, 0).unwrap().id;
    let array = f.types.fixed_array(union, 1).unwrap();
    let slice = f.types.slice(union).unwrap();
    let pointer_type = f.types.pointer(union).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Array(vec![ConstantValue {
                ty: union,
                kind: ConstantKind::Union {
                    field: word_field,
                    value: Box::new(ConstantValue {
                        ty: word,
                        kind: ConstantKind::Int(Integer::wrapping(
                            IntegerType::U64,
                            0x0102_0304_0506_0708,
                        )),
                    }),
                },
            }]),
        },
        &f.types,
    )
    .unwrap();
    f.globals.push(global.clone());
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let Outcome::Complete(values) = vm
        .evaluate(&field(
            ValueExpr::Load(global.place()),
            SequenceField::Data,
            pointer_type,
        ))
        .outcome
    else {
        panic!("expected global data pointer")
    };
    let Value::Pointer(data) = &values[0] else {
        panic!("expected pointer")
    };
    let low = vm.memory().field(&f.types, data, 1).unwrap();
    vm.memory_mut()
        .store(
            &f.types,
            &low,
            Value::Int(Integer::wrapping(IntegerType::U8, 42)),
        )
        .unwrap();
    let base = ValueExpr::Conditional {
        ty: array,
        expression: Box::new(Conditional {
            condition: BoolExpr::Constant(true),
            then_value: ValueExpr::Load(global.place()),
            else_value: ValueExpr::Load(global.place()),
        }),
    };
    assert_eq!(
        vm.evaluate(&field(base.clone(), SequenceField::Count, count_type))
            .outcome,
        Outcome::Complete(vec![value(1)])
    );
    let bases = [
        base.clone(),
        ValueExpr::ArrayView {
            ty: slice,
            array: Box::new(base.clone()),
        },
        field(base, SequenceField::Data, pointer_type),
    ];
    for base in bases {
        let element = ValueExpr::Index {
            base: Box::new(base),
            index: int(0),
            ty: union,
            check: CheckMode::Enabled,
        };
        let expression = ValueExpr::Field {
            base: Box::new(element),
            field: word_field,
            ty: word,
        };
        assert_eq!(
            vm.evaluate(&expression).outcome,
            Outcome::Complete(vec![Value::Int(Integer::wrapping(
                IntegerType::U64,
                0x0102_0304_0506_072a
            ))])
        );
    }
}

#[test]
fn literal_backing_identity_survives_state_transfer() {
    let mut f = fixture();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = f.types.pointer(byte).unwrap();
    let expression = field(string(text, b"abc"), SequenceField::Data, pointer);
    let limits = Limits {
        allocations: 1,
        ..Limits::default()
    };
    let mut original = Vm::new(&f, NoEffects, limits).unwrap();
    let before = original.evaluate(&expression).outcome;
    let mut restored = Vm::with_state(&f, NoEffects, limits, original.into_state()).unwrap();
    assert_eq!(restored.evaluate(&expression).outcome, before);
    assert_eq!(restored.memory().allocation_count(), 1);
}

#[test]
fn literal_pool_growth_precharges_existing_large_keys_and_rolls_back() {
    let mut f = fixture();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = f.types.pointer(byte).unwrap();
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 17_000,
            ..Limits::default()
        },
    )
    .unwrap();
    let data = |bytes: &[u8]| field(string(text, bytes), SequenceField::Data, pointer);
    // First grow the pool with small keys. Subsequent large keys fit the same
    // per-request fuel budget, while their combined rehash cost eventually does not.
    for bytes in [vec![1], vec![2], vec![3]]
        .into_iter()
        .chain((4..8).map(|byte| vec![byte; 4096]))
    {
        let next_inspection = vm.test_literal_insert_inspection_work(bytes.len());
        vm.test_budget_source_work(17_000, true, next_inspection);
        assert!(matches!(
            vm.evaluate(&data(&bytes)).outcome,
            Outcome::Complete(_)
        ));
    }
    let mut exhausted = false;
    for number in 0u64..128 {
        let allocations = vm.memory().allocation_count();
        let next_inspection = vm.test_literal_insert_inspection_work(8);
        vm.test_budget_source_work(17_000, true, next_inspection);
        match vm.evaluate(&data(&number.to_le_bytes())).outcome {
            Outcome::Complete(_) => assert_eq!(vm.memory().allocation_count(), allocations + 1),
            Outcome::Failed(Error::Limit(LimitKind::Fuel)) => {
                assert_eq!(vm.memory().allocation_count(), allocations);
                exhausted = true;
                break;
            }
            result => panic!("unexpected literal publication outcome: {result:?}"),
        }
    }
    assert!(
        exhausted,
        "table growth must account for the earlier large key"
    );
}

#[test]
fn compiler_effect_arguments_materialize_string_descriptors_inside_transaction() {
    let mut f = fixture();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = f.types.pointer(byte).unwrap();
    let signature = signature(&mut f.types, vec![text], vec![]);
    f.signatures.insert(ProcedureId::new(99), signature);
    f.compiler = Some(CompilerProcedure {
        signature,
        intrinsic: CompilerIntrinsic::Message(MessageLevel::Info),
    });
    let mut vm = Vm::new(&f, RecordingEffects::default(), Limits::default()).unwrap();
    let data = vm.evaluate(&field(string(text, b"abc"), SequenceField::Data, pointer));
    let Outcome::Complete(values) = data.outcome else {
        panic!("{data:?}");
    };
    let pointer = values[0].pointer().unwrap().clone();
    let result = vm.execute(
        ProcedureId::new(99),
        vec![Value::StringView {
            pointer: pointer.clone(),
            count: 3,
        }],
    );
    assert_eq!(result.outcome, Outcome::Complete(vec![]));
    assert_eq!(
        vm.effects().committed,
        vec![CompilerRequest::Message {
            level: MessageLevel::Info,
            text: "abc".into()
        }]
    );
    let result = vm.execute(
        ProcedureId::new(99),
        vec![Value::StringView {
            pointer,
            count: 4,
        }],
    );
    assert!(matches!(
        result.outcome,
        Outcome::Failed(Error::OutOfBounds { .. })
    ));
    assert_eq!(vm.effects().committed.len(), 1);
    assert_eq!(vm.effects().finishes, vec![true, true, false]);
}

#[test]
fn disabled_index_checks_skip_descriptor_count_but_preserve_backing_bounds() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer, 4).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let dynamic = f.types.dynamic_array(integer).unwrap();
    let pointer = f.types.pointer(integer).unwrap();
    let array_data = field(
        ValueExpr::Array {
            ty: array,
            elements: [10, 20, 30, 40]
                .into_iter()
                .map(|number| ValueExpr::Int(int(number)))
                .collect(),
        },
        SequenceField::Data,
        pointer,
    );
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    for ty in [slice, dynamic] {
        let mut fields = vec![
            (SequenceField::Count, ValueExpr::Int(int(1))),
            (SequenceField::Data, array_data.clone()),
        ];
        if ty == dynamic {
            fields.push((SequenceField::Allocated, ValueExpr::Int(int(4))));
        }
        let descriptor = ValueExpr::SequenceBuild {
            ty,
            initializers: fields,
        };
        let index = |number, check| ValueExpr::Index {
            base: Box::new(descriptor.clone()),
            index: int(number),
            ty: integer,
            check,
        };
        assert_eq!(
            vm.evaluate(&index(2, CheckMode::Enabled)).outcome,
            Outcome::Failed(Error::OutOfBounds {
                index: 2,
                length: 1
            })
        );
        assert_eq!(
            vm.evaluate(&index(2, CheckMode::Disabled)).outcome,
            Outcome::Complete(vec![value(30)])
        );
        assert!(matches!(
            vm.evaluate(&index(4, CheckMode::Disabled)).outcome,
            Outcome::Failed(Error::OutOfBounds { .. })
        ));
        assert!(matches!(
            vm.evaluate(&index(-1, CheckMode::Disabled)).outcome,
            Outcome::Failed(Error::OutOfBounds { .. })
        ));
    }
}

#[test]
fn disabled_string_index_checks_retain_actual_byte_backing_bounds() {
    let mut f = fixture();
    let text = f.types.string();
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer = f.types.pointer(byte).unwrap();
    let descriptor = ValueExpr::SequenceBuild {
        ty: text,
        initializers: vec![
            (SequenceField::Count, ValueExpr::Int(int(1))),
            (
                SequenceField::Data,
                field(string(text, b"abc"), SequenceField::Data, pointer),
            ),
        ],
    };
    let index = |number, check| ValueExpr::Index {
        base: Box::new(descriptor.clone()),
        index: int(number),
        ty: byte,
        check,
    };
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate(&index(2, CheckMode::Enabled)).outcome,
        Outcome::Failed(Error::OutOfBounds {
            index: 2,
            length: 1
        })
    );
    assert_eq!(
        vm.evaluate(&index(2, CheckMode::Disabled)).outcome,
        Outcome::Complete(vec![Value::Int(Integer::wrapping(IntegerType::U8, 99))])
    );
    assert!(matches!(
        vm.evaluate(&index(3, CheckMode::Disabled)).outcome,
        Outcome::Failed(Error::OutOfBounds { .. })
    ));
}

#[test]
fn signed_descriptor_fields_roundtrip_through_local_storage() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let text = f.types.string();
    let slice = f.types.slice(integer).unwrap();
    let dynamic = f.types.dynamic_array(integer).unwrap();
    let mut places = PlaceRegistry::new();
    for (procedure, ty) in [text, slice, dynamic].into_iter().enumerate() {
        let descriptor = typed_local(&f, procedure, 0, ty);
        let count = places
            .sequence_field(descriptor.place(), SequenceField::Count, &f.types)
            .unwrap();
        let mut initializers = vec![(SequenceField::Count, ValueExpr::Int(int(1)))];
        if ty == dynamic {
            initializers.push((SequenceField::Allocated, ValueExpr::Int(int(-7))));
        }
        value_procedure(
            &mut f,
            procedure,
            integer,
            vec![descriptor],
            vec![
                Statement::Store(
                    descriptor.place(),
                    ValueExpr::SequenceBuild {
                        ty,
                        initializers,
                    },
                ),
                Statement::Store(count, ValueExpr::Int(int(i64::MIN.into()))),
                return_value(ValueExpr::Load(count)),
            ],
        );
    }
    let descriptor = typed_local(&f, 3, 0, dynamic);
    let allocated = places
        .sequence_field(descriptor.place(), SequenceField::Allocated, &f.types)
        .unwrap();
    value_procedure(
        &mut f,
        3,
        integer,
        vec![descriptor],
        vec![
            Statement::Store(
                descriptor.place(),
                ValueExpr::SequenceBuild {
                    ty: dynamic,
                    initializers: vec![
                        (SequenceField::Count, ValueExpr::Int(int(i64::MAX.into()))),
                        (SequenceField::Allocated, ValueExpr::Int(int(0))),
                    ],
                },
            ),
            Statement::Store(allocated, ValueExpr::Int(int(i64::MIN.into()))),
            return_value(ValueExpr::Load(allocated)),
        ],
    );
    f.places = places.freeze();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    for procedure in 0..4 {
        assert_eq!(
            vm.execute(ProcedureId::new(procedure), vec![]).outcome,
            Outcome::Complete(vec![value(i64::MIN.into())])
        );
        assert_eq!(vm.memory().allocation_count(), 0);
    }
}

#[test]
fn signed_count_field_reads_and_unchecked_access_do_not_validate_logical_extents() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(integer).unwrap();
    let array = f.types.fixed_array(integer, 1).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let dynamic = f.types.dynamic_array(integer).unwrap();
    let data = field(
        ValueExpr::Array {
            ty: array,
            elements: vec![ValueExpr::Int(int(42))],
        },
        SequenceField::Data,
        pointer,
    );
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    for ty in [slice, dynamic] {
        for count in [-1, i64::MIN] {
            let mut initializers = vec![
                (SequenceField::Data, data.clone()),
                (SequenceField::Count, ValueExpr::Int(int(count.into()))),
            ];
            if ty == dynamic {
                initializers.push((SequenceField::Allocated, ValueExpr::Int(int(-9))));
            }
            let descriptor = ValueExpr::SequenceBuild {
                ty,
                initializers,
            };
            assert_eq!(
                vm.evaluate(&field(descriptor.clone(), SequenceField::Count, integer))
                    .outcome,
                Outcome::Complete(vec![value(count.into())])
            );
            let index = |check| ValueExpr::Index {
                base: Box::new(descriptor.clone()),
                index: int(0),
                ty: integer,
                check,
            };
            assert_eq!(
                vm.evaluate(&index(CheckMode::Enabled)).outcome,
                Outcome::Failed(Error::OutOfBounds {
                    index: 0,
                    length: 0
                })
            );
            assert_eq!(
                vm.evaluate(&index(CheckMode::Disabled)).outcome,
                Outcome::Complete(vec![value(42)])
            );
        }
    }
    let null_descriptor = ValueExpr::SequenceBuild {
        ty: slice,
        initializers: vec![(SequenceField::Count, ValueExpr::Int(int(1)))],
    };
    assert_eq!(
        vm.evaluate(&field(
            null_descriptor.clone(),
            SequenceField::Count,
            integer
        ))
        .outcome,
        Outcome::Complete(vec![value(1)])
    );
    assert_eq!(
        vm.evaluate(&ValueExpr::Index {
            base: Box::new(null_descriptor),
            index: int(0),
            ty: integer,
            check: CheckMode::Enabled,
        })
        .outcome,
        Outcome::Failed(Error::NullPointer)
    );
}

#[test]
fn indexed_places_apply_signed_counts_after_descriptor_field_stores() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(integer).unwrap();
    let slice = f.types.slice(integer).unwrap();
    let array = f.types.fixed_array(integer, 1).unwrap();
    let data = field(
        ValueExpr::Array {
            ty: array,
            elements: vec![ValueExpr::Int(int(42))],
        },
        SequenceField::Data,
        pointer,
    );
    let mut places = PlaceRegistry::new();
    for (procedure, check) in [CheckMode::Enabled, CheckMode::Disabled]
        .into_iter()
        .enumerate()
    {
        let descriptor = typed_local(&f, procedure, 0, slice);
        let count = places
            .sequence_field(descriptor.place(), SequenceField::Count, &f.types)
            .unwrap();
        let indexed = places
            .index_with_check(descriptor.place(), int(0), check, &f.types)
            .unwrap();
        value_procedure(
            &mut f,
            procedure,
            integer,
            vec![descriptor],
            vec![
                Statement::Store(
                    descriptor.place(),
                    ValueExpr::SequenceBuild {
                        ty: slice,
                        initializers: vec![
                            (SequenceField::Count, ValueExpr::Int(int(1))),
                            (SequenceField::Data, data.clone()),
                        ],
                    },
                ),
                Statement::Store(count, ValueExpr::Int(int(i64::MIN.into()))),
                return_value(ValueExpr::Load(indexed)),
            ],
        );
    }
    f.places = places.freeze();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Failed(Error::OutOfBounds {
            index: 0,
            length: 0
        })
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(
        vm.execute(ProcedureId::new(1), vec![]).outcome,
        Outcome::Complete(vec![value(42)])
    );
}

#[test]
fn negative_string_counts_are_preserved_until_materialization_uses_the_bytes() {
    let f = fixture();
    let text = f.types.string();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let descriptor = ValueExpr::SequenceBuild {
        ty: text,
        initializers: vec![(SequenceField::Count, ValueExpr::Int(int(i64::MIN.into())))],
    };
    assert_eq!(
        vm.evaluate(&field(descriptor.clone(), SequenceField::Count, integer))
            .outcome,
        Outcome::Complete(vec![value(i64::MIN.into())])
    );
    let Outcome::Complete(values) = vm.evaluate(&descriptor).outcome else {
        panic!("descriptor construction must preserve signed fields");
    };
    assert_eq!(vm.materialize_value(&values[0]), Err(Error::CheckedCast));
}
