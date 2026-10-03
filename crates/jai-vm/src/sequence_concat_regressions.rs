use super::*;

fn typed_local(f: &Fixture, procedure: usize, index: usize, ty: TypeId) -> Local {
    Local::new_typed(ProcedureId::new(procedure), index, ty, &f.types).unwrap()
}
fn index(base: Place, number: i128, element: TypeId) -> ValueExpr {
    ValueExpr::Index {
        base: Box::new(ValueExpr::Load(base)),
        index: int(number),
        ty: element,
        check: CheckMode::Enabled,
    }
}
fn integer(expression: ValueExpr, ty: IntegerType) -> IntExpr {
    IntExpr::new(ty, IntExprKind::Value(Box::new(expression)))
}
fn count(base: Place, integer_type: TypeId) -> IntExpr {
    integer(
        ValueExpr::SequenceField {
            base: Box::new(ValueExpr::Load(base)),
            field: SequenceField::Count,
            ty: integer_type,
        },
        IntegerType::S64,
    )
}
fn pack(ty: TypeId, parts: Vec<SequencePackPart>) -> ValueExpr {
    ValueExpr::SequenceConcat {
        ty,
        parts,
    }
}
fn element(value: i128) -> SequencePackPart {
    SequencePackPart::Element(ValueExpr::Int(int(value)))
}
fn variadic(
    f: &mut Fixture,
    procedure: usize,
    element: TypeId,
    parameter: Local,
    result: TypeId,
    statements: Vec<Statement>,
) {
    let signature = f
        .types
        .procedure(ProcedureType {
            parameters: vec![parameter.ty()].into(),
            results: vec![result].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: jai_types::Variadic::Jai {
                parameter: 0,
                element,
            },
        })
        .unwrap();
    f.signatures.insert(ProcedureId::new(procedure), signature);
    f.procedures.push(Procedure {
        id: ProcedureId::new(procedure),
        signature,
        parameters: vec![parameter],
        locals: vec![parameter],
        body: block(statements),
        cleanups: vec![],
    });
}
fn return_value(value: ValueExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![value]),
    })
}

#[test]
fn concat_spread_is_copied_before_a_later_part_mutates_its_original_array() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = f.types.fixed_array(integer_type, 2).unwrap();
    let slice = f.types.slice(integer_type).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Array(
                [10, 20]
                    .into_iter()
                    .map(|number| ConstantValue {
                        ty: integer_type,
                        kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, number)),
                    })
                    .collect(),
            ),
        },
        &f.types,
    )
    .unwrap();
    let global_place = global.place();
    f.globals.push(global);
    let mut places = PlaceRegistry::new();
    let first = places.index(global_place, int(0), &f.types).unwrap();
    f.places = places.freeze();
    let mutate = procedure(
        &mut f,
        1,
        vec![],
        vec![],
        vec![
            Statement::Store(first, ValueExpr::Int(int(99))),
            ret(int(42)),
        ],
    );
    f.procedures.push(mutate);
    let args = typed_local(&f, 2, 0, slice);
    let encode = binary(
        IntOp::Add,
        binary(
            IntOp::Add,
            binary(
                IntOp::Multiply,
                integer(index(args.place(), 0, integer_type), IntegerType::S64),
                int(100),
            ),
            binary(
                IntOp::Multiply,
                integer(index(args.place(), 1, integer_type), IntegerType::S64),
                int(10),
            ),
        ),
        integer(index(args.place(), 2, integer_type), IntegerType::S64),
    );
    variadic(
        &mut f,
        2,
        integer_type,
        args,
        integer_type,
        vec![ret(encode)],
    );
    let expression = pack(
        slice,
        vec![
            SequencePackPart::Spread(ValueExpr::ArrayToSlice {
                array: global_place,
                ty: slice,
            }),
            SequencePackPart::Element(ValueExpr::Int(call_int(1, vec![]))),
        ],
    );
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![ret(call_int(2, vec![(0, expression)]))],
    );
    f.procedures.push(root);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(1242)])
    );
    assert_eq!(vm.memory().allocation_count(), 1);
}

#[test]
fn scalar_and_spread_pack_copies_keep_inactive_union_bytes() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let union = f.types.reserve_record(RecordKind::Union);
    f.types.define_record(union, [integer_type, byte]).unwrap();
    let whole = f.types.field(union, 0).unwrap().id;
    let low = f.types.field(union, 1).unwrap().id;
    let array = f.types.fixed_array(union, 1).unwrap();
    let slice = f.types.slice(union).unwrap();
    let original = 0x0102_0304_0506_0708;
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Array(vec![ConstantValue {
                ty: union,
                kind: ConstantKind::Union {
                    field: whole,
                    value: Box::new(ConstantValue {
                        ty: integer_type,
                        kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, original)),
                    }),
                },
            }]),
        },
        &f.types,
    )
    .unwrap();
    let global_place = global.place();
    f.globals.push(global);
    let args = typed_local(&f, 1, 0, slice);
    let mut places = PlaceRegistry::new();
    let source = places.index(global_place, int(0), &f.types).unwrap();
    let source_low = places.field(source, low, &f.types).unwrap();
    let packed = places.index(args.place(), int(0), &f.types).unwrap();
    let packed_whole = places.field(packed, whole, &f.types).unwrap();
    f.places = places.freeze();
    variadic(
        &mut f,
        1,
        union,
        args,
        integer_type,
        vec![return_value(ValueExpr::Load(packed_whole))],
    );
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::Store(source_low, ValueExpr::Int(typed(IntegerType::U8, 42))),
            ret(call_int(
                1,
                vec![(
                    0,
                    pack(
                        slice,
                        vec![SequencePackPart::Element(ValueExpr::Load(source))],
                    ),
                )],
            )),
        ],
    );
    f.procedures.push(root);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let expected = value(0x0102_0304_0506_072a);
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![expected.clone()])
    );
    assert_eq!(
        vm.evaluate_call(&call(
            1,
            vec![(
                0,
                pack(
                    slice,
                    vec![SequencePackPart::Spread(ValueExpr::ArrayToSlice {
                        array: global_place,
                        ty: slice,
                    })],
                ),
            )],
        ))
        .outcome,
        Outcome::Complete(vec![expected])
    );
}

#[test]
fn concat_backing_belongs_to_the_caller_after_variadic_callee_returns() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = f.types.slice(integer_type).unwrap();
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        integer_type,
        args,
        slice,
        vec![return_value(ValueExpr::Load(args.place()))],
    );
    let result = typed_local(&f, 0, 0, slice);
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![result],
        vec![
            Statement::Store(
                result.place(),
                ValueExpr::Call {
                    call: call(1, vec![(0, pack(slice, vec![element(42)]))]),
                    ty: slice,
                },
            ),
            ret(integer(
                index(result.place(), 0, integer_type),
                IntegerType::S64,
            )),
        ],
    );
    f.procedures.push(root);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(42)])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(
        vm.evaluate_call(&call(1, vec![(0, pack(slice, vec![element(7)]))]))
            .outcome,
        Outcome::Failed(Error::SequenceTemporaryEscape)
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn inactive_union_bytes_cannot_hide_a_returned_caller_pack_pointer() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_type = f.types.pointer(integer_type).unwrap();
    let slice = f.types.slice(integer_type).unwrap();
    let union = f.types.reserve_record(RecordKind::Union);
    f.types.define_record(union, [pointer_type, byte]).unwrap();
    let pointer_field = f.types.field(union, 0).unwrap().id;
    let byte_field = f.types.field(union, 1).unwrap().id;
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        integer_type,
        args,
        pointer_type,
        vec![return_value(ValueExpr::SequenceField {
            base: Box::new(ValueExpr::Load(args.place())),
            field: SequenceField::Data,
            ty: pointer_type,
        })],
    );
    let local = typed_local(&f, 0, 0, union);
    let mut places = PlaceRegistry::new();
    let low = places.field(local.place(), byte_field, &f.types).unwrap();
    f.places = places.freeze();
    let root_signature = signature(&mut f.types, vec![], vec![union]);
    f.signatures.insert(ProcedureId::new(0), root_signature);
    f.procedures.push(Procedure {
        id: ProcedureId::new(0),
        signature: root_signature,
        parameters: vec![],
        locals: vec![local],
        body: block(vec![
            Statement::Store(
                local.place(),
                ValueExpr::Union {
                    ty: union,
                    field: pointer_field,
                    value: Box::new(ValueExpr::Call {
                        call: call(1, vec![(0, pack(slice, vec![element(42)]))]),
                        ty: pointer_type,
                    }),
                },
            ),
            Statement::Store(
                low,
                ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::U8, 0))),
            ),
            return_value(ValueExpr::Load(local.place())),
        ]),
        cleanups: vec![],
    });
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Failed(Error::SequenceTemporaryEscape)
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn root_call_temporaries_survive_call_and_validator_then_are_released() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = f.types.slice(integer_type).unwrap();
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        integer_type,
        args,
        integer_type,
        vec![ret(integer(
            index(args.place(), 0, integer_type),
            IntegerType::S64,
        ))],
    );
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let expression = call(1, vec![(0, pack(slice, vec![element(42)]))]);
    let result = vm.evaluate_call_validated(&expression, |vm, values| {
        assert_eq!(values, &[value(42)]);
        assert_eq!(vm.memory().allocation_count(), 1);
        Ok(())
    });
    assert_eq!(result.outcome, Outcome::Complete(vec![value(42)]));
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn returning_caller_owned_pack_backing_from_its_owner_rolls_back() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = f.types.slice(integer_type).unwrap();
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        integer_type,
        args,
        slice,
        vec![return_value(ValueExpr::Load(args.place()))],
    );
    let result = signature(&mut f.types, vec![], vec![slice]);
    f.signatures.insert(ProcedureId::new(0), result);
    f.procedures.push(Procedure {
        id: ProcedureId::new(0),
        signature: result,
        parameters: vec![],
        locals: vec![],
        body: block(vec![return_value(ValueExpr::Call {
            call: call(1, vec![(0, pack(slice, vec![element(42)]))]),
            ty: slice,
        })]),
        cleanups: vec![],
    });
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Failed(Error::SequenceTemporaryEscape)
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn returned_pack_address_integers_retain_their_owner_even_outside_its_range() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let unsigned = f.types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer = f.types.pointer(integer_type).unwrap();
    let slice = f.types.slice(integer_type).unwrap();
    for (procedure, offset) in [(1, 0), (2, 0x10_0000)] {
        let args = typed_local(&f, procedure, 0, slice);
        let address = IntExpr::new(
            IntegerType::U64,
            IntExprKind::FromPointer {
                value: Box::new(ValueExpr::SequenceField {
                    base: Box::new(ValueExpr::Load(args.place())),
                    field: SequenceField::Data,
                    ty: pointer,
                }),
                mode: jai_types::CastMode::Unchecked,
            },
        );
        variadic(
            &mut f,
            procedure,
            integer_type,
            args,
            unsigned,
            vec![ret(binary(
                IntOp::Add,
                address,
                typed(IntegerType::U64, offset),
            ))],
        );
    }
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    for procedure in [1, 2] {
        assert_eq!(
            vm.evaluate_call(&call(procedure, vec![(0, pack(slice, vec![element(42)]))],))
                .outcome,
            Outcome::Failed(Error::SequenceTemporaryEscape)
        );
        assert_eq!(vm.memory().allocation_count(), 0);
    }
}

#[test]
fn packed_address_integers_keep_origin_through_scalar_and_spread_images() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let unsigned = f.types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer = f.types.pointer(integer_type).unwrap();
    let slice = f.types.slice(unsigned).unwrap();
    let args = typed_local(&f, 1, 0, slice);
    let recovered = ValueExpr::PointerFromInteger {
        value: integer(index(args.place(), 0, unsigned), IntegerType::U64),
        ty: pointer,
        mode: jai_types::CastMode::Unchecked,
    };
    let mut places = PlaceRegistry::new();
    let recovered = places.dereference(recovered, &f.types).unwrap();
    f.places = places.freeze();
    variadic(
        &mut f,
        1,
        unsigned,
        args,
        integer_type,
        vec![ret(integer(ValueExpr::Load(recovered), IntegerType::S64))],
    );
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 42)),
        &f.types,
    );
    let global_place = global.place();
    f.globals.push(global);
    let converted = ValueExpr::Int(IntExpr::new(
        IntegerType::U64,
        IntExprKind::FromPointer {
            value: Box::new(ValueExpr::AddressOf {
                place: global_place,
                ty: pointer,
            }),
            mode: jai_types::CastMode::Unchecked,
        },
    ));
    let returned = typed_local(&f, 2, 0, slice);
    variadic(
        &mut f,
        2,
        unsigned,
        returned,
        slice,
        vec![return_value(ValueExpr::Load(returned.place()))],
    );
    let stored = typed_local(&f, 0, 0, slice);
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![stored],
        vec![
            Statement::Store(
                stored.place(),
                ValueExpr::Call {
                    call: call(
                        2,
                        vec![(
                            0,
                            pack(slice, vec![SequencePackPart::Element(converted.clone())]),
                        )],
                    ),
                    ty: slice,
                },
            ),
            ret(call_int(
                1,
                vec![(
                    0,
                    pack(
                        slice,
                        vec![SequencePackPart::Spread(ValueExpr::Load(stored.place()))],
                    ),
                )],
            )),
        ],
    );
    f.procedures.push(root);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let scalar = call(
        1,
        vec![(0, pack(slice, vec![SequencePackPart::Element(converted)]))],
    );
    assert_eq!(
        vm.evaluate_call(&scalar).outcome,
        Outcome::Complete(vec![value(42)])
    );
    let array = f.types.lookup(&jai_types::TypeKind::FixedArray {
        element: unsigned,
        count: 1,
    });
    assert!(array.is_none());
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(42)])
    );
    assert_eq!(vm.memory().allocation_count(), 1);
}

#[test]
fn small_spread_precharges_cold_backing_but_keeps_warm_snapshots_cheap() {
    fn run(warm: bool) -> Outcome {
        let mut f = fixture();
        let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
        let array = f.types.fixed_array(integer_type, 4096).unwrap();
        let slice = f.types.slice(integer_type).unwrap();
        let args = typed_local(&f, 1, 0, slice);
        variadic(
            &mut f,
            1,
            integer_type,
            args,
            integer_type,
            vec![ret(int(42))],
        );
        let pending_signature = signature(&mut f.types, vec![], vec![integer_type]);
        f.signatures.insert(ProcedureId::new(2), pending_signature);
        f.pending = Some(ProcedureId::new(2));
        let input = typed_local(&f, 0, 0, slice);
        let root = procedure(
            &mut f,
            0,
            vec![input],
            vec![input],
            vec![ret(call_int(
                1,
                vec![(
                    0,
                    pack(
                        slice,
                        vec![
                            SequencePackPart::Spread(ValueExpr::Load(input.place())),
                            SequencePackPart::Element(ValueExpr::Int(call_int(2, vec![]))),
                        ],
                    ),
                )],
            ))],
        );
        f.procedures.push(root);
        let mut vm = Vm::new(
            &f,
            NoEffects,
            Limits {
                fuel: 1000,
                ..Limits::default()
            },
        )
        .unwrap();
        let backing = vm
            .memory_mut()
            .allocate(
                &f.types,
                array,
                Some(Value::Array {
                    ty: array,
                    elements: vec![value(42); 4096],
                }),
            )
            .unwrap();
        let pointer = vm.memory().index(&f.types, &backing, 0).unwrap();
        if warm {
            vm.memory()
                .sequence_snapshot(&f.types, &pointer, 8)
                .unwrap();
        }
        let cells = vm.memory().value_cells();
        vm.test_budget_source_work(1_000, false, 0);
        let result = vm.execute(
            ProcedureId::new(0),
            vec![Value::Slice {
                ty: slice,
                pointer,
                count: 1,
            }],
        );
        assert_eq!(vm.memory().value_cells(), cells);
        result.outcome
    }
    assert_eq!(run(false), Outcome::Failed(Error::Limit(LimitKind::Fuel)));
    assert_eq!(
        run(true),
        Outcome::Pending(vec![Dependency::Procedure(ProcedureId::new(2))])
    );
}

#[test]
fn pack_snapshots_and_final_assembly_charge_address_origin_metadata() {
    const ORIGINS: usize = 128;
    fn run(spread: bool, addressed: bool, pending: bool, fuel: u64) -> (Execution, u64) {
        let mut f = fixture();
        let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
        let unsigned = f.types.scalar(ScalarType::Int(IntegerType::U64));
        let slice = f.types.slice(unsigned).unwrap();
        let args = typed_local(&f, 1, 0, slice);
        variadic(
            &mut f,
            1,
            unsigned,
            args,
            integer_type,
            vec![ret(count(args.place(), integer_type))],
        );
        let pending_signature = signature(&mut f.types, vec![], vec![unsigned]);
        f.signatures.insert(ProcedureId::new(2), pending_signature);
        f.pending = Some(ProcedureId::new(2));
        let input = typed_local(
            &f,
            0,
            0,
            if spread {
                slice
            } else {
                unsigned
            },
        );
        let mut parts = vec![if spread {
            SequencePackPart::Spread(ValueExpr::Load(input.place()))
        } else {
            SequencePackPart::Element(ValueExpr::Load(input.place()))
        }];
        if pending {
            parts.push(SequencePackPart::Element(ValueExpr::Call {
                call: call(2, vec![]),
                ty: unsigned,
            }));
        }
        let root = procedure(
            &mut f,
            0,
            vec![input],
            vec![input],
            vec![ret(call_int(1, vec![(0, pack(slice, parts))]))],
        );
        f.procedures.push(root);
        let mut vm = Vm::new(
            &f,
            NoEffects,
            Limits {
                fuel,
                ..Limits::default()
            },
        )
        .unwrap();
        let origins: Vec<_> = (0..ORIGINS)
            .map(|_| {
                vm.memory_mut()
                    .allocate(
                        &f.types,
                        unsigned,
                        Some(Value::Int(Integer::wrapping(IntegerType::U64, 0))),
                    )
                    .unwrap()
            })
            .collect();
        let integer = Integer::wrapping(IntegerType::U64, 0);
        let source = if addressed {
            Number::address(
                integer,
                AddressProvenance::Derived {
                    memory: origins[0].memory_identity(),
                    allocations: origins
                        .iter()
                        .map(|pointer| pointer.allocation_key().1)
                        .collect(),
                },
            )
            .into_value()
        } else {
            Value::Int(integer)
        };
        let argument = if spread {
            let pointer = vm
                .memory_mut()
                .allocate(&f.types, unsigned, Some(source))
                .unwrap();
            Value::Slice {
                ty: slice,
                pointer,
                count: 1,
            }
        } else {
            source
        };
        let allocations = vm.memory().allocation_count();
        let checkpoint_work = vm.test_budget_source_work(fuel, false, 0);
        let result = vm.execute(ProcedureId::new(0), vec![argument]);
        assert_eq!(vm.memory().allocation_count(), allocations);
        (result, checkpoint_work)
    }
    for spread in [false, true] {
        let (plain, plain_checkpoint) = run(spread, false, false, 10_000);
        let (addressed, addressed_checkpoint) = run(spread, true, false, 10_000);
        assert_eq!(plain.outcome, Outcome::Complete(vec![value(1)]));
        assert_eq!(addressed.outcome, plain.outcome);
        assert_eq!(
            (addressed.statistics.steps - addressed_checkpoint)
                - (plain.statistics.steps - plain_checkpoint),
            // Both paths charge origins while constructing the cold source
            // image, snapshotting it, and assembling the final pack. A scalar
            // additionally clones its selected value before the raw snapshot.
            if spread {
                3
            } else {
                4
            } * ORIGINS as u64,
        );
        let (final_copy, _) = run(
            spread,
            true,
            false,
            addressed.statistics.steps - addressed_checkpoint - ORIGINS as u64 / 2,
        );
        assert_eq!(
            final_copy.outcome,
            Outcome::Failed(Error::Limit(LimitKind::Fuel))
        );
        assert_eq!(final_copy.statistics.calls, 1);
        let (pending, pending_checkpoint) = run(spread, false, true, 10_000);
        assert_eq!(
            pending.outcome,
            Outcome::Pending(vec![Dependency::Procedure(ProcedureId::new(2))])
        );
        let (first_copy, _) = run(
            spread,
            true,
            true,
            pending.statistics.steps - pending_checkpoint + ORIGINS as u64 / 2,
        );
        assert_eq!(
            first_copy.outcome,
            Outcome::Failed(Error::Limit(LimitKind::Fuel))
        );
        assert_eq!(first_copy.statistics.calls, 1);
    }
}

#[test]
fn frame_pack_budget_accumulates_across_calls_and_rolls_back_on_overflow() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = f.types.slice(integer_type).unwrap();
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        integer_type,
        args,
        integer_type,
        vec![ret(count(args.place(), integer_type))],
    );
    let iterator = local(&f, 0, 0);
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![iterator.local()],
        vec![
            Statement::Range(RangeLoop {
                id: LoopId::new(0),
                iterator,
                start: int(0),
                end: int(12_100),
                direction: Direction::Forward,
                body: block(vec![Statement::DiscardInt(call_int(
                    1,
                    vec![(0, pack(slice, vec![element(1)]))],
                ))]),
            }),
            ret(int(42)),
        ],
    );
    f.procedures.push(root);
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 2_000_000,
            ..Limits::default()
        },
    )
    .unwrap();
    for _ in 0..2 {
        let result = vm.execute(ProcedureId::new(0), vec![]);
        assert_eq!(
            result.outcome,
            Outcome::Failed(Error::Limit(LimitKind::SequenceTemporaryBytes))
        );
        assert_eq!(result.statistics.calls, 1 + 1_048_576 / 87);
        assert_eq!(vm.memory().allocation_count(), 0);
    }
}

#[test]
fn oversized_scalar_prefix_fails_before_building_more_snapshots() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let byte = f.types.scalar(ScalarType::Int(IntegerType::U8));
    let padded = f.types.reserve_record(RecordKind::Struct);
    f.types
        .define_record_with_layout(
            padded,
            [byte],
            jai_types::RecordLayout {
                minimum_alignment: Some(262_144),
                ..jai_types::RecordLayout::default()
            },
        )
        .unwrap();
    let slice = f.types.slice(padded).unwrap();
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        padded,
        args,
        integer_type,
        vec![ret(count(args.place(), integer_type))],
    );
    let pending = signature(&mut f.types, vec![], vec![padded]);
    f.signatures.insert(ProcedureId::new(2), pending);
    f.pending = Some(ProcedureId::new(2));
    let mut parts: Vec<_> = (0..5)
        .map(|_| SequencePackPart::Element(ValueExpr::Zero(padded)))
        .collect();
    parts.push(SequencePackPart::Element(ValueExpr::Call {
        call: call(2, vec![]),
        ty: padded,
    }));
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 5_000_000,
            value_cells: 2_000_000,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        vm.evaluate_call(&call(1, vec![(0, pack(slice, parts))]))
            .outcome,
        Outcome::Failed(Error::Limit(LimitKind::SequenceTemporaryBytes))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}

#[test]
fn empty_and_nonempty_zero_sized_packs_use_null_and_sentinel_data() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let empty = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(empty, []).unwrap();
    let slice = f.types.slice(empty).unwrap();
    let pointer = f.types.pointer(empty).unwrap();
    let array = f.types.fixed_array(empty, 2).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Zero,
        },
        &f.types,
    )
    .unwrap();
    let global_place = global.place();
    f.globals.push(global);
    let args = typed_local(&f, 1, 0, slice);
    let nonnull = BoolExpr::FromPointer(Box::new(ValueExpr::SequenceField {
        base: Box::new(ValueExpr::Load(args.place())),
        field: SequenceField::Data,
        ty: pointer,
    }));
    variadic(
        &mut f,
        1,
        empty,
        args,
        integer_type,
        vec![ret(binary(
            IntOp::Add,
            count(args.place(), integer_type),
            IntExpr::new(IntegerType::S64, IntExprKind::FromBool(Box::new(nonnull))),
        ))],
    );
    let indexed_args = typed_local(&f, 2, 0, slice);
    let mut places = PlaceRegistry::new();
    let second = places
        .index(indexed_args.place(), int(1), &f.types)
        .unwrap();
    f.places = places.freeze();
    variadic(
        &mut f,
        2,
        empty,
        indexed_args,
        integer_type,
        vec![
            Statement::DiscardValue(index(indexed_args.place(), 1, empty)),
            Statement::DiscardValue(ValueExpr::Load(second)),
            ret(int(42)),
        ],
    );
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate_call(&call(1, vec![(0, pack(slice, vec![]))]))
            .outcome,
        Outcome::Complete(vec![value(0)])
    );
    assert_eq!(
        vm.evaluate_call(&call(
            1,
            vec![(
                0,
                pack(
                    slice,
                    vec![
                        SequencePackPart::Element(ValueExpr::Zero(empty)),
                        SequencePackPart::Element(ValueExpr::Zero(empty))
                    ]
                )
            )]
        ))
        .outcome,
        Outcome::Complete(vec![value(3)])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(
        vm.evaluate_call(&call(
            1,
            vec![(
                0,
                pack(
                    slice,
                    vec![SequencePackPart::Spread(ValueExpr::ArrayToSlice {
                        array: global_place,
                        ty: slice
                    })]
                )
            )]
        ))
        .outcome,
        Outcome::Complete(vec![value(3)])
    );
    assert_eq!(vm.memory().allocation_count(), 1);
    assert_eq!(
        vm.evaluate_call(&call(
            2,
            vec![(
                0,
                pack(
                    slice,
                    vec![
                        SequencePackPart::Element(ValueExpr::Zero(empty)),
                        SequencePackPart::Spread(ValueExpr::ArrayToSlice {
                            array: global_place,
                            ty: slice,
                        }),
                    ],
                ),
            )],
        ))
        .outcome,
        Outcome::Complete(vec![value(42)])
    );
}

#[test]
fn invalid_spread_descriptors_fail_before_allocating_pack_storage() {
    let mut f = fixture();
    let integer_type = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = f.types.slice(integer_type).unwrap();
    let pointer = f.types.pointer(integer_type).unwrap();
    let array = f.types.fixed_array(integer_type, 1).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Zero,
        },
        &f.types,
    )
    .unwrap();
    let global_place = global.place();
    f.globals.push(global);
    let args = typed_local(&f, 1, 0, slice);
    variadic(
        &mut f,
        1,
        integer_type,
        args,
        integer_type,
        vec![ret(count(args.place(), integer_type))],
    );
    let descriptors = [
        (
            i64::MIN.into(),
            ValueExpr::Zero(pointer),
            Error::CheckedCast,
        ),
        (1, ValueExpr::Zero(pointer), Error::NullPointer),
        (
            131_073,
            ValueExpr::SequenceField {
                base: Box::new(ValueExpr::ArrayToSlice {
                    array: global_place,
                    ty: slice,
                }),
                field: SequenceField::Data,
                ty: pointer,
            },
            Error::Limit(LimitKind::SequenceTemporaryBytes),
        ),
        (
            2,
            ValueExpr::SequenceField {
                base: Box::new(ValueExpr::ArrayToSlice {
                    array: global_place,
                    ty: slice,
                }),
                field: SequenceField::Data,
                ty: pointer,
            },
            Error::OutOfBounds {
                index: 16,
                length: 8,
            },
        ),
    ];
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    for (count, data, expected) in descriptors {
        let descriptor = ValueExpr::SequenceBuild {
            ty: slice,
            initializers: vec![
                (SequenceField::Count, ValueExpr::Int(int(count))),
                (SequenceField::Data, data),
            ],
        };
        assert_eq!(
            vm.evaluate_call(&call(
                1,
                vec![(0, pack(slice, vec![SequencePackPart::Spread(descriptor)]))],
            ))
            .outcome,
            Outcome::Failed(expected)
        );
        assert_eq!(vm.memory().allocation_count(), 0);
    }
}
