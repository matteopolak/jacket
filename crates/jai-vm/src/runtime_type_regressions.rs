use super::*;
use jai_types::{LayoutPolicy, ReflectionGraph, ReflectionMetadata, ReflectionReadiness};
use std::sync::Arc;

fn catalog_builder() -> (Fixture, StaticDataBuilder, StaticObjectId, TypeId) {
    let mut f = fixture();
    let tag = f.types.reserve_enum(IntegerType::U32);
    f.types
        .define_enum(
            tag,
            [0, 1, 13].map(|v| Integer::wrapping(IntegerType::U32, v)),
        )
        .unwrap();
    let header = f.types.reserve_record(RecordKind::Struct);
    let descriptor = f.types.reserve_record(RecordKind::Struct);
    let size = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = f.types.scalar(ScalarType::Bool);
    f.types.define_record(header, [tag, size]).unwrap();
    f.types
        .define_record(descriptor, [header, boolean])
        .unwrap();
    f.types.bind_runtime_type_header(header).unwrap();
    let represented = f.types.scalar(ScalarType::Int(IntegerType::S32));
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &f.types,
        represented,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("scalar ready");
    };
    let scalar = |ty, kind| StaticValue::constant(ConstantValue { ty, kind });
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(descriptor, &f.types).unwrap();
    builder
        .define_type_descriptor(
            object,
            StaticValue {
                ty: descriptor,
                kind: StaticValueKind::Record(vec![
                    StaticValue {
                        ty: header,
                        kind: StaticValueKind::Record(vec![
                            scalar(
                                tag,
                                ConstantKind::Enum(Integer::wrapping(IntegerType::U32, 0)),
                            ),
                            scalar(
                                size,
                                ConstantKind::Int(Integer::wrapping(IntegerType::S64, 4)),
                            ),
                        ]),
                    },
                    scalar(boolean, ConstantKind::Bool(true)),
                ]),
            },
            &graph,
            graph.root(),
            &f.types,
        )
        .unwrap();
    (f, builder, object, header)
}
fn catalog() -> (Fixture, RuntimeTypeConstant, TypeId) {
    let (f, mut builder, object, header) = catalog_builder();
    let data = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let constant = RuntimeTypeConstant::new(data, object, &f.types).unwrap();
    (f, constant, header)
}

#[test]
fn type_publication_retains_the_newest_committed_prefix_without_downgrading() {
    let (mut f, mut builder, object, _) = catalog_builder();
    let boolean = f.types.scalar(ScalarType::Bool);
    let bool_pointer = f.types.pointer(boolean).unwrap();
    let first = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let constant = RuntimeTypeConstant::new(Arc::clone(&first), object, &f.types).unwrap();
    let extra = builder.reserve(boolean, &f.types).unwrap();
    builder
        .define(
            extra,
            StaticValue::constant(ConstantValue {
                ty: boolean,
                kind: ConstantKind::Bool(true),
            }),
        )
        .unwrap();
    let second = Arc::new(
        builder
            .publish(&f.types, StaticDataLimits::default())
            .unwrap(),
    );
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let value = complete(vm.evaluate(&ValueExpr::RuntimeType(constant.clone())));
    let appended = ValueExpr::StaticAddress {
        data: Arc::clone(&second),
        address: StaticAddress::new(extra),
        ty: bool_pointer,
    };
    let rejected = vm.evaluate_validated(&appended, |vm, _| {
        assert!(Arc::ptr_eq(
            vm.runtime_type_constant_value(&value)?.data(),
            &second
        ));
        Err(Error::InvalidIr("reject appended publication"))
    });
    assert!(matches!(rejected.outcome, Outcome::Failed(_)));
    assert!(Arc::ptr_eq(
        vm.runtime_type_constant_value(&value).unwrap().data(),
        &first
    ));
    complete(vm.evaluate(&appended));
    complete(vm.evaluate(&ValueExpr::RuntimeType(constant)));
    assert!(Arc::ptr_eq(
        vm.runtime_type_constant_value(&value).unwrap().data(),
        &second
    ));
    let resumed = Vm::with_state(&f, NoEffects, Limits::default(), vm.into_state()).unwrap();
    assert!(Arc::ptr_eq(
        resumed.runtime_type_constant_value(&value).unwrap().data(),
        &second
    ));
}
fn complete(execution: Execution) -> Value {
    let Outcome::Complete(mut values) = execution.outcome else {
        panic!("{execution:?}");
    };
    assert_eq!(values.len(), 1);
    values.remove(0)
}

#[test]
fn runtime_type_identity_survives_raw_descriptor_aliases_and_state_transfer() {
    let (f, constant, header) = catalog();
    let meta = f.types.meta_type();
    let descriptor_type = constant.descriptor_type();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let value = complete(vm.evaluate(&ValueExpr::RuntimeType(constant.clone())));
    assert_eq!(
        vm.runtime_type_identity(&value).unwrap(),
        constant.identity()
    );
    let published = vm.runtime_type_constant_value(&value).unwrap();
    assert_eq!(published, constant);
    assert!(Arc::ptr_eq(published.data(), constant.data()));
    let pointer = complete(vm.evaluate(&ValueExpr::TypeDescriptor {
        value: Box::new(ValueExpr::RuntimeType(constant.clone())),
        ty: descriptor_type,
    }))
    .pointer()
    .unwrap()
    .clone();
    assert_eq!(pointer.pointee(), header);
    let slot = vm
        .memory_mut()
        .allocate(&f.types, meta, Some(value.clone()))
        .unwrap();
    let alias = vm
        .memory()
        .cast_pointer(
            &f.types,
            &slot,
            descriptor_type,
            jai_types::CastMode::Checked,
        )
        .unwrap();
    assert_eq!(
        vm.memory().load(&f.types, &alias).unwrap(),
        Value::Pointer(pointer.clone())
    );
    vm.memory_mut()
        .store(&f.types, &alias, Value::Pointer(pointer))
        .unwrap();
    assert_eq!(
        vm.runtime_type_identity(&vm.memory().load(&f.types, &slot).unwrap())
            .unwrap(),
        constant.identity()
    );
    let state = vm.into_state();
    let resumed = Vm::with_state(&f, NoEffects, Limits::default(), state).unwrap();
    assert_eq!(
        resumed.runtime_type_identity(&value).unwrap(),
        constant.identity()
    );
    assert_eq!(
        resumed.runtime_type_constant_value(&value).unwrap(),
        constant
    );
}

#[test]
fn matching_header_bytes_and_raw_alias_writes_cannot_forge_type_identity() {
    let (f, constant, header) = catalog();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let certified = complete(vm.evaluate(&ValueExpr::RuntimeType(constant.clone())));
    let Value::Type {
        descriptor: Some(pointer),
    } = &certified
    else {
        panic!("nonnull Type");
    };
    let header_value = vm.memory().load(&f.types, pointer).unwrap();
    let imitation = vm
        .memory_mut()
        .allocate(&f.types, header, Some(header_value))
        .unwrap();
    let forged = Value::Type {
        descriptor: Some(imitation.clone()),
    };
    let error = Error::InvalidIr("runtime Type value does not name a canonical descriptor");
    assert_eq!(vm.runtime_type_identity(&forged), Err(error.clone()));
    assert_eq!(vm.runtime_type_constant_value(&forged), Err(error.clone()));
    assert_eq!(
        vm.memory_mut()
            .allocate(&f.types, f.types.meta_type(), Some(forged)),
        Err(error.clone())
    );
    let slot = vm
        .memory_mut()
        .allocate(&f.types, f.types.meta_type(), Some(certified))
        .unwrap();
    let alias = vm
        .memory()
        .cast_pointer(
            &f.types,
            &slot,
            constant.descriptor_type(),
            jai_types::CastMode::Checked,
        )
        .unwrap();
    vm.memory_mut()
        .store(&f.types, &alias, Value::Pointer(imitation))
        .unwrap();
    assert_eq!(vm.memory().load(&f.types, &slot), Err(error));
}

#[test]
fn failed_type_publication_discards_identity_and_successful_retry_uses_fresh_storage() {
    let (f, constant, _) = catalog();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let mut abandoned = None;
    let expression = ValueExpr::RuntimeType(constant.clone());
    assert_eq!(
        vm.evaluate_validated(&expression, |_, values| {
            abandoned = Some(values[0].clone());
            Err(Error::InvalidIr("reject publication"))
        })
        .outcome,
        Outcome::Failed(Error::InvalidIr("reject publication"))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(
        vm.runtime_type_identity(abandoned.as_ref().unwrap()),
        Err(Error::DanglingPointer)
    );
    assert_eq!(
        vm.runtime_type_constant_value(abandoned.as_ref().unwrap()),
        Err(Error::DanglingPointer)
    );
    let fresh = complete(vm.evaluate(&expression));
    assert_ne!(fresh, abandoned.unwrap());
    assert_eq!(
        vm.runtime_type_identity(&fresh).unwrap(),
        constant.identity()
    );
    assert_eq!(vm.runtime_type_constant_value(&fresh).unwrap(), constant);
}

#[test]
fn runtime_type_constants_inside_global_aggregates_are_hydrated_by_the_vm() {
    let (mut f, constant, _) = catalog();
    let record = f.types.reserve_record(RecordKind::Struct);
    f.types
        .define_record(record, [f.types.meta_type()])
        .unwrap();
    f.globals.push(Global::new(
        0,
        GlobalInitializer::Value(ConstantValue {
            ty: record,
            kind: ConstantKind::Record(vec![ConstantValue {
                ty: f.types.meta_type(),
                kind: ConstantKind::RuntimeType(constant.clone()),
            }]),
        }),
        &f.types,
    ));
    let expression = ValueExpr::Load(f.globals[0].place());
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let Value::Record { fields, .. } = complete(vm.evaluate(&expression)) else {
        panic!("record result");
    };
    assert_eq!(
        vm.runtime_type_identity(&fields[0]).unwrap(),
        constant.identity()
    );
    assert_eq!(
        vm.runtime_type_constant_value(&fields[0]).unwrap(),
        constant
    );
}

#[test]
fn runtime_type_publication_converter_runs_inside_the_validation_transaction() {
    let (f, constant, _) = catalog();
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    let mut published = None;
    let execution =
        vm.evaluate_validated(&ValueExpr::RuntimeType(constant.clone()), |vm, values| {
            published = Some(vm.runtime_type_constant_value(&values[0])?);
            Ok(())
        });
    assert!(matches!(execution.outcome, Outcome::Complete(_)));
    assert_eq!(published.unwrap(), constant);
    assert!(
        vm.runtime_type_constant_value(&Value::Type { descriptor: None })
            .is_err()
    );
}

#[test]
fn runtime_type_target_mismatch_is_rejected_before_any_descriptor_storage_is_allocated() {
    use jai_types::ScalarLayout;
    let (f, constant, _) = catalog();
    let policy = LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 8),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
        ScalarLayout::new(1, 1),
    )
    .unwrap();
    let mut vm = Vm::new_with_target(
        &f,
        NoEffects,
        Limits::default(),
        ByteTarget {
            policy,
            endian: Endian::Little,
        },
    )
    .unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::RuntimeType(constant)).outcome,
        Outcome::Failed(Error::InvalidIr(
            "runtime Type descriptor uses a different target layout"
        ))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}
