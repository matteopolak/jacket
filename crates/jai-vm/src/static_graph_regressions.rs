use super::*;
use std::sync::Arc;

#[test]
fn cyclic_static_graphs_are_readonly_cached_and_survive_ready_snapshot_transfer() {
    let mut f = fixture();
    let node = f.types.reserve_record(RecordKind::Struct);
    let next = f.types.pointer(node).unwrap();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    f.types.define_record(node, [integer, next]).unwrap();
    let mut builder = StaticDataBuilder::new();
    let a = builder.reserve(node, &f.types).unwrap();
    let b = builder.reserve(node, &f.types).unwrap();
    for (id, referent, number) in [(a, b, 3), (b, a, 5)] {
        builder
            .define(
                id,
                StaticValue {
                    ty: node,
                    kind: StaticValueKind::Record(vec![
                        StaticValue::constant(ConstantValue {
                            ty: integer,
                            kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, number)),
                        }),
                        StaticValue {
                            ty: next,
                            kind: StaticValueKind::Address(StaticAddress::new(referent)),
                        },
                    ]),
                },
            )
            .unwrap();
    }
    let data = Arc::new(
        builder
            .finish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let expression = ValueExpr::StaticAddress {
        data,
        address: StaticAddress::new(a),
        ty: next,
    };
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let result = vm.evaluate(&expression);
    let Outcome::Complete(values) = result.outcome else {
        panic!("{result:?}")
    };
    let first = values[0].pointer().unwrap().clone();
    let second = vm
        .memory()
        .load(&f.types, &vm.memory().field(&f.types, &first, 1).unwrap())
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    let roundtrip = vm
        .memory()
        .load(&f.types, &vm.memory().field(&f.types, &second, 1).unwrap())
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    assert!(
        vm.memory()
            .same_address(&f.types, &first, &roundtrip)
            .unwrap()
    );
    assert_eq!(
        vm.memory()
            .load(&f.types, &vm.memory().field(&f.types, &second, 0).unwrap())
            .unwrap(),
        value(5)
    );
    let first_value = vm.memory().field(&f.types, &first, 0).unwrap();
    assert_eq!(
        vm.memory_mut().store(&f.types, &first_value, value(8)),
        Err(Error::ReadOnlyStorage)
    );
    assert_eq!(vm.memory_mut().release(&first), Err(Error::ReadOnlyStorage));
    assert_eq!(vm.memory().allocation_count(), 2);
    let state = vm.into_state();
    let mut vm = Vm::with_state(&f, NoEffects, Limits::default(), state).unwrap();
    assert_eq!(
        vm.evaluate(&expression).outcome,
        Outcome::Complete(vec![Value::Pointer(first)])
    );
    assert_eq!(vm.memory().allocation_count(), 2);
}

#[test]
fn failed_static_publication_discards_cache_and_never_recycles_virtual_addresses() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = f.types.pointer(integer).unwrap();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(integer, &f.types).unwrap();
    builder
        .define(
            object,
            StaticValue::constant(ConstantValue {
                ty: integer,
                kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, 7)),
            }),
        )
        .unwrap();
    let data = Arc::new(
        builder
            .finish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let expression = ValueExpr::StaticAddress {
        data,
        address: StaticAddress::new(object),
        ty: pointer,
    };
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let mut abandoned = None;
    let result = vm.evaluate_validated(&expression, |_, values| {
        abandoned = Some(values[0].pointer().unwrap().clone());
        Err(Error::InvalidIr("pointer publication rejected"))
    });
    assert_eq!(
        result.outcome,
        Outcome::Failed(Error::InvalidIr("pointer publication rejected"))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    let result = vm.evaluate(&expression);
    let Outcome::Complete(values) = result.outcome else {
        panic!("{result:?}")
    };
    let live = values[0].pointer().unwrap();
    let abandoned = abandoned.unwrap();
    assert_ne!(live, &abandoned);
    assert_eq!(
        vm.memory().load(&f.types, &abandoned),
        Err(Error::DanglingPointer)
    );
    assert_eq!(vm.memory().load(&f.types, live).unwrap(), value(7));
}
