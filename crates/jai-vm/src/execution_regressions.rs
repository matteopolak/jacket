use super::*;

#[test]
fn argument_and_return_lists_share_a_budget_before_later_pending_expressions() {
    for returns in [false, true] {
        let mut f = fixture();
        let empty = f.types.reserve_record(RecordKind::Struct);
        f.types.define_record(empty, []).unwrap();
        let array = f.types.fixed_array(empty, 3).unwrap();
        let pending = ProcedureId::new(1);
        let pending_signature = signature(&mut f.types, vec![], vec![array]);
        f.signatures.insert(pending, pending_signature);
        f.pending = Some(pending);
        let later = ValueExpr::Call {
            call: Call::new(pending, vec![]),
            ty: array,
        };
        let values = vec![ValueExpr::Zero(array), ValueExpr::Zero(array), later];
        let outcome = if returns {
            let signature = signature(&mut f.types, vec![], vec![array; 3]);
            f.signatures.insert(ProcedureId::new(0), signature);
            f.procedures.push(Procedure {
                id: ProcedureId::new(0),
                signature,
                parameters: vec![],
                locals: vec![],
                body: block(vec![Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnValues(values),
                })]),
                cleanups: vec![],
            });
            Vm::new(
                &f,
                NoEffects,
                Limits {
                    value_cells: 6,
                    ..Limits::default()
                },
            )
            .unwrap()
            .execute(ProcedureId::new(0), vec![])
            .outcome
        } else {
            let target = ProcedureId::new(2);
            let signature = signature(&mut f.types, vec![array; 3], vec![]);
            f.signatures.insert(target, signature);
            Vm::new(
                &f,
                NoEffects,
                Limits {
                    value_cells: 6,
                    ..Limits::default()
                },
            )
            .unwrap()
            .evaluate_call(&Call::new(
                target,
                values
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| (ParameterId::new(index), value))
                    .collect(),
            ))
            .outcome
        };
        assert_eq!(
            outcome,
            Outcome::Failed(Error::Limit(LimitKind::ValueCells))
        );
    }
}

#[test]
fn scalar_stores_charge_warm_aggregate_root_copies_and_rollback_partial_writes() {
    for boolean in [false, true] {
        let mut f = fixture();
        let element = f.types.scalar(if boolean {
            ScalarType::Bool
        } else {
            ScalarType::Int(IntegerType::S64)
        });
        let array = f.types.fixed_array(element, 512).unwrap();
        let global = Global::new_typed(
            0,
            ConstantValue {
                ty: array,
                kind: ConstantKind::Zero,
            },
            &f.types,
        )
        .unwrap();
        let mut places = PlaceRegistry::new();
        let first = places.index(global.place(), int(0), &f.types).unwrap();
        let (load, store) = if boolean {
            let place = BoolPlace::try_from_place(first, &f.types).unwrap();
            (
                IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::FromBool(Box::new(BoolExpr::Load(place))),
                ),
                Statement::StoreBool(place, BoolExpr::Constant(true)),
            )
        } else {
            let place = IntPlace::try_from_place(first, &f.types).unwrap();
            (IntExpr::load(place), Statement::StoreInt(place, int(42)))
        };
        let reader = procedure(&mut f, 0, vec![], vec![], vec![ret(load.clone())]);
        let writer = procedure(
            &mut f,
            1,
            vec![],
            vec![],
            vec![store.clone(), store.clone(), store, ret(load)],
        );
        f.globals.push(global);
        f.procedures.extend([reader, writer]);
        f.places = places.freeze();
        let mut vm = Vm::new(
            &f,
            NoEffects,
            Limits {
                fuel: 2_000,
                ..Limits::default()
            },
        )
        .unwrap();
        vm.test_budget_source_work(2_000, false, 0);
        assert_eq!(
            vm.execute(ProcedureId::new(0), vec![]).outcome,
            Outcome::Complete(vec![value(0)])
        );
        assert_eq!(vm.memory().allocation_count(), 1);
        vm.test_budget_source_work(2_000, false, 0);
        assert_eq!(
            vm.execute(ProcedureId::new(1), vec![]).outcome,
            Outcome::Failed(Error::Limit(LimitKind::Fuel))
        );
        vm.test_budget_source_work(2_000, false, 0);
        assert_eq!(
            vm.execute(ProcedureId::new(0), vec![]).outcome,
            Outcome::Complete(vec![value(0)])
        );
    }
}

#[test]
fn aggregate_results_count_projected_pointer_children_before_publication() {
    let mut f = fixture();
    let word = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let mut nested = word;
    for _ in 0..64 {
        let outer = f.types.reserve_record(RecordKind::Struct);
        f.types.define_record(outer, [nested]).unwrap();
        nested = outer;
    }
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: nested,
            kind: ConstantKind::Zero,
        },
        &f.types,
    )
    .unwrap();
    let mut places = PlaceRegistry::new();
    let mut place = global.place();
    while matches!(
        f.types.kind(place.ty()).unwrap(),
        jai_types::TypeKind::Record(_)
    ) {
        place = places
            .field(place, f.types.field(place.ty(), 0).unwrap().id, &f.types)
            .unwrap();
    }
    let pointer = f.types.pointer(word).unwrap();
    let array = f.types.fixed_array(pointer, 8).unwrap();
    let record = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(record, [pointer; 8]).unwrap();
    let initializers: Vec<_> = f
        .types
        .record_definition(record)
        .unwrap()
        .fields
        .iter()
        .enumerate()
        .map(|(index, _)| {
            (
                f.types.field(record, index).unwrap().id,
                ValueExpr::AddressOf { place, ty: pointer },
            )
        })
        .collect();
    f.globals.push(global);
    f.places = places.freeze();
    let fields = vec![ValueExpr::AddressOf { place, ty: pointer }; 8];
    let expressions = [
        ValueExpr::Array {
            ty: array,
            elements: fields.clone(),
        },
        ValueExpr::Record { ty: record, fields },
        ValueExpr::RecordBuild {
            ty: record,
            initializers,
        },
    ];
    for expression in expressions {
        let mut vm = Vm::new(
            &f,
            NoEffects,
            Limits {
                value_cells: 128,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(
            vm.evaluate(&expression).outcome,
            Outcome::Failed(Error::Limit(LimitKind::ValueCells))
        );
        assert_eq!(vm.memory().allocation_count(), 0);
    }
}

fn add_array(f: &mut Fixture, index: usize, values: &[i128]) -> Global {
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let ty = f.types.fixed_array(integer, values.len() as u64).unwrap();
    Global::new_typed(
        index,
        ConstantValue {
            ty,
            kind: ConstantKind::Array(
                values
                    .iter()
                    .map(|number| ConstantValue {
                        ty: integer,
                        kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, *number)),
                    })
                    .collect(),
            ),
        },
        &f.types,
    )
    .unwrap()
}

#[test]
fn indexed_store_captures_destination_before_rhs_and_executes_each_call_once() {
    for scalar_store in [true, false] {
        let mut f = fixture();
        let trace = Global::new(
            0,
            GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
            &f.types,
        );
        let trace_place = IntPlace::try_from_place(trace.place(), &f.types).unwrap();
        let array = add_array(&mut f, 1, &[0, 0]);
        let mut places = PlaceRegistry::new();
        let target = places
            .index(array.place(), call_int(1, vec![]), &f.types)
            .unwrap();
        let result = places.index(array.place(), int(0), &f.types).unwrap();
        let result = IntPlace::try_from_place(result, &f.types).unwrap();
        f.globals.extend([trace, array]);
        let index = procedure(
            &mut f,
            1,
            vec![],
            vec![],
            vec![
                Statement::StoreInt(
                    trace_place,
                    binary(
                        IntOp::Add,
                        binary(IntOp::Multiply, IntExpr::load(trace_place), int(10)),
                        int(1),
                    ),
                ),
                ret(int(0)),
            ],
        );
        let rhs = procedure(
            &mut f,
            2,
            vec![],
            vec![],
            vec![
                Statement::StoreInt(
                    trace_place,
                    binary(
                        IntOp::Add,
                        binary(IntOp::Multiply, IntExpr::load(trace_place), int(10)),
                        int(2),
                    ),
                ),
                ret(int(7)),
            ],
        );
        let store = if scalar_store {
            Statement::StoreInt(
                IntPlace::try_from_place(target, &f.types).unwrap(),
                call_int(2, vec![]),
            )
        } else {
            Statement::Store(target, ValueExpr::Int(call_int(2, vec![])))
        };
        let root = procedure(
            &mut f,
            0,
            vec![],
            vec![],
            vec![
                store,
                ret(binary(
                    IntOp::Add,
                    binary(IntOp::Multiply, IntExpr::load(trace_place), int(10)),
                    IntExpr::load(result),
                )),
            ],
        );
        f.procedures.extend([root, index, rhs]);
        f.places = places.freeze();
        assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(127)]));
    }
}

#[test]
fn index_call_mutates_descriptor_after_original_base_has_been_captured() {
    let mut f = fixture();
    let first = add_array(&mut f, 0, &[10]);
    let second = add_array(&mut f, 1, &[20]);
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = f.types.slice(integer).unwrap();
    f.types.pointer(integer).unwrap();
    let descriptor = Global::new_typed(
        2,
        ConstantValue {
            ty: slice,
            kind: ConstantKind::Zero,
        },
        &f.types,
    )
    .unwrap();
    let mut places = PlaceRegistry::new();
    let indexed = places
        .index(descriptor.place(), call_int(1, vec![]), &f.types)
        .unwrap();
    let indexed = IntPlace::try_from_place(indexed, &f.types).unwrap();
    let current = places.index(descriptor.place(), int(0), &f.types).unwrap();
    let current = IntPlace::try_from_place(current, &f.types).unwrap();
    let mutate = procedure(
        &mut f,
        1,
        vec![],
        vec![],
        vec![
            Statement::Store(
                descriptor.place(),
                ValueExpr::ArrayToSlice {
                    array: second.place(),
                    ty: slice,
                },
            ),
            ret(int(0)),
        ],
    );
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::Store(
                descriptor.place(),
                ValueExpr::ArrayToSlice {
                    array: first.place(),
                    ty: slice,
                },
            ),
            ret(IntExpr::load(indexed)),
        ],
    );
    let getter = procedure(&mut f, 2, vec![], vec![], vec![ret(IntExpr::load(current))]);
    f.procedures.extend([root, mutate, getter]);
    f.globals.extend([first, second, descriptor]);
    f.places = places.freeze();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(10)])
    );
    assert_eq!(
        vm.execute(ProcedureId::new(2), vec![]).outcome,
        Outcome::Complete(vec![value(20)])
    );
}

#[test]
fn publication_validation_and_checked_effect_finalization_rollback_successful_execution() {
    let mut f = fixture();
    let trace = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let trace_place = IntPlace::try_from_place(trace.place(), &f.types).unwrap();
    let text = f.types.string();
    let effect_signature = signature(&mut f.types, vec![text], vec![]);
    f.compiler = Some(CompilerProcedure {
        signature: effect_signature,
        intrinsic: CompilerIntrinsic::Message(MessageLevel::Info),
    });
    f.signatures.insert(ProcedureId::new(99), effect_signature);
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::StoreInt(trace_place, int(7)),
            Statement::CallVoid(call(
                99,
                vec![(
                    0,
                    ValueExpr::StringBytes {
                        ty: text,
                        bytes: b"before publication".to_vec(),
                    },
                )],
            )),
            ret(IntExpr::load(trace_place)),
        ],
    );
    let getter = procedure(
        &mut f,
        1,
        vec![],
        vec![],
        vec![ret(IntExpr::load(trace_place))],
    );
    f.globals.push(trace);
    f.procedures.extend([root, getter]);
    let expression = ValueExpr::Int(call_int(0, vec![]));
    let mut vm = Vm::new(&f, RecordingEffects::default(), Limits::default()).unwrap();
    let result = vm.evaluate_validated(&expression, |_, values| {
        assert_eq!(values, &[value(7)]);
        Err(Error::InvalidIr("publication rejected"))
    });
    assert_eq!(
        result.outcome,
        Outcome::Failed(Error::InvalidIr("publication rejected"))
    );
    assert!(vm.effects().committed.is_empty());
    assert_eq!(vm.effects().finishes, &[false]);
    assert_eq!(
        vm.execute(ProcedureId::new(1), vec![]).outcome,
        Outcome::Complete(vec![value(0)])
    );

    struct RejectCommit {
        inner: RecordingEffects,
        reject: bool,
    }
    impl CompilerEffects for RejectCommit {
        fn begin(&mut self) {
            self.inner.begin()
        }
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            self.inner.request(request)
        }
        fn finish(&mut self, commit: bool) -> Result<(), Error> {
            if commit && self.reject {
                self.inner.finish(false)?;
                return Err(Error::EffectRejected("stream ended early".into()));
            }
            self.inner.finish(commit)
        }
    }
    let mut vm = Vm::new(
        &f,
        RejectCommit {
            inner: RecordingEffects::default(),
            reject: true,
        },
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        vm.evaluate(&expression).outcome,
        Outcome::Failed(Error::EffectRejected("stream ended early".into()))
    );
    assert!(vm.effects().inner.committed.is_empty());
    vm.effects_mut().reject = false;
    assert_eq!(
        vm.execute(ProcedureId::new(1), vec![]).outcome,
        Outcome::Complete(vec![value(0)])
    );
    assert_eq!(
        vm.evaluate(&expression).outcome,
        Outcome::Complete(vec![value(7)])
    );
    assert_eq!(vm.effects().inner.committed.len(), 1);
}
