use super::*;

fn zero_stride_array(f: &mut Fixture, count: u64) -> TypeId {
    let empty = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(empty, []).unwrap();
    f.types.fixed_array(empty, count).unwrap()
}

#[test]
fn large_zero_stride_expression_and_global_fail_fuel_before_storage() {
    let mut f = fixture();
    let array = zero_stride_array(&mut f, 100_000);
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Zero,
        },
        &f.types,
    )
    .unwrap();
    let place = global.place();
    f.globals.push(global);
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 128,
            ..Limits::default()
        },
    )
    .unwrap();
    for expression in [ValueExpr::Zero(array), ValueExpr::Load(place)] {
        assert_eq!(
            vm.evaluate(&expression).outcome,
            Outcome::Failed(Error::Limit(LimitKind::Fuel))
        );
        assert_eq!(vm.memory().allocation_count(), 0);
        assert_eq!(vm.memory().value_cells(), 0);
    }
}

#[test]
fn default_record_preflight_stops_before_a_pending_explicit_initializer() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = zero_stride_array(&mut f, 8192);
    let record = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(record, [array, integer]).unwrap();
    let initialized = f.types.field(record, 1).unwrap().id;
    let defaulted = f.types.field(record, 0).unwrap().id;
    let pending_signature = signature(&mut f.types, vec![], vec![integer]);
    f.signatures.insert(ProcedureId::new(9), pending_signature);
    f.pending = Some(ProcedureId::new(9));
    let expression = ValueExpr::RecordBuild {
        ty: record,
        initializers: vec![
            (initialized, ValueExpr::Int(call_int(9, vec![]))),
            (defaulted, ValueExpr::Zero(array)),
        ],
    };
    for (fuel, expected) in [
        (128, Outcome::Failed(Error::Limit(LimitKind::Fuel))),
        (
            20_000,
            Outcome::Pending(vec![Dependency::Procedure(ProcedureId::new(9))]),
        ),
    ] {
        let mut vm = Vm::new(
            &f,
            NoEffects,
            Limits {
                fuel,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(vm.evaluate(&expression).outcome, expected);
        assert_eq!(vm.memory().allocation_count(), 0);
    }
}

#[test]
fn repeated_default_record_builds_charge_each_expanded_shape() {
    let mut f = fixture();
    let array = zero_stride_array(&mut f, 256);
    let record = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(record, [array]).unwrap();
    let field = f.types.field(record, 0).unwrap().id;
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
                end: int(19),
                direction: Direction::Forward,
                body: block(vec![Statement::DiscardValue(ValueExpr::RecordBuild {
                    ty: record,
                    initializers: vec![(field, ValueExpr::Zero(array))],
                })]),
            }),
            ret(int(42)),
        ],
    );
    f.procedures.push(root);
    let mut limited = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 1000,
            ..Limits::default()
        },
    )
    .unwrap();
    let result = limited.execute(ProcedureId::new(0), vec![]);
    assert_eq!(
        result.outcome,
        Outcome::Failed(Error::Limit(LimitKind::Fuel))
    );
    assert!(result.statistics.steps >= 258 + 257 + 257);
    assert_eq!(limited.memory().allocation_count(), 0);
    let mut ready = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 20_000,
            ..Limits::default()
        },
    )
    .unwrap();
    let result = ready.execute(ProcedureId::new(0), vec![]);
    assert_eq!(result.outcome, Outcome::Complete(vec![value(42)]));
    assert!(result.statistics.steps >= 20 * (258 + 257));
}

#[test]
fn union_zero_uses_first_field_and_incomplete_zero_remains_a_dependency() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let array = zero_stride_array(&mut f, 100_000);
    let union = f.types.reserve_record(RecordKind::Union);
    f.types.define_record(union, [integer, array]).unwrap();
    let pending = f.types.reserve_record(RecordKind::Struct);
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 128,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::Zero(union)).outcome,
        Outcome::Complete(vec![Value::Union {
            ty: union,
            field: 0,
            value: Box::new(value(0)),
        }])
    );
    assert_eq!(
        vm.evaluate(&ValueExpr::Zero(pending)).outcome,
        Outcome::Pending(vec![Dependency::Type(pending)])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}
